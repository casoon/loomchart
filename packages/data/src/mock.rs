//! Mock data provider for testing.

use loom_core::{Candle, Timeframe, Timestamp};
use crate::{DataProvider, DataResult, DataError, TimeRange};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, vec};

/// Mock data provider that generates synthetic data
pub struct MockProvider {
    candles: Vec<Candle>,
    symbol: String,
}

impl MockProvider {
    /// Create a new mock provider with custom candles
    pub fn new(symbol: &str, candles: Vec<Candle>) -> Self {
        Self {
            candles,
            symbol: symbol.into(),
        }
    }

    /// Generate trending data
    pub fn trending(count: usize, trend_strength: f64) -> Self {
        let mut candles = Vec::with_capacity(count);
        let mut price = 100.0;

        for i in 0..count {
            let trend = trend_strength * (1.0 + (i as f64 * 0.1).sin() * 0.3);
            let open = price;
            let close = price + trend;
            let high = open.max(close) + trend.abs() * 0.3;
            let low = open.min(close) - trend.abs() * 0.2;
            let volume = 1000.0 + (i as f64 * 0.2).sin().abs() * 500.0;

            candles.push(Candle::new(
                (i as i64) * 60000,
                open,
                high,
                low,
                close,
                volume,
            ));
            price = close;
        }

        Self {
            candles,
            symbol: "MOCK/USD".into(),
        }
    }

    /// Generate ranging/sideways data
    pub fn ranging(count: usize, center: f64, range_size: f64) -> Self {
        let mut candles = Vec::with_capacity(count);

        for i in 0..count {
            let offset = (i as f64 * 0.3).sin() * range_size * 0.4;
            let open = center + offset;
            let noise = (i as f64 * 0.7).cos() * range_size * 0.1;
            let close = center + offset + noise;
            let high = open.max(close) + range_size * 0.1;
            let low = open.min(close) - range_size * 0.1;

            candles.push(Candle::new(
                (i as i64) * 60000,
                open,
                high,
                low,
                close,
                1000.0,
            ));
        }

        Self {
            candles,
            symbol: "MOCK/USD".into(),
        }
    }

    /// Generate volatile data
    pub fn volatile(count: usize, volatility: f64) -> Self {
        let mut candles = Vec::with_capacity(count);
        let mut price = 100.0;

        for i in 0..count {
            let change = (i as f64 * 1.5).sin() * volatility
                + (i as f64 * 0.3).cos() * volatility * 0.5;
            let open = price;
            let close = price + change;
            let high = open.max(close) + volatility * 0.5;
            let low = open.min(close) - volatility * 0.5;
            let volume = 1000.0 + change.abs() * 100.0;

            candles.push(Candle::new(
                (i as i64) * 60000,
                open,
                high,
                low,
                close,
                volume,
            ));
            price = close;
        }

        Self {
            candles,
            symbol: "MOCK/USD".into(),
        }
    }

    /// Generate OHLC patterns for testing
    pub fn with_pattern(pattern: MockPattern) -> Self {
        let candles = match pattern {
            MockPattern::Doji(count) => Self::generate_doji(count),
            MockPattern::Hammer(count) => Self::generate_hammer(count),
            MockPattern::Engulfing(count) => Self::generate_engulfing(count),
            MockPattern::ThreeWhiteSoldiers(count) => Self::generate_three_white_soldiers(count),
            MockPattern::HeadAndShoulders => Self::generate_head_and_shoulders(),
        };

        Self {
            candles,
            symbol: "MOCK/USD".into(),
        }
    }

    fn generate_doji(count: usize) -> Vec<Candle> {
        (0..count)
            .map(|i| {
                let price = 100.0;
                Candle::new(
                    (i as i64) * 60000,
                    price,
                    price + 2.0,
                    price - 2.0,
                    price + 0.1, // Close very near open
                    1000.0,
                )
            })
            .collect()
    }

    fn generate_hammer(count: usize) -> Vec<Candle> {
        (0..count)
            .map(|i| {
                let price = 100.0;
                Candle::new(
                    (i as i64) * 60000,
                    price,
                    price + 0.5,     // Small upper wick
                    price - 4.0,     // Long lower wick
                    price + 0.3,     // Close near open
                    1000.0,
                )
            })
            .collect()
    }

    fn generate_engulfing(count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        for i in 0..count {
            let is_bullish_engulfing = i % 2 == 1;
            if is_bullish_engulfing {
                // Small bearish candle followed by large bullish
                if i > 0 {
                    let prev = &candles[i - 1];
                    candles.push(Candle::new(
                        (i as i64) * 60000,
                        prev.close - 0.5,
                        prev.high + 1.0,
                        prev.low - 0.5,
                        prev.open + 1.0, // Engulfs previous
                        1500.0,
                    ));
                } else {
                    candles.push(Candle::new(
                        0,
                        100.0,
                        101.0,
                        99.0,
                        99.5,
                        1000.0,
                    ));
                }
            } else {
                // Small bearish
                candles.push(Candle::new(
                    (i as i64) * 60000,
                    100.0,
                    100.5,
                    99.5,
                    99.7,
                    800.0,
                ));
            }
        }
        candles
    }

    fn generate_three_white_soldiers(count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let mut price = 100.0;

        for i in 0..count {
            let open = price;
            let close = price + 2.0;
            candles.push(Candle::new(
                (i as i64) * 60000,
                open,
                close + 0.5,
                open - 0.2,
                close,
                1000.0 + (i as f64) * 100.0,
            ));
            price = close + 0.1; // Each opens near previous close
        }

        candles
    }

    fn generate_head_and_shoulders() -> Vec<Candle> {
        let mut candles = Vec::new();
        let mut time = 0i64;

        // Left shoulder up
        for i in 0..5 {
            let price = 100.0 + (i as f64) * 2.0;
            candles.push(Candle::new(time, price, price + 1.0, price - 0.5, price + 0.5, 1000.0));
            time += 60000;
        }

        // Left shoulder down
        for i in 0..3 {
            let price = 110.0 - (i as f64) * 2.0;
            candles.push(Candle::new(time, price, price + 0.5, price - 1.0, price - 0.5, 1000.0));
            time += 60000;
        }

        // Head up (higher peak)
        for i in 0..7 {
            let price = 104.0 + (i as f64) * 2.5;
            candles.push(Candle::new(time, price, price + 1.0, price - 0.5, price + 0.8, 1200.0));
            time += 60000;
        }

        // Head down
        for i in 0..5 {
            let price = 121.5 - (i as f64) * 2.5;
            candles.push(Candle::new(time, price, price + 0.5, price - 1.0, price - 0.8, 1100.0));
            time += 60000;
        }

        // Right shoulder up
        for i in 0..4 {
            let price = 109.0 + (i as f64) * 1.5;
            candles.push(Candle::new(time, price, price + 1.0, price - 0.5, price + 0.5, 900.0));
            time += 60000;
        }

        // Right shoulder down (breakdown)
        for i in 0..6 {
            let price = 115.0 - (i as f64) * 2.0;
            candles.push(Candle::new(time, price, price + 0.5, price - 1.0, price - 0.7, 1300.0));
            time += 60000;
        }

        candles
    }
}

/// Predefined patterns for testing
pub enum MockPattern {
    /// Doji candles
    Doji(usize),
    /// Hammer candles
    Hammer(usize),
    /// Engulfing patterns
    Engulfing(usize),
    /// Three white soldiers
    ThreeWhiteSoldiers(usize),
    /// Head and shoulders
    HeadAndShoulders,
}

impl DataProvider for MockProvider {
    fn get_candles(
        &self,
        symbol: &str,
        _timeframe: Timeframe,
        range: TimeRange,
    ) -> DataResult<Vec<Candle>> {
        if symbol != self.symbol {
            return Err(DataError::InvalidSymbol(symbol.into()));
        }

        Ok(range.filter(self.candles.clone()))
    }

    fn symbols(&self) -> DataResult<Vec<String>> {
        Ok(vec![self.symbol.clone()])
    }

    fn timeframes(&self, _symbol: &str) -> DataResult<Vec<Timeframe>> {
        Ok(vec![
            Timeframe::M1,
            Timeframe::M5,
            Timeframe::M15,
            Timeframe::H1,
            Timeframe::H4,
            Timeframe::D1,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trending_provider() {
        let provider = MockProvider::trending(100, 0.5);
        let candles = provider.get_candles("MOCK/USD", Timeframe::M1, TimeRange::all()).unwrap();

        assert_eq!(candles.len(), 100);

        // Verify uptrend
        let first_close = candles.first().unwrap().close;
        let last_close = candles.last().unwrap().close;
        assert!(last_close > first_close, "Should trend upward");
    }

    #[test]
    fn test_ranging_provider() {
        let provider = MockProvider::ranging(100, 100.0, 10.0);
        let candles = provider.get_candles("MOCK/USD", Timeframe::M1, TimeRange::all()).unwrap();

        // All prices should be within range
        for candle in &candles {
            assert!(candle.close > 85.0 && candle.close < 115.0);
        }
    }
}
