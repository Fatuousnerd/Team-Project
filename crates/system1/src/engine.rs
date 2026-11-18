//! Decision orchestration: prompt → Ollama → parse → guardrails.
//!
//! Transport failures are returned as errors so the caller can decide whether
//! to retry, skip the tick, or flatten risk. Malformed model output is treated
//! as a defensive hold because the model did respond but cannot be trusted.

use std::time::{Duration, Instant};

use crate::chat::{ChatBackend, ChatMessage, ChatResponse};
use crate::error::Error;
use crate::features::MarketFeatures;
use crate::ollama::OllamaClient;
use crate::parse::parse_decision;
use crate::prompt::{PromptOptions, build_messages, decision_schema};
use crate::types::{
    DecisionOutcome, DecisionSource, TradeDecision, TradingContext, ValidationReport,
};
use crate::validate::validate;

#[derive(Debug, Clone)]
pub struct System1Config {
    /// Extra attempts for transient transport/server errors.
    pub retries: u32,
    /// Extra calls asking the model to repair malformed JSON.
    pub repair_attempts: u32,
    /// Base backoff between transport retries.
    pub retry_backoff: Duration,
    /// Upper bound on exponential backoff.
    pub max_backoff: Duration,
    pub prompt: PromptOptions,
}

impl Default for System1Config {
    fn default() -> Self {
        Self {
            retries: 2,
            repair_attempts: 1,
            retry_backoff: Duration::from_millis(250),
            max_backoff: Duration::from_secs(2),
            prompt: PromptOptions::default(),
        }
    }
}

/// The reusable decision service. `B` is normally [`OllamaClient`]; tests inject
/// a fake backend to exercise retries, repair, and guardrails without a server.
pub struct System1<B: ChatBackend = OllamaClient> {
    backend: B,
    config: System1Config,
}

impl<B: ChatBackend> System1<B> {
    pub fn new(backend: B, config: System1Config) -> Self {
        Self { backend, config }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn model(&self) -> &str {
        self.backend.model()
    }

    /// Ask the model for a decision and apply deterministic guardrails.
    ///
    /// Returns `Err` only when the backend could not produce a response (after
    /// transport retries). Invalid JSON becomes a [`DecisionSource::FallbackHold`]
    /// outcome so an automated loop can keep running safely.
    pub async fn decide(&self, ctx: &TradingContext) -> Result<DecisionOutcome, Error> {
        let started = Instant::now();
        let risk = ctx.risk.normalized();
        let features = MarketFeatures::compute(&ctx.market);
        let messages = build_messages(ctx, &features, &risk, &self.config.prompt);
        let schema = decision_schema();
        let mut attempts = 0u32;

        let response = self
            .chat_with_retries(messages.clone(), Some(schema.clone()), &mut attempts)
            .await?;

        let mut content = response.content;
        match parse_decision(&content) {
            Ok(proposal) => Ok(self.finalize(ctx, features, &risk, proposal, started, attempts)),
            Err(first_error) => {
                if self.config.repair_attempts == 0 {
                    return Ok(self.fallback(
                        ctx,
                        features,
                        started,
                        attempts,
                        format!("model output was invalid: {first_error}"),
                    ));
                }

                let mut repair_messages = messages;
                repair_messages.push(ChatMessage::assistant(content.clone()));
                repair_messages.push(ChatMessage::user(repair_prompt(&first_error.to_string())));

                let mut last_error = first_error.to_string();
                for _ in 0..self.config.repair_attempts {
                    let response = self
                        .chat_with_retries(
                            repair_messages.clone(),
                            Some(schema.clone()),
                            &mut attempts,
                        )
                        .await?;
                    content = response.content;

                    match parse_decision(&content) {
                        Ok(proposal) => {
                            return Ok(
                                self.finalize(ctx, features, &risk, proposal, started, attempts)
                            );
                        }
                        Err(error) => {
                            last_error = error.to_string();
                            repair_messages.push(ChatMessage::assistant(content.clone()));
                            repair_messages.push(ChatMessage::user(repair_prompt(&last_error)));
                        }
                    }
                }

                Ok(self.fallback(
                    ctx,
                    features,
                    started,
                    attempts,
                    format!("model output was invalid after repair: {last_error}"),
                ))
            }
        }
    }

    /// Convenience for trading loops: never fails, returns a defensive hold if
    /// Ollama is unavailable.
    pub async fn decide_or_hold(&self, ctx: &TradingContext) -> DecisionOutcome {
        match self.decide(ctx).await {
            Ok(outcome) => outcome,
            Err(error) => {
                let features = MarketFeatures::compute(&ctx.market);
                let reason = format!("model unavailable: {error}");
                DecisionOutcome {
                    decision: TradeDecision::hold(format!("No trade: {reason}")),
                    proposal: None,
                    features,
                    report: ValidationReport::blocked(reason.clone()),
                    source: DecisionSource::FallbackHold,
                    model: self.backend.model().to_string(),
                    latency_ms: 0,
                    attempts: 0,
                    error: Some(reason),
                }
            }
        }
    }

    fn finalize(
        &self,
        ctx: &TradingContext,
        features: MarketFeatures,
        risk: &crate::types::RiskPolicy,
        proposal: TradeDecision,
        started: Instant,
        attempts: u32,
    ) -> DecisionOutcome {
        let validated = validate(proposal.clone(), ctx, &features, risk);
        let source = if validated.report.blocked_reason.is_some() {
            DecisionSource::GuardrailHold
        } else if validated.report.adjustments.is_empty() {
            DecisionSource::Model
        } else {
            DecisionSource::ModelAdjusted
        };

        DecisionOutcome {
            decision: validated.decision,
            proposal: Some(proposal),
            features,
            report: validated.report,
            source,
            model: self.backend.model().to_string(),
            latency_ms: elapsed_ms(started),
            attempts,
            error: None,
        }
    }

    fn fallback(
        &self,
        _ctx: &TradingContext,
        features: MarketFeatures,
        started: Instant,
        attempts: u32,
        reason: String,
    ) -> DecisionOutcome {
        DecisionOutcome {
            decision: TradeDecision::hold(format!("No trade: {reason}")),
            proposal: None,
            features,
            report: ValidationReport::blocked(reason.clone()),
            source: DecisionSource::FallbackHold,
            model: self.backend.model().to_string(),
            latency_ms: elapsed_ms(started),
            attempts,
            error: Some(reason),
        }
    }

    async fn chat_with_retries(
        &self,
        messages: Vec<ChatMessage>,
        schema: Option<serde_json::Value>,
        attempts: &mut u32,
    ) -> Result<ChatResponse, Error> {
        let mut last_error = None;

        for attempt in 0..=self.config.retries {
            *attempts += 1;
            match self.backend.chat(messages.clone(), schema.clone()).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let can_retry = error.is_retryable() && attempt < self.config.retries;
                    if !can_retry {
                        return Err(error);
                    }
                    let backoff = self
                        .config
                        .retry_backoff
                        .saturating_mul(1u32 << attempt.min(6))
                        .min(self.config.max_backoff);
                    tracing::warn!(
                        model = %self.backend.model(),
                        attempt = attempt + 1,
                        backoff_ms = backoff.as_millis(),
                        error = %error,
                        "transient Ollama error; retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::InvalidResponse("retry loop exhausted without a response".to_string())
        }))
    }
}

impl System1<OllamaClient> {
    pub fn from_env() -> Result<Self, Error> {
        let client = OllamaClient::from_env()?;
        Ok(Self::new(client, System1Config::default()))
    }

    pub fn ollama(config: crate::ollama::OllamaConfig) -> Result<Self, Error> {
        let client = OllamaClient::new(config)?;
        Ok(Self::new(client, System1Config::default()))
    }

    pub async fn health(&self) -> Result<crate::ollama::Health, Error> {
        self.backend.health().await
    }

    pub async fn ensure_model(&self) -> Result<crate::ollama::Health, Error> {
        self.backend.ensure_model().await
    }
}

fn repair_prompt(error: &str) -> String {
    format!(
        "Your previous response could not be parsed: {error}\n\
         Return exactly one JSON object matching the schema. \
         No markdown fences, no prose, no trailing commas."
    )
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use serde_json::Value;

    use super::*;
    use crate::types::{
        AccountState, Action, Candle, DecisionSource, MarketKind, MarketState, RiskPolicy,
        TradingContext,
    };

    enum FakeReply {
        Content(&'static str),
        Error(Error),
    }

    struct FakeBackend {
        model: String,
        replies: Mutex<VecDeque<FakeReply>>,
        calls: Mutex<usize>,
    }

    impl FakeBackend {
        fn new(replies: Vec<FakeReply>) -> Self {
            Self {
                model: "fake:test".to_string(),
                replies: Mutex::new(replies.into()),
                calls: Mutex::new(0),
            }
        }

        fn calls(&self) -> usize {
            *self.calls.lock().unwrap()
        }
    }

    #[async_trait]
    impl ChatBackend for FakeBackend {
        fn model(&self) -> &str {
            &self.model
        }

        async fn chat(
            &self,
            _messages: Vec<ChatMessage>,
            _format: Option<Value>,
        ) -> Result<ChatResponse, Error> {
            *self.calls.lock().unwrap() += 1;
            let reply = self.replies.lock().unwrap().pop_front();
            match reply {
                Some(FakeReply::Content(content)) => Ok(ChatResponse {
                    content: content.to_string(),
                    ..ChatResponse::default()
                }),
                Some(FakeReply::Error(error)) => Err(error),
                None => Err(Error::InvalidResponse("no fake reply left".to_string())),
            }
        }
    }

    fn context() -> TradingContext {
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

    fn config() -> System1Config {
        System1Config {
            retries: 0,
            repair_attempts: 0,
            retry_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(2),
            ..System1Config::default()
        }
    }

    const BUY: &str = r#"{"action":"buy","confidence":0.8,"quantity":1,"order_type":"market","stop_loss":125,"rationale":"trend"}"#;

    #[tokio::test]
    async fn returns_model_decision() {
        let system = System1::new(FakeBackend::new(vec![FakeReply::Content(BUY)]), config());
        let outcome = system.decide(&context()).await.unwrap();
        assert_eq!(outcome.decision.action, Action::Buy);
        assert_eq!(outcome.source, DecisionSource::Model);
        assert_eq!(outcome.attempts, 1);
        assert!(outcome.proposal.is_some());
    }

    #[tokio::test]
    async fn repairs_invalid_json() {
        let system = System1::new(
            FakeBackend::new(vec![
                FakeReply::Content("not json"),
                FakeReply::Content(BUY),
            ]),
            System1Config {
                repair_attempts: 1,
                ..config()
            },
        );
        let outcome = system.decide(&context()).await.unwrap();
        assert_eq!(outcome.decision.action, Action::Buy);
        assert_eq!(outcome.attempts, 2);
        assert!(outcome.error.is_none());
    }

    #[tokio::test]
    async fn falls_back_after_unrepairable_output() {
        let system = System1::new(
            FakeBackend::new(vec![FakeReply::Content("still not json")]),
            config(),
        );
        let outcome = system.decide(&context()).await.unwrap();
        assert_eq!(outcome.decision.action, Action::Hold);
        assert_eq!(outcome.source, DecisionSource::FallbackHold);
        assert!(outcome.error.is_some());
    }

    #[tokio::test]
    async fn retries_transient_errors() {
        let system = System1::new(
            FakeBackend::new(vec![
                FakeReply::Error(Error::HttpStatus {
                    status: 503,
                    body: "busy".to_string(),
                }),
                FakeReply::Content(BUY),
            ]),
            System1Config {
                retries: 1,
                ..config()
            },
        );
        let outcome = system.decide(&context()).await.unwrap();
        assert_eq!(outcome.attempts, 2);
        assert_eq!(outcome.decision.action, Action::Buy);
    }

    #[tokio::test]
    async fn guardrails_force_hold_on_low_confidence() {
        let low_confidence =
            r#"{"action":"buy","confidence":0.2,"quantity":1,"stop_loss":125,"rationale":"maybe"}"#;
        let system = System1::new(
            FakeBackend::new(vec![FakeReply::Content(low_confidence)]),
            config(),
        );
        let outcome = system.decide(&context()).await.unwrap();
        assert_eq!(outcome.decision.action, Action::Hold);
        assert_eq!(outcome.source, DecisionSource::GuardrailHold);
        assert!(
            outcome
                .report
                .blocked_reason
                .unwrap()
                .contains("confidence")
        );
    }

    #[tokio::test]
    async fn decide_or_hold_survives_transport_failure() {
        let system = System1::new(
            FakeBackend::new(vec![FakeReply::Error(Error::HttpStatus {
                status: 503,
                body: "offline".to_string(),
            })]),
            config(),
        );
        let outcome = system.decide_or_hold(&context()).await;
        assert_eq!(outcome.decision.action, Action::Hold);
        assert_eq!(outcome.source, DecisionSource::FallbackHold);
    }

    #[tokio::test]
    async fn fake_backend_tracks_calls() {
        let backend = FakeBackend::new(vec![FakeReply::Content("x")]);
        assert_eq!(backend.model(), "fake:test");
        let _ = backend.chat(Vec::new(), None).await;
        assert_eq!(backend.calls(), 1);
    }
}
