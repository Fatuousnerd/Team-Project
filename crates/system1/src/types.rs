//! Domain inputs and outputs for the System 1 decision engine.
//!
//! The crate is deliberately independent from `crates/bot`: callers translate
//! their engine/portfolio types into these snapshots, and translate the
//! resulting [`TradeDecision`] back into an order. This keeps the model layer
//! testable and lets the desktop app reuse it outside the mock engine.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::features::MarketFeatures;

// ---------------------------------------------------------------------------
// Market inputs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarketKind {
    #[default]
    Stock,
    Forex,
    Crypto,
}

impl MarketKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stock => "stock",
            Self::Forex => "forex",
            Self::Crypto => "crypto",
        }
    }
}

/// One OHLCV bar. Candles are expected oldest → newest, in quote currency.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn is_valid(&self) -> bool {
        self.open.is_finite()
            && self.high.is_finite()
            && self.low.is_finite()
            && self.close.is_finite()
            && self.volume.is_finite()
            && self.low > 0.0
            && self.close > 0.0
            && self.high >= self.low
            && self.high >= self.open.max(self.close)
            && self.low <= self.open.min(self.close)
            && self.volume >= 0.0
    }

    /// True range, using `previous_close` when available.
    pub(crate) fn true_range(&self, previous_close: Option<f64>) -> f64 {
        let hl = (self.high - self.low).max(0.0);
        match previous_close {
            Some(prev) if prev > 0.0 => hl
                .max((self.high - prev).abs())
                .max((self.low - prev).abs()),
            _ => hl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OrderBookState {
    #[serde(default)]
    pub bids: Vec<BookLevel>,
    #[serde(default)]
    pub asks: Vec<BookLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    pub market_type: MarketKind,
    /// Whether the venue is currently accepting orders.
    #[serde(default = "default_true")]
    pub session_open: bool,
    /// Set by the caller when the quote/feed is older than its freshness budget.
    #[serde(default)]
    pub stale: bool,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    #[serde(default)]
    pub high_24h: Option<f64>,
    #[serde(default)]
    pub low_24h: Option<f64>,
    #[serde(default)]
    pub volume_24h: Option<f64>,
    #[serde(default)]
    pub change_pct_24h: Option<f64>,
    /// Oldest → newest. The engine ignores malformed bars and uses the last
    /// `max_candles_in_prompt` of them for the prompt.
    #[serde(default)]
    pub candles: Vec<Candle>,
    #[serde(default)]
    pub order_book: Option<OrderBookState>,
}

// ---------------------------------------------------------------------------
// Account, position, and activity inputs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub equity: f64,
    /// Free cash / buying power in quote currency.
    pub cash: f64,
    #[serde(default)]
    pub margin_used: f64,
    #[serde(default)]
    pub day_pnl: f64,
    #[serde(default)]
    pub day_pnl_pct: Option<f64>,
    #[serde(default)]
    pub open_positions: usize,
    /// Total absolute notional currently held, when the caller tracks it.
    #[serde(default)]
    pub gross_exposure: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PositionState {
    /// Signed quantity: positive long, negative short.
    #[serde(default)]
    pub quantity: f64,
    #[serde(default)]
    pub average_price: f64,
    #[serde(default)]
    pub unrealized_pnl: f64,
}

impl PositionState {
    pub fn is_long(&self) -> bool {
        self.quantity > 0.0
    }

    pub fn is_short(&self) -> bool {
        self.quantity < 0.0
    }

    pub fn abs_quantity(&self) -> f64 {
        self.quantity.abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

impl OrderType {
    pub fn is_protective(self) -> bool {
        matches!(self, Self::Stop | Self::StopLimit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrderState {
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    #[serde(default)]
    pub limit_price: Option<f64>,
    #[serde(default)]
    pub stop_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeState {
    pub side: Side,
    pub quantity: f64,
    pub price: f64,
    #[serde(default)]
    pub pnl: Option<f64>,
    /// Age of the fill in seconds, used by the model to judge recency.
    #[serde(default)]
    pub age_secs: Option<u64>,
}

// ---------------------------------------------------------------------------
// Risk policy
// ---------------------------------------------------------------------------

/// Hard limits applied after the model answers. All percentages are fractions
/// of account equity (`0.05` = 5 %), not percent units.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RiskPolicy {
    /// Maximum notional for a single new order.
    pub max_order_notional_pct: f64,
    /// Maximum absolute position notional after a fill.
    pub max_position_notional_pct: f64,
    /// Maximum absolute gross exposure across the account.
    pub max_gross_exposure_pct: f64,
    /// Maximum loss from entry to protective stop.
    pub max_risk_per_trade_pct: f64,
    /// Stop opening new risk after this much intraday loss.
    pub max_daily_loss_pct: f64,
    /// Minimum model confidence required to act.
    pub min_confidence: f64,
    /// Wide spreads are a no-trade condition.
    pub max_spread_bps: f64,
    /// Furthest a limit price may sit from the mid, in basis points.
    pub max_limit_deviation_bps: f64,
    /// Round-trip cost assumption shown to the model (basis points).
    pub fee_bps: f64,
    /// Extra edge required before a trade is worth taking (basis points).
    pub min_edge_bps: f64,
    pub allow_short: bool,
    pub allow_margin: bool,
    pub require_stop_loss: bool,
    pub min_stop_distance_bps: f64,
    pub max_stop_distance_bps: f64,
    /// ATR multiple used to derive a stop when the model omits one.
    pub default_stop_atr_multiple: f64,
    /// Fallback stop distance when ATR is unavailable.
    pub default_stop_pct: f64,
    /// Warn when take-profit reward/risk is below this.
    pub min_reward_risk: f64,
    /// Entries require at least this many valid candles.
    pub min_candles: usize,
    /// Refuse new risk when the quote is marked stale.
    pub require_fresh_quote: bool,
    /// Quantity increment; `0` disables rounding.
    pub quantity_step: f64,
    /// Minimum executable quantity; `0` disables the check.
    pub min_quantity: f64,
    /// Optional absolute quantity cap.
    pub max_quantity: Option<f64>,
    /// Price increment for limit/stop/target levels; `0` disables rounding.
    pub price_tick: f64,
    /// Minimum time between entries for the same symbol.
    pub min_seconds_between_trades: u64,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            max_order_notional_pct: 0.05,
            max_position_notional_pct: 0.20,
            max_gross_exposure_pct: 1.0,
            max_risk_per_trade_pct: 0.01,
            max_daily_loss_pct: 0.03,
            min_confidence: 0.60,
            max_spread_bps: 20.0,
            max_limit_deviation_bps: 75.0,
            fee_bps: 2.0,
            min_edge_bps: 8.0,
            allow_short: false,
            allow_margin: false,
            require_stop_loss: true,
            min_stop_distance_bps: 15.0,
            max_stop_distance_bps: 1_200.0,
            default_stop_atr_multiple: 1.5,
            default_stop_pct: 0.015,
            min_reward_risk: 1.2,
            min_candles: 20,
            require_fresh_quote: true,
            quantity_step: 0.0,
            min_quantity: 0.0,
            max_quantity: None,
            price_tick: 0.0,
            min_seconds_between_trades: 30,
        }
    }
}

impl RiskPolicy {
    /// Clamp malformed caller input into a safe, non-panicking policy.
    pub fn normalized(&self) -> Self {
        let pct = |v: f64, fallback: f64| {
            if v.is_finite() {
                v.clamp(0.0, 10.0)
            } else {
                fallback
            }
        };
        let non_negative = |v: f64, fallback: f64| {
            if v.is_finite() { v.max(0.0) } else { fallback }
        };

        let mut min_stop = non_negative(self.min_stop_distance_bps, 15.0);
        let mut max_stop = non_negative(self.max_stop_distance_bps, 1_200.0);
        if max_stop < min_stop {
            std::mem::swap(&mut min_stop, &mut max_stop);
        }

        Self {
            max_order_notional_pct: pct(self.max_order_notional_pct, 0.05),
            max_position_notional_pct: pct(self.max_position_notional_pct, 0.20),
            max_gross_exposure_pct: pct(self.max_gross_exposure_pct, 1.0),
            max_risk_per_trade_pct: pct(self.max_risk_per_trade_pct, 0.01),
            max_daily_loss_pct: pct(self.max_daily_loss_pct, 0.03),
            min_confidence: pct(self.min_confidence, 0.60).min(1.0),
            max_spread_bps: non_negative(self.max_spread_bps, 20.0),
            max_limit_deviation_bps: non_negative(self.max_limit_deviation_bps, 75.0),
            fee_bps: non_negative(self.fee_bps, 2.0),
            min_edge_bps: non_negative(self.min_edge_bps, 8.0),
            allow_short: self.allow_short,
            allow_margin: self.allow_margin,
            require_stop_loss: self.require_stop_loss,
            min_stop_distance_bps: min_stop,
            max_stop_distance_bps: max_stop,
            default_stop_atr_multiple: {
                let v = non_negative(self.default_stop_atr_multiple, 1.5);
                if v == 0.0 { 1.5 } else { v.min(25.0) }
            },
            default_stop_pct: {
                let v = non_negative(self.default_stop_pct, 0.015);
                if v == 0.0 { 0.015 } else { v.min(0.50) }
            },
            min_reward_risk: non_negative(self.min_reward_risk, 1.2),
            min_candles: self.min_candles.min(1_000),
            require_fresh_quote: self.require_fresh_quote,
            quantity_step: non_negative(self.quantity_step, 0.0),
            min_quantity: non_negative(self.min_quantity, 0.0),
            max_quantity: self.max_quantity.filter(|q| q.is_finite() && *q > 0.0),
            price_tick: non_negative(self.price_tick, 0.0),
            min_seconds_between_trades: self.min_seconds_between_trades,
        }
    }
}

// ---------------------------------------------------------------------------
// Decision context
// ---------------------------------------------------------------------------

/// Everything the model is allowed to reason about, plus the guardrails that
/// are enforced after it answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingContext {
    /// RFC 3339 timestamp supplied by the caller.
    pub as_of: String,
    pub symbol: String,
    #[serde(default)]
    pub strategy: Option<String>,
    /// Chart timeframe, e.g. `"1m"`, `"5m"`, `"1h"`.
    #[serde(default)]
    pub timeframe: Option<String>,
    pub market: MarketState,
    pub account: AccountState,
    #[serde(default)]
    pub position: Option<PositionState>,
    #[serde(default)]
    pub open_orders: Vec<OpenOrderState>,
    #[serde(default)]
    pub recent_trades: Vec<TradeState>,
    /// How long ago the last fill on this symbol occurred.
    #[serde(default)]
    pub seconds_since_last_trade: Option<u64>,
    #[serde(default)]
    pub risk: RiskPolicy,
    /// Free-form caller notes (session news, maintenance, strategy state).
    #[serde(default)]
    pub notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Decision output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Buy,
    Sell,
    #[default]
    Hold,
    Close,
}

impl Action {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
            Self::Hold => "hold",
            Self::Close => "close",
        }
    }

    /// True for actions that would add or flip exposure.
    pub fn opens_risk(self) -> bool {
        matches!(self, Self::Buy | Self::Sell)
    }
}

/// The model's raw proposal. Every optional field is permissively parsed so a
/// slightly sloppy 4B model response still reaches validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDecision {
    #[serde(default, deserialize_with = "de_action")]
    pub action: Action,
    #[serde(default, deserialize_with = "de_f64")]
    pub confidence: f64,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub quantity: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub notional: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_order_type")]
    pub order_type: Option<OrderType>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub limit_price: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub stop_loss: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub take_profit: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_u32")]
    pub time_horizon_minutes: Option<u32>,
    #[serde(default)]
    pub rationale: String,
    #[serde(default)]
    pub invalidation: Option<String>,
    #[serde(default)]
    pub risk_flags: Vec<String>,
}

impl TradeDecision {
    pub fn hold(reason: impl Into<String>) -> Self {
        Self {
            action: Action::Hold,
            confidence: 0.0,
            quantity: None,
            notional: None,
            order_type: None,
            limit_price: None,
            stop_loss: None,
            take_profit: None,
            time_horizon_minutes: None,
            rationale: reason.into(),
            invalidation: None,
            risk_flags: Vec::new(),
        }
    }

    /// Drop order fields that do not apply once a decision is forced to hold.
    pub(crate) fn clear_order_fields(&mut self) {
        self.quantity = None;
        self.notional = None;
        self.order_type = None;
        self.limit_price = None;
        self.stop_loss = None;
        self.take_profit = None;
        self.time_horizon_minutes = None;
    }

    pub fn is_hold(&self) -> bool {
        self.action == Action::Hold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub approved: bool,
    #[serde(default)]
    pub adjustments: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blocked_reason: Option<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self {
            approved: true,
            adjustments: Vec::new(),
            warnings: Vec::new(),
            blocked_reason: None,
        }
    }
}

impl ValidationReport {
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self {
            approved: false,
            adjustments: Vec::new(),
            warnings: Vec::new(),
            blocked_reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionSource {
    /// Model output passed validation unchanged.
    Model,
    /// Model output passed after deterministic risk adjustments.
    ModelAdjusted,
    /// Model wanted to act, but a guardrail forced a hold.
    GuardrailHold,
    /// No usable model response; the engine returned a defensive hold.
    FallbackHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    /// The executable decision after guardrails.
    pub decision: TradeDecision,
    /// The parsed model proposal before guardrails, when one was obtained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal: Option<TradeDecision>,
    pub features: MarketFeatures,
    pub report: ValidationReport,
    pub source: DecisionSource,
    pub model: String,
    pub latency_ms: u64,
    pub attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Permissive deserializers
// ---------------------------------------------------------------------------

fn de_action<'de, D>(deserializer: D) -> Result<Action, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "buy" | "long" | "enter_long" => Ok(Action::Buy),
        "sell" | "short" | "enter_short" => Ok(Action::Sell),
        "hold" | "wait" | "none" | "no_trade" => Ok(Action::Hold),
        "close" | "exit" | "flatten" | "close_position" => Ok(Action::Close),
        other => Err(serde::de::Error::custom(format!(
            "unknown action `{other}`; expected buy, sell, hold, or close"
        ))),
    }
}

fn de_opt_order_type<'de, D>(deserializer: D) -> Result<Option<OrderType>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let normalized = raw.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    let order_type = match normalized.as_str() {
        "market" | "mkt" => OrderType::Market,
        "limit" | "lmt" => OrderType::Limit,
        "stop" | "stop_market" => OrderType::Stop,
        "stop_limit" | "stoplimit" => OrderType::StopLimit,
        "" => return Ok(None),
        other => {
            return Err(serde::de::Error::custom(format!(
                "unknown order_type `{other}`"
            )));
        }
    };
    Ok(Some(order_type))
}

fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    value_to_f64(&value)
        .map_err(serde::de::Error::custom)
        .map(|v| v.unwrap_or(0.0))
}

fn de_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    value_to_f64(&value).map_err(serde::de::Error::custom)
}

fn value_to_f64(value: &Value) -> Result<Option<f64>, String> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_f64()
            .filter(|v| v.is_finite())
            .map(Some)
            .ok_or_else(|| "number is not finite".to_string()),
        Value::String(text) => {
            let trimmed = text.trim();
            let is_percent = trimmed.ends_with('%');
            let numeric = trimmed.trim_end_matches('%').trim();
            if numeric.is_empty() {
                return Ok(None);
            }
            numeric
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(|v| if is_percent { v / 100.0 } else { v })
                .map(Some)
                .ok_or_else(|| format!("`{trimmed}` is not a number"))
        }
        _ => Err(format!("expected a number, got {value}")),
    }
}

fn de_opt_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("time horizon is out of range")),
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.is_empty() {
                Ok(None)
            } else {
                text.parse::<u32>()
                    .map(Some)
                    .map_err(|_| serde::de::Error::custom("time horizon is not an integer"))
            }
        }
        Some(other) => Err(serde::de::Error::custom(format!(
            "invalid time horizon: {other}"
        ))),
    }
}

fn default_true() -> bool {
    true
}
