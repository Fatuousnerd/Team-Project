//! Deterministic feature extraction.
//!
//! A 4B model reasons better over compact, pre-computed indicators than over a
//! raw candle dump. These features are also shown to the caller in the
//! [`crate::DecisionOutcome`] so the UI can explain what the model saw.

use serde::{Deserialize, Serialize};

use crate::types::{BookLevel, Candle, MarketState, OrderBookState};

const RSI_PERIOD: usize = 14;
const ATR_PERIOD: usize = 14;
const VOL_PERIOD: usize = 20;
const BOOK_LEVELS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    #[default]
    Flat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketFeatures {
    /// Bid, ask, and last are positive and the book is not crossed.
    pub quote_valid: bool,
    pub mid: f64,
    pub spread_bps: f64,
    pub candle_count: usize,
    pub return_1_pct: Option<f64>,
    pub return_5_pct: Option<f64>,
    pub return_20_pct: Option<f64>,
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub ema_12: Option<f64>,
    pub ema_26: Option<f64>,
    pub macd: Option<f64>,
    pub macd_signal: Option<f64>,
    pub macd_hist: Option<f64>,
    pub rsi_14: Option<f64>,
    /// Average true range in quote currency.
    pub atr_14: Option<f64>,
    /// ATR as a percentage of the mid price.
    pub atr_pct: Option<f64>,
    /// Standard deviation of the last 20 log returns, in percent.
    pub realized_vol_20_pct: Option<f64>,
    pub volume_ratio_20: Option<f64>,
    /// 0 = at the 20-bar low, 1 = at the 20-bar high.
    pub range_position_20: Option<f64>,
    pub support_20: Option<f64>,
    pub resistance_20: Option<f64>,
    pub price_vs_sma_20_pct: Option<f64>,
    pub price_vs_sma_50_pct: Option<f64>,
    pub trend: Trend,
    /// Notional resting within the top 10 bid levels.
    pub bid_depth_notional: Option<f64>,
    /// Notional resting within the top 10 ask levels.
    pub ask_depth_notional: Option<f64>,
    /// `(bid_depth - ask_depth) / (bid_depth + ask_depth)`, in `-1..=1`.
    pub book_imbalance: Option<f64>,
}

impl MarketFeatures {
    pub fn compute(market: &MarketState) -> Self {
        let quote_valid = market.bid.is_finite()
            && market.ask.is_finite()
            && market.last.is_finite()
            && market.bid > 0.0
            && market.ask > 0.0
            && market.last > 0.0
            && market.ask >= market.bid;

        let mid = if quote_valid {
            (market.bid + market.ask) / 2.0
        } else if market.last.is_finite() && market.last > 0.0 {
            market.last
        } else {
            0.0
        };

        let spread_bps = if quote_valid && mid > 0.0 {
            ((market.ask - market.bid) / mid * 10_000.0).max(0.0)
        } else {
            0.0
        };

        let candles: Vec<Candle> = market
            .candles
            .iter()
            .copied()
            .filter(Candle::is_valid)
            .collect();
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        let sma_20 = sma(&closes, 20);
        let sma_50 = sma(&closes, 50);
        let ema_12 = ema(&closes, 12);
        let ema_26 = ema(&closes, 26);
        let macd = match (ema_12, ema_26) {
            (Some(fast), Some(slow)) => Some(fast - slow),
            _ => None,
        };
        let macd_signal = macd_signal_series(&closes);
        let macd_hist = match (macd, macd_signal) {
            (Some(macd), Some(signal)) => Some(macd - signal),
            _ => None,
        };

        let atr_14 = atr(&candles, ATR_PERIOD);
        let atr_pct = match (atr_14, mid) {
            (Some(atr), mid) if mid > 0.0 => Some(atr / mid * 100.0),
            _ => None,
        };

        let (bid_depth_notional, ask_depth_notional, book_imbalance) =
            book_features(market.order_book.as_ref());

        let trend = trend(ema_12, ema_26, market.last, sma_20);

        Self {
            quote_valid,
            mid,
            spread_bps,
            candle_count: candles.len(),
            return_1_pct: return_pct(&closes, 1),
            return_5_pct: return_pct(&closes, 5),
            return_20_pct: return_pct(&closes, 20),
            sma_20,
            sma_50,
            ema_12,
            ema_26,
            macd,
            macd_signal,
            macd_hist,
            rsi_14: rsi(&closes, RSI_PERIOD),
            atr_14,
            atr_pct,
            realized_vol_20_pct: realized_vol(&closes, VOL_PERIOD),
            volume_ratio_20: volume_ratio(&candles, VOL_PERIOD),
            range_position_20: range_position(&candles, VOL_PERIOD),
            support_20: window_min_low(&candles, VOL_PERIOD),
            resistance_20: window_max_high(&candles, VOL_PERIOD),
            price_vs_sma_20_pct: pct_from(market.last, sma_20),
            price_vs_sma_50_pct: pct_from(market.last, sma_50),
            trend,
            bid_depth_notional,
            ask_depth_notional,
            book_imbalance,
        }
    }
}

fn trend(ema_12: Option<f64>, ema_26: Option<f64>, last: f64, sma_20: Option<f64>) -> Trend {
    if let (Some(fast), Some(slow)) = (ema_12, ema_26)
        && fast > 0.0
        && slow > 0.0
    {
        let diff_pct = (fast - slow) / slow * 100.0;
        if diff_pct > 0.05 {
            return Trend::Up;
        }
        if diff_pct < -0.05 {
            return Trend::Down;
        }
        return Trend::Flat;
    }

    if let Some(sma) = sma_20
        && last.is_finite()
        && sma > 0.0
    {
        let diff_pct = (last - sma) / sma * 100.0;
        if diff_pct > 0.05 {
            return Trend::Up;
        }
        if diff_pct < -0.05 {
            return Trend::Down;
        }
    }

    Trend::Flat
}

fn pct_from(value: f64, reference: Option<f64>) -> Option<f64> {
    let reference = reference?;
    if !value.is_finite() || reference <= 0.0 {
        return None;
    }
    finite(Some((value - reference) / reference * 100.0))
}

fn return_pct(closes: &[f64], periods: usize) -> Option<f64> {
    if periods == 0 || closes.len() <= periods {
        return None;
    }
    let last = *closes.last()?;
    let base = closes[closes.len() - 1 - periods];
    if base <= 0.0 {
        return None;
    }
    finite(Some((last / base - 1.0) * 100.0))
}

fn sma(values: &[f64], period: usize) -> Option<f64> {
    if period == 0 || values.len() < period {
        return None;
    }
    let slice = &values[values.len() - period..];
    finite(Some(slice.iter().sum::<f64>() / period as f64))
}

fn ema_series(values: &[f64], period: usize) -> Option<Vec<f64>> {
    if period == 0 || values.is_empty() {
        return None;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = Vec::with_capacity(values.len());
    let mut current = values[0];
    out.push(current);
    for value in &values[1..] {
        current = alpha * value + (1.0 - alpha) * current;
        out.push(current);
    }
    if out.iter().all(|value| value.is_finite()) {
        Some(out)
    } else {
        None
    }
}

fn ema(values: &[f64], period: usize) -> Option<f64> {
    ema_series(values, period)?
        .last()
        .copied()
        .and_then(|v| finite(Some(v)))
}

fn macd_signal_series(closes: &[f64]) -> Option<f64> {
    let fast = ema_series(closes, 12)?;
    let slow = ema_series(closes, 26)?;
    let macd: Vec<f64> = fast
        .iter()
        .zip(slow.iter())
        .map(|(fast, slow)| fast - slow)
        .collect();
    ema(&macd, 9)
}

fn rsi(closes: &[f64], period: usize) -> Option<f64> {
    if period == 0 || closes.len() <= period {
        return None;
    }

    let mut gain = 0.0;
    let mut loss = 0.0;
    for window in closes[..=period].windows(2) {
        let delta = window[1] - window[0];
        if delta > 0.0 {
            gain += delta;
        } else {
            loss -= delta;
        }
    }

    let period_f = period as f64;
    let mut avg_gain = gain / period_f;
    let mut avg_loss = loss / period_f;

    for window in closes[period..].windows(2) {
        let delta = window[1] - window[0];
        let (gain, loss) = if delta > 0.0 {
            (delta, 0.0)
        } else {
            (0.0, -delta)
        };
        avg_gain = (avg_gain * (period_f - 1.0) + gain) / period_f;
        avg_loss = (avg_loss * (period_f - 1.0) + loss) / period_f;
    }

    let value = if avg_loss <= f64::EPSILON {
        if avg_gain <= f64::EPSILON {
            50.0
        } else {
            100.0
        }
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    };
    finite(Some(value))
}

fn atr(candles: &[Candle], period: usize) -> Option<f64> {
    if period == 0 || candles.len() <= period {
        return None;
    }

    let mut true_ranges = Vec::with_capacity(candles.len());
    for (index, candle) in candles.iter().enumerate() {
        let previous_close = index.checked_sub(1).map(|i| candles[i].close);
        true_ranges.push(candle.true_range(previous_close));
    }

    let mut atr = true_ranges[..period].iter().sum::<f64>() / period as f64;
    for tr in &true_ranges[period..] {
        atr = (atr * (period as f64 - 1.0) + tr) / period as f64;
    }
    finite(Some(atr))
}

fn realized_vol(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() < period + 1 {
        return None;
    }
    let start = closes.len() - period - 1;
    let returns: Vec<f64> = closes[start..]
        .windows(2)
        .filter_map(|w| {
            if w[0] > 0.0 && w[1] > 0.0 {
                Some((w[1] / w[0]).ln() * 100.0)
            } else {
                None
            }
        })
        .collect();
    if returns.len() < 2 {
        return None;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    finite(Some(variance.sqrt()))
}

fn volume_ratio(candles: &[Candle], period: usize) -> Option<f64> {
    if period == 0 || candles.len() < period {
        return None;
    }
    let slice = &candles[candles.len() - period..];
    let mean = slice.iter().map(|c| c.volume).sum::<f64>() / period as f64;
    let last = candles.last()?.volume;
    if mean <= 0.0 {
        return None;
    }
    finite(Some(last / mean))
}

fn range_position(candles: &[Candle], period: usize) -> Option<f64> {
    let low = window_min_low(candles, period)?;
    let high = window_max_high(candles, period)?;
    let last = candles.last()?.close;
    if high <= low || !last.is_finite() {
        return None;
    }
    finite(Some(((last - low) / (high - low)).clamp(0.0, 1.0)))
}

fn window_min_low(candles: &[Candle], period: usize) -> Option<f64> {
    if period == 0 || candles.len() < period {
        return None;
    }
    let slice = &candles[candles.len() - period..];
    finite(slice.iter().map(|c| c.low).reduce(f64::min))
}

fn window_max_high(candles: &[Candle], period: usize) -> Option<f64> {
    if period == 0 || candles.len() < period {
        return None;
    }
    let slice = &candles[candles.len() - period..];
    finite(slice.iter().map(|c| c.high).reduce(f64::max))
}

fn book_features(book: Option<&OrderBookState>) -> (Option<f64>, Option<f64>, Option<f64>) {
    let Some(book) = book else {
        return (None, None, None);
    };

    let bid_depth = depth(book.bids.iter().copied(), true);
    let ask_depth = depth(book.asks.iter().copied(), false);

    let imbalance = match (bid_depth, ask_depth) {
        (Some(bid), Some(ask)) if bid + ask > 0.0 => finite(Some((bid - ask) / (bid + ask))),
        _ => None,
    };
    (bid_depth, ask_depth, imbalance)
}

fn depth(levels: impl Iterator<Item = BookLevel>, bids: bool) -> Option<f64> {
    let mut levels: Vec<BookLevel> = levels
        .filter(|level| {
            level.price.is_finite()
                && level.size.is_finite()
                && level.price > 0.0
                && level.size > 0.0
        })
        .collect();
    if levels.is_empty() {
        return None;
    }
    if bids {
        levels.sort_by(|a, b| b.price.total_cmp(&a.price));
    } else {
        levels.sort_by(|a, b| a.price.total_cmp(&b.price));
    }
    let notional = levels
        .iter()
        .take(BOOK_LEVELS)
        .map(|level| level.price * level.size)
        .sum::<f64>();
    finite(Some(notional))
}

fn finite(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MarketKind;

    fn candle(close: f64) -> Candle {
        Candle {
            open: close,
            high: close * 1.01,
            low: close * 0.99,
            close,
            volume: 100.0,
        }
    }

    fn market(candles: Vec<Candle>) -> MarketState {
        MarketState {
            market_type: MarketKind::Crypto,
            session_open: true,
            stale: false,
            bid: 99.9,
            ask: 100.1,
            last: 100.0,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            change_pct_24h: None,
            candles,
            order_book: None,
        }
    }

    #[test]
    fn computes_indicators_for_a_trend() {
        let candles: Vec<Candle> = (0..60).map(|i| candle(100.0 + i as f64)).collect();
        let features = MarketFeatures::compute(&market(candles));

        assert!(features.quote_valid);
        assert_eq!(features.candle_count, 60);
        assert!(features.sma_20.unwrap() > features.sma_50.unwrap());
        assert!(features.rsi_14.unwrap() > 70.0);
        assert!(features.atr_14.unwrap() > 0.0);
        assert_eq!(features.trend, Trend::Up);
        assert!(features.return_20_pct.unwrap() > 0.0);
    }

    #[test]
    fn drops_malformed_candles() {
        let mut candles: Vec<Candle> = (0..25).map(|i| candle(100.0 + i as f64)).collect();
        candles.push(Candle {
            open: 10.0,
            high: 5.0,
            low: 20.0,
            close: 10.0,
            volume: 1.0,
        });
        let features = MarketFeatures::compute(&market(candles));
        assert_eq!(features.candle_count, 25);
    }

    #[test]
    fn rejects_crossed_quotes() {
        let mut state = market(vec![candle(100.0); 30]);
        state.bid = 101.0;
        state.ask = 99.0;
        let features = MarketFeatures::compute(&state);
        assert!(!features.quote_valid);
        assert_eq!(features.spread_bps, 0.0);
    }

    #[test]
    fn computes_book_imbalance() {
        let mut state = market(vec![candle(100.0); 30]);
        state.order_book = Some(OrderBookState {
            bids: vec![
                BookLevel {
                    price: 99.9,
                    size: 10.0,
                },
                BookLevel {
                    price: 99.8,
                    size: 20.0,
                },
            ],
            asks: vec![BookLevel {
                price: 100.1,
                size: 5.0,
            }],
        });
        let features = MarketFeatures::compute(&state);
        assert!(features.book_imbalance.unwrap() > 0.0);
        assert!(features.bid_depth_notional.unwrap() > features.ask_depth_notional.unwrap());
    }

    #[test]
    fn handles_short_and_flat_series() {
        let features = MarketFeatures::compute(&market(vec![candle(100.0); 5]));
        assert_eq!(features.candle_count, 5);
        assert!(features.rsi_14.is_none());
        assert!(features.sma_20.is_none());
        assert_eq!(features.trend, Trend::Flat);
    }
}
