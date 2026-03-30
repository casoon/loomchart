//! Klinger Volume Oscillator
//!
//! A volume-based momentum indicator that measures the difference between
//! two exponential moving averages of volume force. The volume force is
//! calculated based on the relationship between price movement and volume.
//!
//! # Formula
//!
//! ```text
//! Typical Price = (High + Low + Close) / 3
//! Money Flow Multiplier = ((Close - Low) - (High - Close)) / (High - Low)
//! Money Flow Volume = Money Flow Multiplier * Volume
//! Volume Force = Money Flow Volume * Trend (where Trend is +1 or -1)
//! KVO = EMA(Volume Force, fast_period) - EMA(Volume Force, slow_period)
//! Signal = EMA(KVO, signal_period)
//! ```
//!
//! # Examples
//!
//! ```
//! use loom_indicators::prelude::*;
//! use loom_indicators::indicators::volume::Klinger;
//!
//! let mut klinger = Klinger::new(34, 55, 13);
//!
//! let candle1 = Ohlcv { open: 100.0, high: 102.0, low: 99.0, close: 101.0, volume: 1000.0 };
//! let candle2 = Ohlcv { open: 101.0, high: 103.0, low: 100.0, close: 102.0, volume: 1200.0 };
//!
//! let result1 = klinger.next(&candle1);
//! let result2 = klinger.next(&candle2);
//! ```

use crate::indicators::trend::Ema;
use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;
#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(feature = "std")]
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Klinger Volume Oscillator output
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KlingerOutput {
    /// The KVO value
    pub kvo: f64,
    /// The signal line (EMA of KVO)
    pub signal: f64,
    /// The histogram (KVO - Signal)
    pub histogram: f64,
}

/// Klinger Volume Oscillator
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Klinger {
    #[allow(dead_code)]
    fast_period: usize,
    slow_period: usize,
    #[allow(dead_code)]
    signal_period: usize,
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
    prev_hlc: Option<f64>,
    trend: i8,
    current: Option<KlingerOutput>,
}

impl Klinger {
    /// Creates a new Klinger Volume Oscillator
    ///
    /// # Arguments
    ///
    /// * `fast_period` - Fast EMA period (default: 34)
    /// * `slow_period` - Slow EMA period (default: 55)
    /// * `signal_period` - Signal line EMA period (default: 13)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if:
    /// - Any period is 0
    /// - Fast period >= Slow period
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        assert!(
            fast_period > 0 && slow_period > 0 && signal_period > 0,
            "periods must be greater than 0"
        );
        assert!(
            fast_period < slow_period,
            "fast_period must be less than slow_period"
        );

        Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
            prev_hlc: None,
            trend: 1,
            current: None,
        }
    }

    /// Calculate volume force from OHLCV data
    fn calculate_volume_force(&mut self, candle: &Ohlcv) -> f64 {
        let hlc = (candle.high + candle.low + candle.close) / 3.0;

        // Determine trend
        if let Some(prev_hlc) = self.prev_hlc {
            if hlc > prev_hlc {
                self.trend = 1;
            } else if hlc < prev_hlc {
                self.trend = -1;
            }
            // else trend stays the same
        }

        self.prev_hlc = Some(hlc);

        // Calculate money flow multiplier
        let range = candle.high - candle.low;
        let dm = if range > 0.0 {
            ((candle.close - candle.low) - (candle.high - candle.close)) / range
        } else {
            0.0
        };

        // Calculate volume force
        let cm = dm * candle.volume;
        cm * self.trend as f64
    }
}

impl Period for Klinger {
    fn period(&self) -> usize {
        self.slow_period
    }
}

impl Next<&Ohlcv> for Klinger {
    type Output = KlingerOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        let vf = self.calculate_volume_force(candle);

        // Update EMAs
        let fast_ema_val = self.fast_ema.next(vf)?;
        let slow_ema_val = self.slow_ema.next(vf)?;

        // Calculate KVO
        let kvo = fast_ema_val - slow_ema_val;

        // Calculate signal line
        let signal = self.signal_ema.next(kvo)?;

        // Calculate histogram
        let histogram = kvo - signal;

        let output = KlingerOutput {
            kvo,
            signal,
            histogram,
        };

        self.current = Some(output.clone());
        Some(output)
    }
}

impl<'a> Next<&'a Ohlcv> for Box<Klinger> {
    type Output = KlingerOutput;

    fn next(&mut self, input: &'a Ohlcv) -> Option<Self::Output> {
        (**self).next(input)
    }
}

impl Reset for Klinger {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.prev_hlc = None;
        self.trend = 1;
        self.current = None;
    }
}

impl Current for Klinger {
    type Output = KlingerOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current.clone()
    }
}

impl Default for Klinger {
    fn default() -> Self {
        Self::new(34, 55, 13)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let klinger = Klinger::new(34, 55, 13);
        assert_eq!(klinger.period(), 55);
    }

    #[test]
    #[should_panic(expected = "periods must be greater than 0")]
    fn test_new_invalid_zero_period() {
        Klinger::new(0, 55, 13);
    }

    #[test]
    #[should_panic(expected = "fast_period must be less than slow_period")]
    fn test_new_invalid_fast_slow() {
        Klinger::new(55, 34, 13);
    }

    #[test]
    fn test_default() {
        let klinger = Klinger::default();
        assert_eq!(klinger.fast_period, 34);
        assert_eq!(klinger.slow_period, 55);
        assert_eq!(klinger.signal_period, 13);
    }

    #[test]
    fn test_uptrend_with_increasing_volume() {
        let mut klinger = Klinger::new(5, 10, 3);

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 1200.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 2800.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // In an uptrend with increasing volume, KVO should be positive
        assert!(result.kvo > 0.0);
    }

    #[test]
    fn test_downtrend_with_increasing_volume() {
        let mut klinger = Klinger::new(5, 10, 3);

        let candles = vec![
            Ohlcv {
                open: 110.0,
                high: 111.0,
                low: 108.0,
                close: 109.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 109.0,
                high: 110.0,
                low: 107.0,
                close: 108.0,
                volume: 1200.0,
            },
            Ohlcv {
                open: 108.0,
                high: 109.0,
                low: 106.0,
                close: 107.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 107.0,
                high: 108.0,
                low: 105.0,
                close: 106.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 106.0,
                high: 107.0,
                low: 104.0,
                close: 105.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 106.0,
                low: 103.0,
                close: 104.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 104.0,
                high: 105.0,
                low: 102.0,
                close: 103.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 103.0,
                high: 104.0,
                low: 101.0,
                close: 102.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 102.0,
                high: 103.0,
                low: 100.0,
                close: 101.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 101.0,
                high: 102.0,
                low: 99.0,
                close: 100.0,
                volume: 2800.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // In a downtrend with increasing volume, KVO should be negative
        assert!(result.kvo < 0.0);
    }

    #[test]
    fn test_signal_crossover() {
        let mut klinger = Klinger::new(5, 10, 3);

        // Start with uptrend
        let uptrend_candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 2800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 3000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 3200.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 3400.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 3600.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 3800.0,
            },
        ];

        for candle in &uptrend_candles {
            klinger.next(candle);
        }

        // Get result after uptrend - histogram should be positive
        let uptrend_result = klinger.current().unwrap();
        assert!(uptrend_result.histogram > 0.0);

        // Now reverse with downtrend
        let downtrend_candles = vec![
            Ohlcv {
                open: 110.0,
                high: 111.0,
                low: 108.0,
                close: 109.0,
                volume: 4000.0,
            },
            Ohlcv {
                open: 109.0,
                high: 110.0,
                low: 107.0,
                close: 108.0,
                volume: 4200.0,
            },
            Ohlcv {
                open: 108.0,
                high: 109.0,
                low: 106.0,
                close: 107.0,
                volume: 4400.0,
            },
            Ohlcv {
                open: 107.0,
                high: 108.0,
                low: 105.0,
                close: 106.0,
                volume: 4600.0,
            },
            Ohlcv {
                open: 106.0,
                high: 107.0,
                low: 104.0,
                close: 105.0,
                volume: 4800.0,
            },
        ];

        for candle in &downtrend_candles {
            klinger.next(candle);
        }

        // Get result after downtrend - histogram should turn negative
        let downtrend_result = klinger.current().unwrap();
        assert!(downtrend_result.histogram < 0.0);
    }

    #[test]
    fn test_zero_volume() {
        let mut klinger = Klinger::new(5, 10, 3);

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 0.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 0.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With zero volume, all values should be zero
        assert_eq!(result.kvo, 0.0);
        assert_eq!(result.signal, 0.0);
        assert_eq!(result.histogram, 0.0);
    }

    #[test]
    fn test_constant_price() {
        let mut klinger = Klinger::new(5, 10, 3);

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With constant price and zero range, volume force should be zero
        assert_eq!(result.kvo, 0.0);
        assert_eq!(result.signal, 0.0);
        assert_eq!(result.histogram, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut klinger = Klinger::new(5, 10, 3);

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 1200.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 2800.0,
            },
        ];

        for candle in &candles {
            klinger.next(candle);
        }

        let result_before = klinger.current().unwrap();

        klinger.reset();
        assert!(klinger.current().is_none());

        for candle in &candles {
            klinger.next(candle);
        }

        let result_after = klinger.current().unwrap();
        assert_eq!(result_before.kvo, result_after.kvo);
        assert_eq!(result_before.signal, result_after.signal);
        assert_eq!(result_before.histogram, result_after.histogram);
    }

    #[test]
    fn test_current() {
        let mut klinger = Klinger::new(5, 10, 3);
        assert!(klinger.current().is_none());

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 1200.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 2800.0,
            },
        ];

        for candle in &candles {
            klinger.next(candle);
        }

        assert!(klinger.current().is_some());
    }

    #[test]
    fn test_histogram_calculation() {
        let mut klinger = Klinger::new(5, 10, 3);

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 1200.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 2800.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        let result = last_result.unwrap();

        // Verify histogram is KVO - Signal
        assert!((result.histogram - (result.kvo - result.signal)).abs() < 1e-10);
    }

    #[test]
    fn test_divergence_detection_setup() {
        let mut klinger = Klinger::new(5, 10, 3);

        // Rising prices with declining volume (bearish divergence setup)
        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 3000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 2800.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 1200.0,
            },
        ];

        let mut kvo_values = Vec::new();
        for candle in &candles {
            if let Some(result) = klinger.next(candle) {
                kvo_values.push(result.kvo);
            }
        }

        // With declining volume in uptrend, KVO should decrease over time
        assert!(kvo_values.len() > 2);
        let first_kvo = kvo_values[0];
        let last_kvo = *kvo_values.last().unwrap();

        // Last KVO should be less than first (declining momentum)
        assert!(last_kvo < first_kvo);
    }

    #[test]
    fn test_volume_force_trend_persistence() {
        let mut klinger = Klinger::new(5, 10, 3);

        // Sideways price with same HLC
        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
            Ohlcv {
                open: 100.5,
                high: 102.5,
                low: 99.5,
                close: 100.5,
                volume: 1000.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        // With sideways price, KVO should be close to zero
        assert!(last_result.is_some());
        let result = last_result.unwrap();
        assert!(result.kvo.abs() < 100.0); // Should be relatively small
    }

    #[test]
    fn test_boxed() {
        let mut klinger: Box<Klinger> = Box::new(Klinger::new(5, 10, 3));

        let candles = vec![
            Ohlcv {
                open: 100.0,
                high: 102.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
            },
            Ohlcv {
                open: 101.0,
                high: 103.0,
                low: 100.0,
                close: 102.0,
                volume: 1200.0,
            },
            Ohlcv {
                open: 102.0,
                high: 104.0,
                low: 101.0,
                close: 103.0,
                volume: 1400.0,
            },
            Ohlcv {
                open: 103.0,
                high: 105.0,
                low: 102.0,
                close: 104.0,
                volume: 1600.0,
            },
            Ohlcv {
                open: 104.0,
                high: 106.0,
                low: 103.0,
                close: 105.0,
                volume: 1800.0,
            },
            Ohlcv {
                open: 105.0,
                high: 107.0,
                low: 104.0,
                close: 106.0,
                volume: 2000.0,
            },
            Ohlcv {
                open: 106.0,
                high: 108.0,
                low: 105.0,
                close: 107.0,
                volume: 2200.0,
            },
            Ohlcv {
                open: 107.0,
                high: 109.0,
                low: 106.0,
                close: 108.0,
                volume: 2400.0,
            },
            Ohlcv {
                open: 108.0,
                high: 110.0,
                low: 107.0,
                close: 109.0,
                volume: 2600.0,
            },
            Ohlcv {
                open: 109.0,
                high: 111.0,
                low: 108.0,
                close: 110.0,
                volume: 2800.0,
            },
        ];

        let mut last_result = None;
        for candle in &candles {
            last_result = klinger.next(candle);
        }

        assert!(last_result.is_some());
    }
}
