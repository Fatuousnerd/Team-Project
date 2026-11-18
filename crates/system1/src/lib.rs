//! System 1 — an Ollama-backed trading decision layer.
//!
//! The crate is intentionally separate from the mock engine in `crates/bot`.
//! Callers build a [`TradingContext`] from their live market/account state, ask
//! a locally served model for a decision, and receive an executable
//! [`TradeDecision`] after deterministic guardrails have been applied.
//!
//! # Quick start
//!
//! ```text
//! let system = System1::from_env()?;           // SYSTEM1_MODEL defaults to tev1:4b
//! system.ensure_model().await?;                // clear error if `ollama pull` is needed
//! let outcome = system.decide(&context).await?;
//! match outcome.decision.action {
//!     Action::Buy | Action::Sell | Action::Close => submit(outcome.decision),
//!     Action::Hold => log(outcome.decision.rationale),
//! }
//! ```
//!
//! # Safety model
//!
//! 1. The prompt contains only the supplied snapshot and computed indicators.
//! 2. Ollama is asked for structured JSON; parsing tolerates fences/trailing
//!    commas, and a bounded repair call is made if JSON is malformed.
//! 3. [`validate`] enforces spread, freshness, confidence, cooldown, shorting,
//!    notional, position, gross-exposure, stop-distance, and risk-budget limits.
//! 4. Any unrepairable output or transport failure becomes a defensive hold
//!    when using [`System1::decide_or_hold`].
//!
//! Percentages in [`RiskPolicy`] are fractions of equity (`0.05` = 5 %).

#![forbid(unsafe_code)]

pub mod chat;
pub mod engine;
pub mod error;
pub mod features;
pub mod ollama;
pub mod parse;
pub mod prompt;
pub mod types;
pub mod validate;

pub use chat::{ChatBackend, ChatMessage, ChatResponse};
pub use engine::{System1, System1Config};
pub use error::Error;
pub use features::{MarketFeatures, Trend};
pub use ollama::{DEFAULT_BASE_URL, DEFAULT_MODEL, Health, OllamaClient, OllamaConfig};
pub use parse::parse_decision;
pub use prompt::{PromptOptions, SYSTEM_PROMPT, decision_schema};
pub use types::{
    AccountState, Action, BookLevel, Candle, DecisionOutcome, DecisionSource, MarketKind,
    MarketState, OpenOrderState, OrderBookState, OrderType, PositionState, RiskPolicy, Side,
    TradeDecision, TradeState, TradingContext, ValidationReport,
};
pub use validate::{ValidatedDecision, validate};
