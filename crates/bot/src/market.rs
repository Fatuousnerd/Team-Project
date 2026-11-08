use crate::types::*;
use chrono::Utc;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;

const STOCKS: &[&str] = &["AAPL", "TSLA", "MSFT", "GOOGL", "AMZN", "NVDA", "META"];
const FOREX: &[&str] = &["EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD", "USDCHF"];
const CRYPTO: &[&str] = &["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT", "DOGEUSDT", "XRPUSDT"];

/// In-memory mock market. Seeded with realistic-looking symbols for stocks,
/// forex, and crypto. A background task calls `tick()` to simulate price
/// movement so the GUIs have live(ish) data to render before a real API is wired.
pub struct MockMarket {
    prices: HashMap<Symbol, MarketData>,
    rng: StdRng,
}

impl MockMarket {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        let mut rng = StdRng::from_entropy();

        let mut base: Vec<(&str, f64)> = Vec::new();
        for s in STOCKS {
            base.push((s, 50.0 + rng.gen_range(0.0..450.0)));
        }
        for s in FOREX {
            base.push((s, 0.6 + rng.gen_range(0.0..1.6)));
        }
        for s in CRYPTO {
            base.push((s, 0.1 + rng.gen_range(0.0..110_000.0)));
        }

        for (symbol, mid) in base {
            let bid = rust_decimal::Decimal::from_f64_retain(mid * 0.999).unwrap();
            let ask = rust_decimal::Decimal::from_f64_retain(mid * 1.001).unwrap();
            let last = rust_decimal::Decimal::from_f64_retain(mid).unwrap();
            let sym = Symbol::new(symbol.to_string());
            prices.insert(
                sym.clone(),
                MarketData {
                    symbol: sym,
                    bid: bid,
                    ask: ask,
                    last_price: last,
                    volume: rust_decimal::Decimal::from(rng.gen_range(1_000..1_000_000)),
                    timestamp: Utc::now(),
                },
            );
        }

        Self { prices, rng }
    }

    /// Advance every symbol by a small random step (random walk).
    pub fn tick(&mut self) {
        for data in self.prices.values_mut() {
            let change =
                rust_decimal::Decimal::from_f64_retain(self.rng.gen_range(-0.005..0.005)).unwrap();
            let multiplier = rust_decimal::Decimal::ONE + change;
            data.last_price = data.last_price * multiplier;
            let last = data.last_price;
            data.bid = last * rust_decimal::Decimal::from_f64_retain(0.999).unwrap();
            data.ask = last * rust_decimal::Decimal::from_f64_retain(1.001).unwrap();
            data.volume += rust_decimal::Decimal::from(self.rng.gen_range(1..100));
            data.timestamp = Utc::now();
        }
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&MarketData> {
        self.prices.get(symbol)
    }

    pub fn list(&self) -> Vec<MarketData> {
        self.prices.values().cloned().collect()
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        self.prices.keys().cloned().collect()
    }
}

impl Default for MockMarket {
    fn default() -> Self {
        Self::new()
    }
}
