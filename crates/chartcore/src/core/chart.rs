// Chart - main chart struct

use super::{buffer::ChartBuffer, config::ChartConfig, types::Candle};

/// Main chart structure
#[derive(Debug, Clone)]
pub struct Chart {
    /// Chart configuration
    pub config: ChartConfig,
    /// Candle data buffer
    buffer: ChartBuffer,
}

impl Chart {
    /// Create new chart with default configuration
    pub fn new() -> Self {
        let config = ChartConfig::default();
        let buffer = ChartBuffer::new(config.max_candles);
        Self { config, buffer }
    }

    /// Create chart with custom configuration
    pub fn with_config(config: ChartConfig) -> Self {
        let buffer = ChartBuffer::new(config.max_candles);
        Self { config, buffer }
    }

    /// Push new candle to chart
    pub fn push(&mut self, candle: Candle) {
        self.buffer.push(candle);
    }

    /// Update last candle (for real-time updates)
    pub fn update_last(&mut self, candle: Candle) {
        self.buffer.update_last(candle);
    }

    /// Get all candles
    pub fn candles(&self) -> Vec<Candle> {
        self.buffer.as_slice()
    }

    /// Get last candle
    pub fn last(&self) -> Option<&Candle> {
        self.buffer.last()
    }

    /// Get last N candles
    pub fn last_n(&self, n: usize) -> Vec<Candle> {
        self.buffer.last_n(n)
    }

    /// Get candle at index
    pub fn get(&self, index: usize) -> Option<&Candle> {
        self.buffer.get(index)
    }

    /// Get number of candles
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if chart is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear all candles
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get visible price range (min, max)
    pub fn price_range(&self) -> Option<(f64, f64)> {
        if self.is_empty() {
            return None;
        }

        let candles = self.candles();
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for candle in candles {
            if candle.l < min {
                min = candle.l;
            }
            if candle.h > max {
                max = candle.h;
            }
        }

        Some((min, max))
    }

    /// Get time range (earliest, latest)
    pub fn time_range(&self) -> Option<(i64, i64)> {
        if self.is_empty() {
            return None;
        }

        let candles = self.candles();
        let earliest = candles.first()?.time;
        let latest = candles.last()?.time;

        Some((earliest, latest))
    }

    /// Find candle by timestamp
    pub fn find_by_time(&self, time: i64) -> Option<(usize, &Candle)> {
        self.buffer.find_by_time(time)
    }

    /// Find nearest candle to timestamp
    pub fn find_nearest(&self, time: i64) -> Option<(usize, &Candle)> {
        self.buffer.find_nearest(time)
    }

    /// Load candles from vector (replaces existing)
    pub fn load(&mut self, candles: Vec<Candle>) {
        self.buffer.clear();
        for candle in candles {
            self.buffer.push(candle);
        }
    }

    /// Append multiple candles
    pub fn append(&mut self, candles: Vec<Candle>) {
        for candle in candles {
            self.buffer.push(candle);
        }
    }
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(time: i64, close: f64) -> Candle {
        Candle::new(time, close - 1.0, close + 1.0, close - 2.0, close, 1000.0)
    }

    #[test]
    fn test_chart_creation() {
        let chart = Chart::new();
        assert!(chart.is_empty());
        assert_eq!(chart.len(), 0);
    }

    #[test]
    fn test_chart_with_config() {
        let config = ChartConfig::new()
            .with_max_candles(500)
            .with_symbol("BTC/USD");

        let chart = Chart::with_config(config);
        assert_eq!(chart.config.max_candles, 500);
        assert_eq!(chart.config.symbol.as_deref(), Some("BTC/USD"));
    }

    #[test]
    fn test_chart_push_update() {
        let mut chart = Chart::new();

        chart.push(create_test_candle(1000, 100.0));
        chart.push(create_test_candle(2000, 101.0));

        assert_eq!(chart.len(), 2);
        assert_eq!(chart.last().unwrap().c, 101.0);

        // Update last candle
        chart.update_last(create_test_candle(2000, 102.0));
        assert_eq!(chart.len(), 2);
        assert_eq!(chart.last().unwrap().c, 102.0);
    }

    #[test]
    fn test_chart_ranges() {
        let mut chart = Chart::new();

        chart.push(create_test_candle(1000, 100.0));
        chart.push(create_test_candle(2000, 105.0));
        chart.push(create_test_candle(3000, 95.0));

        let (min, max) = chart.price_range().unwrap();
        assert_eq!(min, 93.0); // lowest low
        assert_eq!(max, 106.0); // highest high

        let (earliest, latest) = chart.time_range().unwrap();
        assert_eq!(earliest, 1000);
        assert_eq!(latest, 3000);
    }

    #[test]
    fn test_chart_load_append() {
        let mut chart = Chart::new();

        let candles = vec![
            create_test_candle(1000, 100.0),
            create_test_candle(2000, 101.0),
        ];

        chart.load(candles);
        assert_eq!(chart.len(), 2);

        chart.append(vec![create_test_candle(3000, 102.0)]);
        assert_eq!(chart.len(), 3);
    }
}
