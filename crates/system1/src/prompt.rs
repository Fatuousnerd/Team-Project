//! Prompt construction.
//!
//! The system prompt holds the policy; the user prompt holds a compact,
//! point-in-time JSON snapshot. Keeping the schema and the rules in one place
//! makes the model's job narrow and the output easy to validate.

use serde_json::{Value, json};

use crate::chat::ChatMessage;
use crate::features::MarketFeatures;
use crate::types::{Candle, RiskPolicy, TradingContext};

pub const SYSTEM_PROMPT: &str = r#"You are System 1, a fast, disciplined trading analyst for a paper-trading desk.
Choose exactly one action for one symbol from a complete point-in-time snapshot.

ACTIONS
- buy: open/add a long, or cover a short.
- sell: open/add a short, or reduce a long.
- close: flatten/reduce the existing position. Never open new risk.
- hold: do nothing.

HARD RULES
1. Use only the supplied numbers. Never invent prices, news, fundamentals, or indicators.
2. Hold when the quote is stale, the session is closed, candles are insufficient, spread exceeds max_spread_bps, or the daily loss limit is reached. `close` may still reduce risk.
3. Prefer hold. Act only when the setup's expected edge clearly exceeds spread + fees + min_edge_bps.
4. For buy or sell, provide quantity OR notional, a stop_loss, and confidence.
5. confidence is the probability (0-1) that the trade is net profitable after costs. Below min_confidence is rejected.
6. Size so the loss from entry to stop is at most max_risk_per_trade_pct of equity. Never exceed max_order_notional_pct or max_position_notional_pct.
7. Shorting, or selling more than an existing long, requires allow_short=true.
8. A limit price must be within max_limit_deviation_bps of mid; otherwise choose market.
9. `close` must not open a new position; its quantity is capped to the current position.
10. Do not chase parabolic moves, trade into a wide spread, or fight a strong trend without a clear reversal signal.
11. Do not stack a new entry when an open order already expresses the same idea.
12. If the setup is unclear, hold with low confidence and state what is missing.

DECISION PROCEDURE
1. Check data quality, session, spread, daily loss, and cooldown.
2. Read the regime: trend (EMA12 vs EMA26, price vs SMA20/50), momentum (RSI14, returns), volatility (ATR%, realized vol), and location (range position, support/resistance).
3. Compare continuation vs mean reversion. Require room to a sensible stop and at least min_reward_risk to a target.
4. Choose hold unless the edge is clear.
5. Size from the stop distance and the risk budget, then round to quantity_step.

OUTPUT
Return ONLY one JSON object matching the supplied schema. No markdown, prose, or trailing commas.
Prices, quantities, and confidence must be numbers, not strings."#;

#[derive(Debug, Clone)]
pub struct PromptOptions {
    /// Most recent candles included as raw OHLCV rows.
    pub max_candles: usize,
    /// Levels per side of the order book included in the prompt.
    pub max_book_levels: usize,
    /// Most recent fills included in the prompt.
    pub max_recent_trades: usize,
}

impl Default for PromptOptions {
    fn default() -> Self {
        Self {
            max_candles: 40,
            max_book_levels: 8,
            max_recent_trades: 5,
        }
    }
}

/// JSON schema passed to Ollama's structured-output `format` field.
pub fn decision_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["buy", "sell", "hold", "close"]
            },
            "confidence": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0
            },
            "quantity": {
                "type": "number",
                "exclusiveMinimum": 0.0
            },
            "notional": {
                "type": "number",
                "exclusiveMinimum": 0.0
            },
            "order_type": {
                "type": "string",
                "enum": ["market", "limit"]
            },
            "limit_price": {
                "type": "number",
                "exclusiveMinimum": 0.0
            },
            "stop_loss": {
                "type": "number",
                "exclusiveMinimum": 0.0
            },
            "take_profit": {
                "type": "number",
                "exclusiveMinimum": 0.0
            },
            "time_horizon_minutes": {
                "type": "integer",
                "minimum": 1
            },
            "rationale": {
                "type": "string"
            },
            "invalidation": {
                "type": "string"
            },
            "risk_flags": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["action", "confidence", "rationale"],
        "additionalProperties": false
    })
}

pub fn build_messages(
    ctx: &TradingContext,
    features: &MarketFeatures,
    risk: &RiskPolicy,
    options: &PromptOptions,
) -> Vec<ChatMessage> {
    let payload = user_payload(ctx, features, risk, options);
    let content = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    vec![
        ChatMessage::system(SYSTEM_PROMPT),
        ChatMessage::user(content),
    ]
}

fn user_payload(
    ctx: &TradingContext,
    features: &MarketFeatures,
    risk: &RiskPolicy,
    options: &PromptOptions,
) -> Value {
    let valid_candles: Vec<Candle> = ctx
        .market
        .candles
        .iter()
        .copied()
        .filter(Candle::is_valid)
        .collect();
    let candle_rows = candle_rows(&valid_candles, options.max_candles);
    let recent_trades: Vec<_> = ctx
        .recent_trades
        .iter()
        .rev()
        .take(options.max_recent_trades)
        .collect();

    json!({
        "task": "Choose one action for this symbol now.",
        "as_of": ctx.as_of,
        "symbol": ctx.symbol,
        "strategy": ctx.strategy,
        "timeframe": ctx.timeframe,
        "market": {
            "type": ctx.market.market_type.as_str(),
            "session_open": ctx.market.session_open,
            "stale": ctx.market.stale,
            "quote_valid": features.quote_valid,
            "bid": round(ctx.market.bid, 6),
            "ask": round(ctx.market.ask, 6),
            "last": round(ctx.market.last, 6),
            "mid": round(features.mid, 6),
            "spread_bps": round(features.spread_bps, 2),
            "high_24h": round_opt(ctx.market.high_24h),
            "low_24h": round_opt(ctx.market.low_24h),
            "volume_24h": round_opt(ctx.market.volume_24h),
            "change_pct_24h": round_opt(ctx.market.change_pct_24h),
        },
        "features": features,
        "recent_candles": {
            "note": "oldest to newest; each row is [open, high, low, close, volume]; prices in quote currency",
            "count": candle_rows.len(),
            "rows": candle_rows,
        },
        "order_book": book_payload(ctx, options.max_book_levels),
        "account": {
            "equity": round(ctx.account.equity, 2),
            "cash": round(ctx.account.cash, 2),
            "margin_used": round(ctx.account.margin_used, 2),
            "day_pnl": round(ctx.account.day_pnl, 2),
            "day_pnl_pct": round_opt(ctx.account.day_pnl_pct),
            "open_positions": ctx.account.open_positions,
            "gross_exposure": round_opt(ctx.account.gross_exposure),
        },
        "position": ctx.position.map(|position| json!({
            "side": if position.quantity > 0.0 { "long" } else if position.quantity < 0.0 { "short" } else { "flat" },
            "quantity": round(position.quantity, 8),
            "average_price": round(position.average_price, 6),
            "unrealized_pnl": round(position.unrealized_pnl, 2),
        })),
        "open_orders": ctx.open_orders.iter().map(|order| json!({
            "side": order.side.as_str(),
            "type": order.order_type,
            "quantity": round(order.quantity, 8),
            "limit_price": round_opt(order.limit_price),
            "stop_price": round_opt(order.stop_price),
        })).collect::<Vec<_>>(),
        "recent_trades": recent_trades,
        "seconds_since_last_trade": ctx.seconds_since_last_trade,
        "risk_limits": risk,
        "costs": {
            "spread_bps": round(features.spread_bps, 2),
            "fee_bps": round(risk.fee_bps, 2),
            "min_edge_bps": round(risk.min_edge_bps, 2),
            "target_round_trip_cost_bps": round(features.spread_bps + risk.fee_bps + risk.min_edge_bps, 2),
        },
        "notes": ctx.notes,
        "response_requirements": [
            "Return one JSON object only.",
            "buy/sell require quantity or notional, stop_loss, and a limit_price when order_type=limit.",
            "hold needs no order fields. close reduces the current position.",
            "Use confidence as a calibrated probability, not a label."
        ]
    })
}

fn book_payload(ctx: &TradingContext, max_levels: usize) -> Value {
    let Some(book) = ctx.market.order_book.as_ref() else {
        return Value::Null;
    };
    if max_levels == 0 {
        return Value::Null;
    }

    let mut bids: Vec<_> = book
        .bids
        .iter()
        .filter(|level| {
            level.price.is_finite()
                && level.size.is_finite()
                && level.price > 0.0
                && level.size > 0.0
        })
        .collect();
    bids.sort_by(|a, b| b.price.total_cmp(&a.price));

    let mut asks: Vec<_> = book
        .asks
        .iter()
        .filter(|level| {
            level.price.is_finite()
                && level.size.is_finite()
                && level.price > 0.0
                && level.size > 0.0
        })
        .collect();
    asks.sort_by(|a, b| a.price.total_cmp(&b.price));

    json!({
        "bids": bids.iter().take(max_levels).map(|level| json!([round(level.price, 6), round(level.size, 8)])).collect::<Vec<_>>(),
        "asks": asks.iter().take(max_levels).map(|level| json!([round(level.price, 6), round(level.size, 8)])).collect::<Vec<_>>(),
    })
}

fn candle_rows(candles: &[Candle], max_candles: usize) -> Vec<Vec<f64>> {
    if candles.is_empty() || max_candles == 0 {
        return Vec::new();
    }
    let start = candles.len().saturating_sub(max_candles);
    candles[start..]
        .iter()
        .map(|candle| {
            vec![
                round(candle.open, 6),
                round(candle.high, 6),
                round(candle.low, 6),
                round(candle.close, 6),
                round(candle.volume, 4),
            ]
        })
        .collect()
}

fn round(value: f64, decimals: i32) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    let factor = 10f64.powi(decimals);
    (value * factor).round() / factor
}

fn round_opt(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite()).map(|v| round(v, 6))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::MarketFeatures;
    use crate::types::{AccountState, Candle, MarketKind, MarketState, RiskPolicy, TradingContext};

    fn sample() -> TradingContext {
        let candles = (0..60)
            .map(|i| Candle {
                open: 100.0 + i as f64,
                high: 101.0 + i as f64,
                low: 99.0 + i as f64,
                close: 100.5 + i as f64,
                volume: 1_000.0 + i as f64,
            })
            .collect();
        TradingContext {
            as_of: "2026-11-18T12:00:00Z".to_string(),
            symbol: "BTCUSDT".to_string(),
            strategy: Some("momentum".to_string()),
            timeframe: Some("5m".to_string()),
            market: MarketState {
                market_type: MarketKind::Crypto,
                session_open: true,
                stale: false,
                bid: 159.4,
                ask: 159.6,
                last: 159.5,
                high_24h: Some(165.0),
                low_24h: Some(140.0),
                volume_24h: Some(1_000_000.0),
                change_pct_24h: Some(3.2),
                candles,
                order_book: None,
            },
            account: AccountState {
                equity: 100_000.0,
                cash: 80_000.0,
                margin_used: 0.0,
                day_pnl: 250.0,
                day_pnl_pct: Some(0.25),
                open_positions: 1,
                gross_exposure: Some(20_000.0),
            },
            position: None,
            open_orders: Vec::new(),
            recent_trades: Vec::new(),
            seconds_since_last_trade: None,
            risk: RiskPolicy::default(),
            notes: vec!["sample note".to_string()],
        }
    }

    #[test]
    fn schema_is_closed_and_requires_action() {
        let schema = decision_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"][0], "action");
        assert_eq!(schema["properties"]["action"]["enum"][0], "buy");
    }

    #[test]
    fn prompt_contains_state_and_rules() {
        let ctx = sample();
        let features = MarketFeatures::compute(&ctx.market);
        let risk = ctx.risk.normalized();
        let messages = build_messages(&ctx, &features, &risk, &PromptOptions::default());

        assert_eq!(messages.len(), 2);
        assert!(messages[0].content.contains("System 1"));
        assert!(messages[1].content.contains("BTCUSDT"));
        assert!(messages[1].content.contains("rsi_14"));
        assert!(messages[1].content.contains("max_spread_bps"));
        assert!(messages[1].content.contains("sample note"));

        let payload: Value = serde_json::from_str(&messages[1].content).unwrap();
        assert_eq!(payload["symbol"], "BTCUSDT");
        assert_eq!(payload["recent_candles"]["count"], 40);
        assert_eq!(payload["market"]["type"], "crypto");
    }
}
