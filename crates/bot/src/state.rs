use std::sync::Arc;
use std::sync::RwLock;

use crate::market::MockMarket;
use crate::model::TradingEngine;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<TradingEngine>,
    /// Same `RwLock<MockMarket>` held by the engine, exposed separately so the
    /// price-ticker task and the WebSocket feed can read/write it directly.
    pub market: Arc<RwLock<MockMarket>>,
}

impl AppState {
    pub fn new() -> Self {
        let engine = Arc::new(TradingEngine::new());
        let market = engine.market.clone();
        Self { engine, market }
    }
}
