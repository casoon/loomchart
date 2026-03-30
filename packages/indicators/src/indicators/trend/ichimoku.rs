//! Ichimoku Cloud (Ichimoku Kinko Hyo)

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Ichimoku Cloud Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IchimokuOutput {
    /// Tenkan-sen (Conversion Line) - (9-period high + 9-period low) / 2
    pub tenkan: f64,
    /// Kijun-sen (Base Line) - (26-period high + 26-period low) / 2
    pub kijun: f64,
    /// Senkou Span A (Leading Span A) - (Tenkan + Kijun) / 2, plotted 26 periods ahead
    pub senkou_a: f64,
    /// Senkou Span B (Leading Span B) - (52-period high + 52-period low) / 2, plotted 26 periods ahead
    pub senkou_b: f64,
    /// Chikou Span (Lagging Span) - Close plotted 26 periods behind
    pub chikou: f64,
}

impl IchimokuOutput {
    pub fn new(tenkan: f64, kijun: f64, senkou_a: f64, senkou_b: f64, chikou: f64) -> Self {
        Self { tenkan, kijun, senkou_a, senkou_b, chikou }
    }

    /// Check if bullish (price above cloud)
    pub fn is_bullish(&self, price: f64) -> bool {
        price > self.senkou_a.max(self.senkou_b)
    }

    /// Check if bearish (price below cloud)
    pub fn is_bearish(&self, price: f64) -> bool {
        price < self.senkou_a.min(self.senkou_b)
    }

    /// Check if price is inside the cloud
    pub fn is_in_cloud(&self, price: f64) -> bool {
        let upper = self.senkou_a.max(self.senkou_b);
        let lower = self.senkou_a.min(self.senkou_b);
        price >= lower && price <= upper
    }

    /// Check if Tenkan is above Kijun (bullish signal)
    pub fn is_tenkan_above_kijun(&self) -> bool {
        self.tenkan > self.kijun
    }

    /// Get cloud thickness (Senkou A - Senkou B)
    pub fn cloud_thickness(&self) -> f64 {
        (self.senkou_a - self.senkou_b).abs()
    }

    /// Check if cloud is bullish (Senkou A > Senkou B)
    pub fn is_cloud_bullish(&self) -> bool {
        self.senkou_a > self.senkou_b
    }
}

/// Ichimoku Cloud (Ichimoku Kinko Hyo)
///
/// A comprehensive indicator developed by Goichi Hosoda that shows
/// support/resistance, trend direction, and momentum at a glance.
///
/// Components:
/// - Tenkan-sen (Conversion Line): Short-term trend
/// - Kijun-sen (Base Line): Medium-term trend
/// - Senkou Span A/B: Cloud boundaries (future)
/// - Chikou Span: Lagging line (past)
///
/// Default periods: 9, 26, 52
#[derive(Clone)]
pub struct Ichimoku {
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    high_buffer: Vec<f64>,
    low_buffer: Vec<f64>,
    close_buffer: Vec<f64>,
    index: usize,
    count: usize,
    current: Option<IchimokuOutput>,
}

impl Ichimoku {
    pub fn new(tenkan_period: usize, kijun_period: usize, senkou_b_period: usize) -> Self {
        assert!(tenkan_period > 0, "tenkan_period must be > 0");
        assert!(kijun_period > 0, "kijun_period must be > 0");
        assert!(senkou_b_period > 0, "senkou_b_period must be > 0");

        let max_period = senkou_b_period.max(kijun_period);

        Self {
            tenkan_period,
            kijun_period,
            senkou_b_period,
            high_buffer: vec![f64::NEG_INFINITY; max_period],
            low_buffer: vec![f64::INFINITY; max_period],
            close_buffer: vec![0.0; kijun_period],
            index: 0,
            count: 0,
            current: None,
        }
    }

    fn midpoint(&self, period: usize) -> f64 {
        let mut high = f64::NEG_INFINITY;
        let mut low = f64::INFINITY;

        let start = if self.count >= period {
            (self.index + self.high_buffer.len() - period) % self.high_buffer.len()
        } else {
            0
        };

        for i in 0..period.min(self.count) {
            let idx = (start + i) % self.high_buffer.len();
            if self.high_buffer[idx] > high {
                high = self.high_buffer[idx];
            }
            if self.low_buffer[idx] < low {
                low = self.low_buffer[idx];
            }
        }

        (high + low) / 2.0
    }
}

impl Next<&Ohlcv> for Ichimoku {
    type Output = IchimokuOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<IchimokuOutput> {
        let max_period = self.senkou_b_period.max(self.kijun_period);

        // Store values
        self.high_buffer[self.index] = candle.high;
        self.low_buffer[self.index] = candle.low;

        // Store close for Chikou (we use kijun_period for the lagging line)
        let close_idx = self.index % self.kijun_period;
        let chikou_idx = (close_idx + 1) % self.kijun_period;
        let old_close = self.close_buffer[chikou_idx];
        self.close_buffer[close_idx] = candle.close;

        self.index = (self.index + 1) % max_period;
        self.count += 1;

        // Need enough data for Senkou B
        if self.count < self.senkou_b_period {
            return None;
        }

        let tenkan = self.midpoint(self.tenkan_period);
        let kijun = self.midpoint(self.kijun_period);
        let senkou_a = (tenkan + kijun) / 2.0;
        let senkou_b = self.midpoint(self.senkou_b_period);

        // Chikou is the close from kijun_period bars ago
        let chikou = if self.count >= self.kijun_period {
            old_close
        } else {
            candle.close
        };

        let output = IchimokuOutput::new(tenkan, kijun, senkou_a, senkou_b, chikou);
        self.current = Some(output);
        Some(output)
    }
}

impl Reset for Ichimoku {
    fn reset(&mut self) {
        self.high_buffer.fill(f64::NEG_INFINITY);
        self.low_buffer.fill(f64::INFINITY);
        self.close_buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.current = None;
    }
}

impl Period for Ichimoku {
    fn period(&self) -> usize {
        self.senkou_b_period
    }
}

impl Current for Ichimoku {
    type Output = IchimokuOutput;

    fn current(&self) -> Option<IchimokuOutput> {
        self.current
    }
}

impl Default for Ichimoku {
    fn default() -> Self {
        Self::new(9, 26, 52)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_ichimoku_basic() {
        let mut ich = Ichimoku::new(9, 26, 52);

        // Feed enough data
        for i in 0..60 {
            ich.next(&make_candle(100.0 + i as f64, 95.0 + i as f64, 98.0 + i as f64));
        }

        assert!(ich.current().is_some());
    }

    #[test]
    fn test_ichimoku_signals() {
        let output = IchimokuOutput::new(100.0, 98.0, 99.0, 97.0, 95.0);

        assert!(output.is_bullish(105.0));
        assert!(output.is_bearish(90.0));
        assert!(output.is_in_cloud(98.0));
        assert!(output.is_tenkan_above_kijun());
        assert!(output.is_cloud_bullish());
    }

    #[test]
    fn test_ichimoku_reset() {
        let mut ich = Ichimoku::default();

        for i in 0..60 {
            ich.next(&make_candle(100.0 + i as f64, 95.0, 98.0));
        }

        ich.reset();
        assert!(ich.current().is_none());
    }
}
