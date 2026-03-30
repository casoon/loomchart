//! Average Directional Index (ADX) and related indicators

use crate::indicators::{Next, Period, Reset, Current};
use crate::math::range::{true_range, directional_movement};
use crate::types::Ohlcv;
use libm::fabs;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// ADX Output with all components
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AdxOutput {
    /// ADX value (0-100)
    pub adx: f64,
    /// +DI value
    pub plus_di: f64,
    /// -DI value
    pub minus_di: f64,
}

impl AdxOutput {
    pub fn new(adx: f64, plus_di: f64, minus_di: f64) -> Self {
        Self { adx, plus_di, minus_di }
    }

    /// Check if trend is strong (ADX > 25)
    pub fn is_strong_trend(&self) -> bool {
        self.adx > 25.0
    }

    /// Check if bullish (+DI > -DI)
    pub fn is_bullish(&self) -> bool {
        self.plus_di > self.minus_di
    }

    /// Check if bearish (-DI > +DI)
    pub fn is_bearish(&self) -> bool {
        self.minus_di > self.plus_di
    }
}

/// Average Directional Index (ADX)
///
/// Developed by J. Welles Wilder, ADX measures trend strength regardless
/// of direction. It uses +DI and -DI (Directional Indicators) to determine
/// trend direction.
///
/// Values:
/// - 0-25: Weak or no trend
/// - 25-50: Strong trend
/// - 50-75: Very strong trend
/// - 75-100: Extremely strong trend
///
/// Default period: 14
#[derive(Clone)]
pub struct Adx {
    period: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    smoothed_plus_dm: f64,
    smoothed_minus_dm: f64,
    smoothed_tr: f64,
    dx_buffer: Vec<f64>,
    dx_index: usize,
    dx_count: usize,
    dx_sum: f64,
    adx: Option<f64>,
    count: usize,
    current: Option<AdxOutput>,
}

impl Adx {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            smoothed_plus_dm: 0.0,
            smoothed_minus_dm: 0.0,
            smoothed_tr: 0.0,
            dx_buffer: vec![0.0; period],
            dx_index: 0,
            dx_count: 0,
            dx_sum: 0.0,
            adx: None,
            count: 0,
            current: None,
        }
    }

    /// Get the current ADX value
    pub fn adx(&self) -> Option<f64> {
        self.current.map(|o| o.adx)
    }

    /// Get the current +DI value
    pub fn plus_di(&self) -> Option<f64> {
        self.current.map(|o| o.plus_di)
    }

    /// Get the current -DI value
    pub fn minus_di(&self) -> Option<f64> {
        self.current.map(|o| o.minus_di)
    }
}

impl Next<&Ohlcv> for Adx {
    type Output = AdxOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<AdxOutput> {
        let result = if let (Some(prev_high), Some(prev_low), Some(prev_close)) =
            (self.prev_high, self.prev_low, self.prev_close)
        {
            self.count += 1;

            // Calculate True Range and Directional Movement
            let tr = true_range(candle.high, candle.low, prev_close);
            let (plus_dm, minus_dm) = directional_movement(candle.high, candle.low, prev_high, prev_low);

            let p = self.period as f64;

            if self.count <= self.period {
                // Accumulating first period
                self.smoothed_tr += tr;
                self.smoothed_plus_dm += plus_dm;
                self.smoothed_minus_dm += minus_dm;

                if self.count == self.period {
                    // First DI values
                    let plus_di = if self.smoothed_tr > 0.0 {
                        100.0 * self.smoothed_plus_dm / self.smoothed_tr
                    } else {
                        0.0
                    };

                    let minus_di = if self.smoothed_tr > 0.0 {
                        100.0 * self.smoothed_minus_dm / self.smoothed_tr
                    } else {
                        0.0
                    };

                    // Calculate DX
                    let di_sum = plus_di + minus_di;
                    let dx = if di_sum > 0.0 {
                        100.0 * fabs(plus_di - minus_di) / di_sum
                    } else {
                        0.0
                    };

                    // Store DX for ADX calculation
                    self.dx_buffer[self.dx_index] = dx;
                    self.dx_sum += dx;
                    self.dx_index = (self.dx_index + 1) % self.period;
                    self.dx_count += 1;

                    None // Need more DX values for ADX
                } else {
                    None
                }
            } else {
                // Wilder's smoothing for subsequent values
                self.smoothed_tr = self.smoothed_tr - (self.smoothed_tr / p) + tr;
                self.smoothed_plus_dm = self.smoothed_plus_dm - (self.smoothed_plus_dm / p) + plus_dm;
                self.smoothed_minus_dm = self.smoothed_minus_dm - (self.smoothed_minus_dm / p) + minus_dm;

                // Calculate DI values
                let plus_di = if self.smoothed_tr > 0.0 {
                    100.0 * self.smoothed_plus_dm / self.smoothed_tr
                } else {
                    0.0
                };

                let minus_di = if self.smoothed_tr > 0.0 {
                    100.0 * self.smoothed_minus_dm / self.smoothed_tr
                } else {
                    0.0
                };

                // Calculate DX
                let di_sum = plus_di + minus_di;
                let dx = if di_sum > 0.0 {
                    100.0 * fabs(plus_di - minus_di) / di_sum
                } else {
                    0.0
                };

                // Update DX buffer for ADX
                if self.dx_count >= self.period {
                    self.dx_sum -= self.dx_buffer[self.dx_index];
                }
                self.dx_buffer[self.dx_index] = dx;
                self.dx_sum += dx;
                self.dx_index = (self.dx_index + 1) % self.period;
                if self.dx_count < self.period {
                    self.dx_count += 1;
                }

                // Calculate ADX
                if self.dx_count >= self.period {
                    let adx = match self.adx {
                        Some(prev_adx) => {
                            // Wilder's smoothing for ADX
                            (prev_adx * (p - 1.0) + dx) / p
                        }
                        None => {
                            // First ADX is SMA of DX
                            self.dx_sum / p
                        }
                    };
                    self.adx = Some(adx);

                    let output = AdxOutput::new(adx, plus_di, minus_di);
                    self.current = Some(output);
                    Some(output)
                } else {
                    None
                }
            }
        } else {
            None
        };

        // Store previous values
        self.prev_high = Some(candle.high);
        self.prev_low = Some(candle.low);
        self.prev_close = Some(candle.close);

        result
    }
}

impl Reset for Adx {
    fn reset(&mut self) {
        self.prev_high = None;
        self.prev_low = None;
        self.prev_close = None;
        self.smoothed_plus_dm = 0.0;
        self.smoothed_minus_dm = 0.0;
        self.smoothed_tr = 0.0;
        self.dx_buffer.fill(0.0);
        self.dx_index = 0;
        self.dx_count = 0;
        self.dx_sum = 0.0;
        self.adx = None;
        self.count = 0;
        self.current = None;
    }
}

impl Period for Adx {
    fn period(&self) -> usize {
        // ADX requires 2 * period - 1 bars
        2 * self.period - 1
    }
}

impl Current for Adx {
    type Output = AdxOutput;

    fn current(&self) -> Option<AdxOutput> {
        self.current
    }
}

impl Default for Adx {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_adx_warmup() {
        let mut adx = Adx::new(5);

        // Need 2 * period - 1 = 9 candles minimum
        for i in 0..8 {
            let result = adx.next(&make_candle(100.0 + i as f64, 102.0 + i as f64, 99.0 + i as f64, 101.0 + i as f64));
            assert!(result.is_none(), "Should not produce value at index {}", i);
        }

        // Should produce value after warmup
        for i in 8..15 {
            adx.next(&make_candle(100.0 + i as f64, 102.0 + i as f64, 99.0 + i as f64, 101.0 + i as f64));
        }

        assert!(adx.current().is_some());
    }

    #[test]
    fn test_adx_strong_uptrend() {
        let mut adx = Adx::new(5);

        // Create a strong uptrend
        for i in 0..20 {
            let base = 100.0 + (i as f64 * 2.0);
            adx.next(&make_candle(base, base + 3.0, base - 0.5, base + 2.5));
        }

        let output = adx.current().unwrap();
        assert!(output.plus_di > output.minus_di, "Should be bullish");
    }

    #[test]
    fn test_adx_reset() {
        let mut adx = Adx::new(5);

        for i in 0..15 {
            adx.next(&make_candle(100.0 + i as f64, 102.0, 99.0, 101.0));
        }

        adx.reset();

        assert!(adx.current().is_none());
        assert_eq!(adx.count, 0);
    }
}
