//! Run a decision against a local Ollama instance using a deterministic sample
//! snapshot.
//!
//! ```sh
//! ollama serve
//! ollama pull tev1:4b
//! cargo run -p system1 --example decide
//! cargo run -p system1 --example decide -- --model tev1:4b --url http://127.0.0.1:11434
//! ```

use std::process::ExitCode;

use system1::{
    AccountState, Action, Candle, MarketKind, MarketState, OllamaClient, OllamaConfig, RiskPolicy,
    System1, System1Config, TradingContext,
};

#[tokio::main]
async fn main() -> ExitCode {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut config = OllamaConfig::from_env()?;
    if let Some(value) = arg_value(&args, "--model") {
        config.model = value;
    }
    if let Some(value) = arg_value(&args, "--url") {
        config.base_url = value;
    }

    let client = OllamaClient::new(config)?;
    println!("model:     {}", client.model());
    println!("ollama:    {}", client.config().base_url);

    match client.health().await {
        Ok(health) => {
            if !health.model_present {
                eprintln!(
                    "model `{}` is not installed. Run `ollama pull {}`.",
                    health.model, health.model
                );
                return Err("model is not installed".into());
            }
            println!("available: {}", health.models.join(", "));
        }
        Err(error) => {
            eprintln!("could not reach Ollama: {error}");
            eprintln!("Start it with `ollama serve`, then run `ollama pull tev1:4b`.");
            return Err(error.into());
        }
    }

    let system = System1::new(client, System1Config::default());
    let context = sample_context();
    let started = std::time::Instant::now();
    let outcome = system.decide(&context).await?;

    println!(
        "\ndecision after {:?} ({} attempt(s)):\n{}",
        started.elapsed(),
        outcome.attempts,
        serde_json::to_string_pretty(&outcome)?
    );

    match outcome.decision.action {
        Action::Hold => println!("\n-> no order"),
        Action::Buy => println!("\n-> BUY {}", outcome.decision.quantity.unwrap_or_default()),
        Action::Sell => println!(
            "\n-> SELL {}",
            outcome.decision.quantity.unwrap_or_default()
        ),
        Action::Close => println!("\n-> CLOSE {:?}", outcome.decision.quantity),
    }

    Ok(())
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn print_help() {
    println!(
        "system1 decide example\n\n\
         Usage: cargo run -p system1 --example decide [-- --model NAME --url URL]\n\n\
         Environment overrides: SYSTEM1_OLLAMA_URL / OLLAMA_HOST, SYSTEM1_MODEL,\n\
         SYSTEM1_TIMEOUT_SECS, SYSTEM1_TEMPERATURE, SYSTEM1_SEED"
    );
}

fn sample_context() -> TradingContext {
    let mut price = 96_000.0;
    let mut candles = Vec::with_capacity(80);
    for index in 0..80 {
        let drift = 16.0 + (index as f64 / 5.0).sin() * 55.0;
        let open = price;
        let close = (open + drift).max(1.0);
        let high = open.max(close) + ((index % 7) as f64 + 1.0) * 9.0;
        let low = (open.min(close) - ((index % 5) as f64 + 1.0) * 9.0).max(0.01);
        let volume = 1_200.0 + ((index * 13) % 400) as f64;
        candles.push(Candle {
            open,
            high,
            low,
            close,
            volume,
        });
        price = close;
    }

    let last = price;
    TradingContext {
        as_of: "2026-11-18T12:00:00Z".to_string(),
        symbol: "BTCUSDT".to_string(),
        strategy: Some("sample momentum".to_string()),
        timeframe: Some("5m".to_string()),
        market: MarketState {
            market_type: MarketKind::Crypto,
            session_open: true,
            stale: false,
            bid: last - 6.0,
            ask: last + 6.0,
            last,
            high_24h: Some(last * 1.03),
            low_24h: Some(last * 0.96),
            volume_24h: Some(9_200_000.0),
            change_pct_24h: Some(0.9),
            candles,
            order_book: None,
        },
        account: AccountState {
            equity: 122_000.0,
            cash: 80_000.0,
            margin_used: 0.0,
            day_pnl: 1_842.0,
            day_pnl_pct: Some(1.53),
            open_positions: 1,
            gross_exposure: Some(14_000.0),
        },
        position: Some(system1::PositionState {
            quantity: 0.15,
            average_price: last * 0.97,
            unrealized_pnl: 0.15 * last * 0.03,
        }),
        open_orders: Vec::new(),
        recent_trades: Vec::new(),
        seconds_since_last_trade: Some(600),
        risk: RiskPolicy::default(),
        notes: vec!["Deterministic example snapshot; not live market data.".to_string()],
    }
}
