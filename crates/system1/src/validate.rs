//! Deterministic guardrails applied after the model answers.
//!
//! The model proposes; this module disposes. Every decision is checked against
//! the caller's [`RiskPolicy`] before it can become an order. Adjustments are
//! recorded instead of silently changing the model's intent.

use crate::features::MarketFeatures;
use crate::types::{
    Action, OrderType, RiskPolicy, Side, TradeDecision, TradingContext, ValidationReport,
};

#[derive(Debug, Clone)]
pub struct ValidatedDecision {
    pub decision: TradeDecision,
    pub report: ValidationReport,
}

pub fn validate(
    proposal: TradeDecision,
    ctx: &TradingContext,
    features: &MarketFeatures,
    policy: &RiskPolicy,
) -> ValidatedDecision {
    let policy = policy.normalized();
    let mut report = ValidationReport::default();
    let mut decision = proposal.clone();

    let (confidence, note) = normalize_confidence(decision.confidence);
    decision.confidence = confidence;
    if let Some(note) = note {
        report.adjustments.push(note);
    }

    match decision.action {
        Action::Hold => {
            if decision.quantity.is_some()
                || decision.notional.is_some()
                || decision.order_type.is_some()
            {
                report
                    .warnings
                    .push("hold decision ignored its order fields".to_string());
            }
            decision.clear_order_fields();
            if decision.rationale.trim().is_empty() {
                decision.rationale = "Hold: no qualifying setup.".to_string();
            }
            report.approved = true;
            ValidatedDecision { decision, report }
        }
        Action::Close => validate_close(decision, ctx, features, &policy, report),
        Action::Buy | Action::Sell => validate_entry(decision, ctx, features, &policy, report),
    }
}

fn validate_entry(
    mut decision: TradeDecision,
    ctx: &TradingContext,
    features: &MarketFeatures,
    policy: &RiskPolicy,
    mut report: ValidationReport,
) -> ValidatedDecision {
    if let Err(reason) = entry_gates(ctx, features, policy, decision.confidence) {
        return block(decision, reason, report);
    }

    let side = if decision.action == Action::Buy {
        Side::Buy
    } else {
        Side::Sell
    };

    let order_type = decision.order_type.unwrap_or(OrderType::Market);
    if order_type.is_protective() {
        return block(
            decision,
            "stop orders are not supported by the decision layer; use a market or limit entry with a stop_loss",
            report,
        );
    }

    let mut limit_price = match decision.limit_price {
        Some(value) if value.is_finite() && value > 0.0 => {
            Some(round_tick(value, policy.price_tick))
        }
        Some(_) => {
            return block(decision, "limit_price must be a positive number", report);
        }
        None => None,
    };

    if order_type == OrderType::Limit && limit_price.is_none() {
        return block(decision, "limit order requires limit_price", report);
    }
    if order_type == OrderType::Market && limit_price.is_some() {
        report
            .warnings
            .push("market order ignored limit_price".to_string());
        limit_price = None;
    }

    let market_price = match side {
        Side::Buy => ctx.market.ask,
        Side::Sell => ctx.market.bid,
    };
    if !market_price.is_finite() || market_price <= 0.0 {
        return block(decision, "no executable quote", report);
    }

    if let Some(limit) = limit_price {
        let deviation_bps = ((limit - features.mid).abs() / features.mid) * 10_000.0;
        if deviation_bps > policy.max_limit_deviation_bps {
            return block(
                decision,
                format!(
                    "limit_price is {deviation_bps:.1} bps from mid (max {:.1})",
                    policy.max_limit_deviation_bps
                ),
                report,
            );
        }
    }

    let entry_price = limit_price.unwrap_or(market_price);
    if !entry_price.is_finite() || entry_price <= 0.0 {
        return block(decision, "entry price is invalid", report);
    }

    let current_quantity = match ctx.position {
        Some(position) if !position.quantity.is_finite() => {
            return block(decision, "position quantity is not finite", report);
        }
        Some(position) => position.quantity,
        None => 0.0,
    };

    let mut quantity = match resolve_quantity(
        &decision,
        side,
        entry_price,
        current_quantity,
        policy,
        &mut report,
    ) {
        Ok(quantity) => quantity,
        Err(reason) => return block(decision, reason, report),
    };

    if let Some(max_quantity) = policy.max_quantity
        && quantity > max_quantity
    {
        report.adjustments.push(format!(
            "quantity {quantity:.8} capped at max_quantity {max_quantity:.8}"
        ));
        quantity = max_quantity;
    }

    let equity = ctx.account.equity;
    let max_order_notional = equity * policy.max_order_notional_pct;
    if max_order_notional <= 0.0 {
        return block(decision, "max order notional is zero", report);
    }
    if quantity * entry_price > max_order_notional {
        let capped = max_order_notional / entry_price;
        report.adjustments.push(format!(
            "quantity {quantity:.8} capped by max_order_notional ({max_order_notional:.2}) to {capped:.8}"
        ));
        quantity = capped;
    }

    if side == Side::Buy && !policy.allow_margin {
        let cash = ctx.account.cash;
        if !cash.is_finite() || cash <= 0.0 {
            return block(decision, "no buying power", report);
        }
        let cost_per_unit = entry_price * (1.0 + policy.fee_bps / 10_000.0);
        let affordable = cash / cost_per_unit;
        if quantity > affordable {
            report.adjustments.push(format!(
                "quantity {quantity:.8} capped by buying power to {affordable:.8}"
            ));
            quantity = affordable;
        }
    }

    let signed = if side == Side::Buy {
        quantity
    } else {
        -quantity
    };
    let projected = current_quantity + signed;
    let max_position_notional = equity * policy.max_position_notional_pct;

    if projected.abs() > current_quantity.abs() {
        if max_position_notional <= 0.0 {
            return block(decision, "max position notional is zero", report);
        }
        let projected_notional = projected.abs() * entry_price;
        if projected_notional > max_position_notional {
            let allowed_abs = max_position_notional / entry_price;
            if allowed_abs <= current_quantity.abs() {
                return block(
                    decision,
                    "position limit reached; reduce the existing position first",
                    report,
                );
            }
            let max_increase = allowed_abs - current_quantity.abs();
            if max_increase < quantity {
                report.adjustments.push(format!(
                    "quantity {quantity:.8} capped by max_position_notional to {max_increase:.8}"
                ));
                quantity = max_increase;
            }
        }
    }

    let gross = ctx
        .account
        .gross_exposure
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or_else(|| current_quantity.abs() * entry_price);
    let signed = if side == Side::Buy {
        quantity
    } else {
        -quantity
    };
    let projected_gross = (gross + signed * entry_price).max(0.0);
    let max_gross = equity * policy.max_gross_exposure_pct;
    if signed * entry_price > 0.0 && projected_gross > max_gross {
        let room = max_gross - gross;
        if room <= 0.0 {
            return block(decision, "gross exposure limit reached", report);
        }
        let capped = room / entry_price;
        if capped < quantity {
            report.adjustments.push(format!(
                "quantity {quantity:.8} capped by max_gross_exposure to {capped:.8}"
            ));
            quantity = capped;
        }
    }

    let stop_loss =
        match resolve_stop_loss(&decision, side, entry_price, features, policy, &mut report) {
            Ok(stop) => stop,
            Err(reason) => return block(decision, reason, report),
        };

    if let Some(stop) = stop_loss {
        let risk_per_unit = (entry_price - stop).abs();
        if risk_per_unit <= 0.0 {
            return block(decision, "stop_loss implies zero risk distance", report);
        }
        let max_risk_amount = equity * policy.max_risk_per_trade_pct;
        if max_risk_amount <= 0.0 {
            return block(decision, "max risk per trade is zero", report);
        }
        let risk_capped = max_risk_amount / risk_per_unit;
        if quantity > risk_capped {
            report.adjustments.push(format!(
                "quantity {quantity:.8} sized down to risk budget {risk_capped:.8}"
            ));
            quantity = risk_capped;
        }
    }

    let take_profit =
        match resolve_take_profit(&decision, side, entry_price, stop_loss, policy, &mut report) {
            Ok(take_profit) => take_profit,
            Err(reason) => return block(decision, reason, report),
        };

    quantity = round_down(quantity, policy.quantity_step);
    if !quantity.is_finite() || quantity <= 0.0 {
        return block(decision, "quantity rounds to zero", report);
    }
    if let Some(max_quantity) = policy.max_quantity {
        quantity = quantity.min(max_quantity);
    }
    if policy.min_quantity > 0.0 && quantity < policy.min_quantity {
        return block(
            decision,
            format!(
                "quantity {quantity:.8} is below min_quantity {:.8}",
                policy.min_quantity
            ),
            report,
        );
    }

    decision.quantity = Some(quantity);
    decision.notional = None;
    decision.order_type = Some(order_type);
    decision.limit_price = if order_type == OrderType::Limit {
        limit_price
    } else {
        None
    };
    decision.stop_loss = stop_loss;
    decision.take_profit = take_profit;
    if decision.rationale.trim().is_empty() {
        decision.rationale = "Model did not provide a rationale.".to_string();
    }

    report.approved = true;
    ValidatedDecision { decision, report }
}

fn validate_close(
    mut decision: TradeDecision,
    ctx: &TradingContext,
    features: &MarketFeatures,
    policy: &RiskPolicy,
    mut report: ValidationReport,
) -> ValidatedDecision {
    if !features.quote_valid {
        return block(decision, "no valid quote to price the close", report);
    }

    let position = match ctx.position {
        Some(position) if position.quantity.is_finite() && position.quantity != 0.0 => position,
        Some(_) => return block(decision, "position is already flat", report),
        None => return block(decision, "no position to close", report),
    };

    let order_type = decision.order_type.unwrap_or(OrderType::Market);
    if order_type.is_protective() {
        return block(
            decision,
            "close supports market or limit orders only",
            report,
        );
    }
    if order_type == OrderType::Limit && decision.limit_price.is_none() {
        return block(decision, "limit close requires limit_price", report);
    }

    let abs_position = position.abs_quantity();
    let mut quantity = match decision.quantity {
        Some(value) if value.is_finite() && value > 0.0 => value.min(abs_position),
        Some(_) => {
            report
                .warnings
                .push("invalid close quantity; closing full position".to_string());
            abs_position
        }
        None => abs_position,
    };
    if quantity < abs_position {
        report.adjustments.push(format!(
            "close quantity capped to position size {abs_position:.8}"
        ));
    }
    quantity = round_down(quantity, policy.quantity_step);
    if !quantity.is_finite() || quantity <= 0.0 {
        return block(decision, "close quantity rounds to zero", report);
    }

    let market_price = if position.quantity > 0.0 {
        ctx.market.bid
    } else {
        ctx.market.ask
    };
    let mut limit_price = match decision.limit_price {
        Some(value) if value.is_finite() && value > 0.0 => {
            Some(round_tick(value, policy.price_tick))
        }
        Some(_) => return block(decision, "limit_price must be positive", report),
        None => None,
    };

    if order_type == OrderType::Market {
        limit_price = None;
    }
    if let Some(limit) = limit_price {
        let deviation_bps = ((limit - features.mid).abs() / features.mid) * 10_000.0;
        if deviation_bps > policy.max_limit_deviation_bps {
            return block(
                decision,
                format!(
                    "close limit is {deviation_bps:.1} bps from mid (max {:.1})",
                    policy.max_limit_deviation_bps
                ),
                report,
            );
        }
    }

    if !market_price.is_finite() || market_price <= 0.0 {
        return block(decision, "no executable quote to close", report);
    }

    if decision.confidence < policy.min_confidence {
        report.warnings.push(format!(
            "close confidence {:.2} is below min_confidence; risk reduction allowed",
            decision.confidence
        ));
    }
    if decision.stop_loss.is_some() || decision.take_profit.is_some() {
        report
            .warnings
            .push("close decision ignored stop_loss/take_profit".to_string());
    }

    decision.action = Action::Close;
    decision.quantity = Some(quantity);
    decision.notional = None;
    decision.order_type = Some(order_type);
    decision.limit_price = if order_type == OrderType::Limit {
        limit_price
    } else {
        None
    };
    decision.stop_loss = None;
    decision.take_profit = None;
    if decision.rationale.trim().is_empty() {
        decision.rationale = "Close requested by the model.".to_string();
    }

    report.approved = true;
    ValidatedDecision { decision, report }
}

fn entry_gates(
    ctx: &TradingContext,
    features: &MarketFeatures,
    policy: &RiskPolicy,
    confidence: f64,
) -> Result<(), String> {
    if !features.quote_valid {
        return Err("quote is invalid".to_string());
    }
    if !ctx.market.session_open {
        return Err("market session is closed".to_string());
    }
    if policy.require_fresh_quote && ctx.market.stale {
        return Err("quote is stale".to_string());
    }
    if features.candle_count < policy.min_candles {
        return Err(format!(
            "only {} valid candles; need {}",
            features.candle_count, policy.min_candles
        ));
    }
    if features.spread_bps > policy.max_spread_bps {
        return Err(format!(
            "spread {:.1} bps exceeds max {:.1} bps",
            features.spread_bps, policy.max_spread_bps
        ));
    }
    if !ctx.account.equity.is_finite() || ctx.account.equity <= 0.0 {
        return Err("account equity is not positive".to_string());
    }
    if daily_loss_limit_hit(ctx, policy) {
        return Err(format!(
            "daily loss limit {:.1}% reached",
            policy.max_daily_loss_pct * 100.0
        ));
    }
    if let Some(seconds) = ctx.seconds_since_last_trade
        && seconds < policy.min_seconds_between_trades
    {
        return Err(format!(
            "cooldown active: {seconds}s since last trade, need {}s",
            policy.min_seconds_between_trades
        ));
    }
    if !confidence.is_finite() || confidence < policy.min_confidence {
        return Err(format!(
            "confidence {:.2} is below min {:.2}",
            confidence, policy.min_confidence
        ));
    }
    Ok(())
}

fn daily_loss_limit_hit(ctx: &TradingContext, policy: &RiskPolicy) -> bool {
    let equity = ctx.account.equity;
    let day_pnl = ctx.account.day_pnl;
    if !equity.is_finite() || equity <= 0.0 || !day_pnl.is_finite() || day_pnl >= 0.0 {
        return false;
    }
    (-day_pnl / equity) >= policy.max_daily_loss_pct
}

fn resolve_quantity(
    decision: &TradeDecision,
    side: Side,
    entry_price: f64,
    current_quantity: f64,
    policy: &RiskPolicy,
    report: &mut ValidationReport,
) -> Result<f64, String> {
    let quantity = decision
        .quantity
        .filter(|value| value.is_finite() && *value > 0.0);
    let notional = decision
        .notional
        .filter(|value| value.is_finite() && *value > 0.0);

    let mut quantity = match (quantity, notional) {
        (Some(quantity), Some(notional)) => {
            let implied = notional / entry_price;
            if (implied - quantity).abs() / quantity > 0.05 {
                report.warnings.push(
                    "quantity and notional disagree by more than 5%; using quantity".to_string(),
                );
            }
            quantity
        }
        (Some(quantity), None) => quantity,
        (None, Some(notional)) => notional / entry_price,
        (None, None) => {
            return Err("buy/sell requires quantity or notional".to_string());
        }
    };

    if !quantity.is_finite() || quantity <= 0.0 {
        return Err("quantity must be positive".to_string());
    }

    if side == Side::Sell && !policy.allow_short {
        let long_quantity = current_quantity.max(0.0);
        if long_quantity <= 0.0 {
            return Err("shorting is disabled and there is no long position".to_string());
        }
        if quantity > long_quantity {
            report.adjustments.push(format!(
                "quantity {quantity:.8} reduced to long position {long_quantity:.8} (shorting disabled)"
            ));
            quantity = long_quantity;
        }
    }

    if !quantity.is_finite() || quantity <= 0.0 {
        return Err("quantity is zero after position limits".to_string());
    }

    Ok(quantity)
}

fn resolve_stop_loss(
    decision: &TradeDecision,
    side: Side,
    entry_price: f64,
    features: &MarketFeatures,
    policy: &RiskPolicy,
    report: &mut ValidationReport,
) -> Result<Option<f64>, String> {
    if let Some(raw) = decision.stop_loss {
        if !raw.is_finite() || raw <= 0.0 {
            return Err("stop_loss must be a positive number".to_string());
        }
        let stop = round_tick(raw, policy.price_tick);
        validate_stop_side(side, entry_price, stop)?;
        let distance_bps = ((entry_price - stop).abs() / entry_price) * 10_000.0;
        if distance_bps < policy.min_stop_distance_bps {
            return Err(format!(
                "stop_loss is {distance_bps:.1} bps from entry (min {:.1})",
                policy.min_stop_distance_bps
            ));
        }
        if distance_bps > policy.max_stop_distance_bps {
            return Err(format!(
                "stop_loss is {distance_bps:.1} bps from entry (max {:.1})",
                policy.max_stop_distance_bps
            ));
        }
        return Ok(Some(stop));
    }

    if !policy.require_stop_loss {
        return Ok(None);
    }

    let mut distance = features
        .atr_14
        .filter(|atr| atr.is_finite() && *atr > 0.0)
        .map(|atr| atr * policy.default_stop_atr_multiple)
        .unwrap_or(entry_price * policy.default_stop_pct);
    let min_distance = entry_price * policy.min_stop_distance_bps / 10_000.0;
    if distance < min_distance {
        distance = min_distance;
    }
    if !distance.is_finite() || distance <= 0.0 {
        return Err("could not derive a stop distance".to_string());
    }

    let stop = if side == Side::Buy {
        entry_price - distance
    } else {
        entry_price + distance
    };
    if !stop.is_finite() || stop <= 0.0 {
        return Err("derived stop_loss is invalid".to_string());
    }
    let stop = round_tick(stop, policy.price_tick);
    validate_stop_side(side, entry_price, stop)?;
    report.adjustments.push(format!(
        "stop_loss derived at {stop:.6} from entry {entry_price:.6}"
    ));
    Ok(Some(stop))
}

fn resolve_take_profit(
    decision: &TradeDecision,
    side: Side,
    entry_price: f64,
    stop_loss: Option<f64>,
    policy: &RiskPolicy,
    report: &mut ValidationReport,
) -> Result<Option<f64>, String> {
    let Some(raw) = decision.take_profit else {
        return Ok(None);
    };
    if !raw.is_finite() || raw <= 0.0 {
        return Err("take_profit must be a positive number".to_string());
    }
    let take_profit = round_tick(raw, policy.price_tick);
    match side {
        Side::Buy if take_profit <= entry_price => {
            return Err("take_profit must be above entry for a long".to_string());
        }
        Side::Sell if take_profit >= entry_price => {
            return Err("take_profit must be below entry for a short".to_string());
        }
        _ => {}
    }

    if let Some(stop) = stop_loss {
        let risk = (entry_price - stop).abs();
        let reward = (take_profit - entry_price).abs();
        if risk > 0.0 {
            let reward_risk = reward / risk;
            if reward_risk < policy.min_reward_risk {
                report.warnings.push(format!(
                    "take_profit reward/risk {reward_risk:.2} is below min {:.2}",
                    policy.min_reward_risk
                ));
            }
        }
    }
    Ok(Some(take_profit))
}

fn validate_stop_side(side: Side, entry_price: f64, stop: f64) -> Result<(), String> {
    match side {
        Side::Buy if stop >= entry_price => {
            Err("stop_loss must be below entry for a buy".to_string())
        }
        Side::Sell if stop <= entry_price => {
            Err("stop_loss must be above entry for a sell".to_string())
        }
        _ => Ok(()),
    }
}

fn normalize_confidence(value: f64) -> (f64, Option<String>) {
    if !value.is_finite() {
        return (
            0.0,
            Some("confidence was not finite; treated as 0".to_string()),
        );
    }
    if value > 1.0 && value <= 100.0 {
        let normalized = value / 100.0;
        return (
            normalized.clamp(0.0, 1.0),
            Some(format!("confidence {value} normalized to {normalized:.2}")),
        );
    }
    if !(0.0..=1.0).contains(&value) {
        let clamped = value.clamp(0.0, 1.0);
        return (
            clamped,
            Some(format!("confidence {value} clamped to {clamped:.2}")),
        );
    }
    (value, None)
}

fn round_down(value: f64, step: f64) -> f64 {
    if !step.is_finite() || step <= 0.0 || !value.is_finite() {
        return value;
    }
    let steps = (value / step).floor();
    let rounded = steps * step;
    (rounded * 1e10).round() / 1e10
}

fn round_tick(value: f64, tick: f64) -> f64 {
    if !tick.is_finite() || tick <= 0.0 || !value.is_finite() {
        return value;
    }
    let rounded = (value / tick).round() * tick;
    (rounded * 1e10).round() / 1e10
}

fn block(
    mut decision: TradeDecision,
    reason: impl Into<String>,
    mut report: ValidationReport,
) -> ValidatedDecision {
    let reason = reason.into();
    let rationale = if decision.rationale.trim().is_empty() {
        format!("No trade: {reason}")
    } else {
        format!("{} Guardrail: {reason}", decision.rationale.trim())
    };

    decision.action = Action::Hold;
    decision.confidence = 0.0;
    decision.clear_order_fields();
    decision.rationale = rationale;
    report.approved = false;
    report.blocked_reason = Some(reason);

    ValidatedDecision { decision, report }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AccountState, Candle, MarketKind, MarketState, OrderType, PositionState, RiskPolicy,
        TradingContext,
    };

    fn sample_context() -> TradingContext {
        let candles = (0..60)
            .map(|i| {
                let close = 100.0 + i as f64 * 0.5;
                Candle {
                    open: close - 0.2,
                    high: close + 0.8,
                    low: close - 0.8,
                    close,
                    volume: 1_000.0,
                }
            })
            .collect();
        TradingContext {
            as_of: "2026-11-18T12:00:00Z".to_string(),
            symbol: "TEST".to_string(),
            strategy: None,
            timeframe: Some("5m".to_string()),
            market: MarketState {
                market_type: MarketKind::Crypto,
                session_open: true,
                stale: false,
                bid: 129.4,
                ask: 129.6,
                last: 129.5,
                high_24h: None,
                low_24h: None,
                volume_24h: None,
                change_pct_24h: None,
                candles,
                order_book: None,
            },
            account: AccountState {
                equity: 100_000.0,
                cash: 100_000.0,
                margin_used: 0.0,
                day_pnl: 0.0,
                day_pnl_pct: None,
                open_positions: 0,
                gross_exposure: Some(0.0),
            },
            position: None,
            open_orders: Vec::new(),
            recent_trades: Vec::new(),
            seconds_since_last_trade: None,
            risk: RiskPolicy::default(),
            notes: Vec::new(),
        }
    }

    fn features(ctx: &TradingContext) -> MarketFeatures {
        MarketFeatures::compute(&ctx.market)
    }

    fn buy(quantity: f64, stop: f64) -> TradeDecision {
        TradeDecision {
            action: Action::Buy,
            confidence: 0.8,
            quantity: Some(quantity),
            notional: None,
            order_type: Some(OrderType::Market),
            limit_price: None,
            stop_loss: Some(stop),
            take_profit: None,
            time_horizon_minutes: Some(60),
            rationale: "test".to_string(),
            invalidation: None,
            risk_flags: Vec::new(),
        }
    }

    #[test]
    fn accepts_a_well_formed_buy() {
        let ctx = sample_context();
        let outcome = validate(buy(1.0, 125.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        assert_eq!(outcome.decision.action, Action::Buy);
        assert_eq!(outcome.decision.quantity, Some(1.0));
        assert!(outcome.report.adjustments.is_empty());
    }

    #[test]
    fn clamps_order_to_max_notional() {
        let ctx = sample_context();
        let outcome = validate(buy(1_000.0, 125.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        let quantity = outcome.decision.quantity.unwrap();
        assert!(quantity * 129.6 <= 5_000.0 + 1e-6);
        assert!(!outcome.report.adjustments.is_empty());
    }

    #[test]
    fn blocks_low_confidence() {
        let ctx = sample_context();
        let mut decision = buy(1.0, 125.0);
        decision.confidence = 0.4;
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert_eq!(outcome.decision.action, Action::Hold);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("confidence")
        );
    }

    #[test]
    fn blocks_short_when_disabled() {
        let ctx = sample_context();
        let mut decision = buy(1.0, 125.0);
        decision.action = Action::Sell;
        decision.stop_loss = Some(135.0);
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(outcome.report.blocked_reason.unwrap().contains("shorting"));
    }

    #[test]
    fn clamps_sell_to_long_position() {
        let mut ctx = sample_context();
        ctx.position = Some(PositionState {
            quantity: 0.5,
            average_price: 120.0,
            unrealized_pnl: 4.0,
        });
        let mut decision = buy(2.0, 135.0);
        decision.action = Action::Sell;
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        assert_eq!(outcome.decision.quantity, Some(0.5));
    }

    #[test]
    fn blocks_wide_limit_price() {
        let ctx = sample_context();
        let mut decision = buy(1.0, 125.0);
        decision.order_type = Some(OrderType::Limit);
        decision.limit_price = Some(140.0);
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("limit_price")
        );
    }

    #[test]
    fn blocks_wrong_side_stop() {
        let ctx = sample_context();
        let outcome = validate(buy(1.0, 135.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("below entry")
        );
    }

    #[test]
    fn derives_stop_when_missing() {
        let ctx = sample_context();
        let mut decision = buy(1.0, 0.0);
        decision.stop_loss = None;
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        let stop = outcome.decision.stop_loss.unwrap();
        assert!(stop < 129.6);
        assert!(
            outcome
                .report
                .adjustments
                .iter()
                .any(|a| a.contains("derived"))
        );
    }

    #[test]
    fn blocks_stale_quote_and_daily_loss() {
        let mut ctx = sample_context();
        ctx.market.stale = true;
        let outcome = validate(buy(1.0, 125.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);

        let mut ctx = sample_context();
        ctx.account.day_pnl = -4_000.0;
        let outcome = validate(buy(1.0, 125.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("daily loss")
        );
    }

    #[test]
    fn blocks_wide_spread() {
        let mut ctx = sample_context();
        ctx.market.ask = 131.0;
        let outcome = validate(buy(1.0, 125.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(outcome.report.blocked_reason.unwrap().contains("spread"));
    }

    #[test]
    fn close_flattens_position() {
        let mut ctx = sample_context();
        ctx.position = Some(PositionState {
            quantity: 2.0,
            average_price: 120.0,
            unrealized_pnl: 19.0,
        });
        let decision = TradeDecision {
            action: Action::Close,
            confidence: 0.7,
            quantity: Some(5.0),
            notional: None,
            order_type: Some(OrderType::Market),
            limit_price: None,
            stop_loss: Some(120.0),
            take_profit: None,
            time_horizon_minutes: None,
            rationale: "exit".to_string(),
            invalidation: None,
            risk_flags: Vec::new(),
        };
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        assert_eq!(outcome.decision.action, Action::Close);
        assert_eq!(outcome.decision.quantity, Some(2.0));
        assert!(outcome.decision.stop_loss.is_none());
    }

    #[test]
    fn close_without_position_is_blocked() {
        let ctx = sample_context();
        let decision = TradeDecision {
            action: Action::Close,
            confidence: 0.9,
            quantity: None,
            notional: None,
            order_type: None,
            limit_price: None,
            stop_loss: None,
            take_profit: None,
            time_horizon_minutes: None,
            rationale: "exit".to_string(),
            invalidation: None,
            risk_flags: Vec::new(),
        };
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("no position")
        );
    }

    #[test]
    fn hold_ignores_order_fields() {
        let ctx = sample_context();
        let decision = TradeDecision {
            action: Action::Hold,
            confidence: 0.55,
            quantity: Some(2.0),
            notional: None,
            order_type: Some(OrderType::Market),
            limit_price: None,
            stop_loss: Some(120.0),
            take_profit: None,
            time_horizon_minutes: None,
            rationale: "wait".to_string(),
            invalidation: None,
            risk_flags: Vec::new(),
        };
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        assert!(outcome.decision.quantity.is_none());
        assert!(outcome.decision.order_type.is_none());
    }

    #[test]
    fn normalizes_percent_confidence() {
        let ctx = sample_context();
        let mut decision = buy(1.0, 125.0);
        decision.confidence = 80.0;
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        assert!((outcome.decision.confidence - 0.8).abs() < 1e-9);
    }

    #[test]
    fn take_profit_must_be_on_profit_side() {
        let ctx = sample_context();
        let mut decision = buy(1.0, 125.0);
        decision.take_profit = Some(120.0);
        let outcome = validate(decision, &ctx, &features(&ctx), &ctx.risk);
        assert!(!outcome.report.approved);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("above entry")
        );
    }

    #[test]
    fn risk_budget_reduces_size() {
        let ctx = sample_context();
        let outcome = validate(buy(1_000.0, 129.0), &ctx, &features(&ctx), &ctx.risk);
        assert!(outcome.report.approved);
        let quantity = outcome.decision.quantity.unwrap();
        let risk = (129.6 - 129.0) * quantity;
        assert!(risk <= 1_000.0 + 1e-6);
    }
}
