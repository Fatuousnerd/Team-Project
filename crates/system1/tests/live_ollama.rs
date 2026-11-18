//! Live smoke test. Run with Ollama up:
//!
//! ```sh
//! ollama serve
//! ollama pull tev1:4b
//! cargo test -p system1 --test live_ollama -- --ignored
//! ```

use system1::{
    AccountState, Action, Candle, MarketKind, MarketState, PositionState, RiskPolicy, System1,
    TradingContext,
};

#[tokio::test]
#[ignore = "requires a local Ollama instance with tev1:4b pulled"]
async fn live_decision_smoke() {
    let system = System1::from_env().expect("valid environment");
    system.ensure_model().await.expect("model available");

    let outcome = system
        .decide(&context())
        .await
        .expect("decision request succeeds");

    assert!(outcome.decision.confidence.is_finite());
    assert!(outcome.attempts >= 1);
    assert_eq!(outcome.model, system.model());

    match outcome.decision.action {
        Action::Buy | Action::Sell => {
            let quantity = outcome.decision.quantity.expect("entry has quantity");
            assert!(quantity > 0.0);
            assert!(outcome.decision.stop_loss.is_some());
        }
        Action::Close => {
            assert!(outcome.decision.quantity.unwrap_or(0.0) > 0.0);
        }
        Action::Hold => {
            assert!(!outcome.decision.rationale.trim().is_empty());
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&outcome).expect("serializable outcome")
    );
}

fn context() -> TradingContext {
    let mut price = 96_000.0;
    let mut candles = Vec::with_capacity(60);
    for index in 0..60 {
        let drift = 14.0 + (index as f64 / 4.0).sin() * 40.0;
        let open = price;
        let close = (open + drift).max(1.0);
        let high = open.max(close) + ((index % 5) as f64 + 1.0) * 10.0;
        let low = (open.min(close) - ((index % 3) as f64 + 1.0) * 10.0).max(0.01);
        candles.push(Candle {
            open,
            high,
            low,
            close,
            volume: 1_000.0 + index as f64 * 3.0,
        });
        price = close;
    }

    TradingContext {
        as_of: "2026-11-18T12:00:00Z".to_string(),
        symbol: "BTCUSDT".to_string(),
        strategy: Some("live smoke".to_string()),
        timeframe: Some("5m".to_string()),
        market: MarketState {
            market_type: MarketKind::Crypto,
            session_open: true,
            stale: false,
            bid: price - 6.0,
            ask: price + 6.0,
            last: price,
            high_24h: Some(price * 1.03),
            low_24h: Some(price * 0.97),
            volume_24h: Some(8_000_000.0),
            change_pct_24h: Some(1.1),
            candles,
            order_book: None,
        },
        account: AccountState {
            equity: 100_000.0,
            cash: 100_000.0,
            margin_used: 0.0,
            day_pnl: 0.0,
            day_pnl_pct: Some(0.0),
            open_positions: 1,
            gross_exposure: Some(10_000.0),
        },
        position: Some(PositionState {
            quantity: 0.1,
            average_price: price * 0.98,
            unrealized_pnl: 0.1 * price * 0.02,
        }),
        open_orders: Vec::new(),
        recent_trades: Vec::new(),
        seconds_since_last_trade: Some(900),
        risk: RiskPolicy::default(),
        notes: vec!["live smoke test snapshot".to_string()],
    }
}
