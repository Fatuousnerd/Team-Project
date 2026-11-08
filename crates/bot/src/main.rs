use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use trading_bot_logic::AppState;
use trading_bot_logic::server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = Arc::new(AppState::new());
    state.engine.seed_accounts();

    // Background task: simulate live price movement every 500ms so the GUIs
    // have streaming mock data before a real market-data API is connected.
    let market = state.market.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            market.write().unwrap().tick();
        }
    });

    let addr: SocketAddr = "0.0.0.0:3001"
        .parse()
        .expect("valid socket address");
    server::run(state, addr).await;
}
