//! Volume Oscillator

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume Oscillator
///
/// Shows the relationship between two volume moving averages as a percentage.
/// Helps identify volume trends and potential reversals based on volume changes.
///
/// # Formula
///
/// ```text
/// VO = ((Fast Volume MA - Slow Volume MA) / Slow Volume MA) * 100
/// ```
///
/// # Interpretation
///
/// - **Positive VO**: Short-term volume > long-term volume (increasing volume)
/// - **Negative VO**: Short-term volume < long-term volume (decreasing volume)
/// - **Rising VO**: Volume trend strengthening
/// - **Falling VO**: Volume trend weakening
/// - **VO crossing zero**: Volume trend reversal
///
/// # Trading Signals
///
/// - VO > 0 with price rising: Volume-confirmed uptrend
/// - VO < 0 with price falling: Volume-confirmed downtrend
/// - VO divergence from price: Potential reversal warning
///
/// # Parameters
///
/// * `fast_period` - Fast moving average period (default: 5)
/// * `slow_period` - Slow moving average period (default: 10)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{VolumeOscillator, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut vo = VolumeOscillator::new(5, 10);
///
/// for candle in candles.iter() {
///     if let Some(value) = vo.next(candle) {
///         println!("Volume Oscillator: {:.2}%", value);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct VolumeOscillator {
    fast_period: usize,
    slow_period: usize,
    fast_volumes: VecDeque<f64>,
    slow_volumes: VecDeque<f64>,
    fast_sum: f64,
    slow_sum: f64,
    current: Option<f64>,
}

impl VolumeOscillator {
    /// Create a new Volume Oscillator with specified periods
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period > 0, "Fast period must be greater than 0");
        assert!(slow_period > 0, "Slow period must be greater than 0");
        assert!(
            fast_period < slow_period,
            "Fast period must be less than slow period"
        );

        Self {
            fast_period,
            slow_period,
            fast_volumes: VecDeque::with_capacity(fast_period),
            slow_volumes: VecDeque::with_capacity(slow_period),
            fast_sum: 0.0,
            slow_sum: 0.0,
            current: None,
        }
    }

    fn calculate(&self, fast_ma: f64, slow_ma: f64) -> f64 {
        if slow_ma == 0.0 {
            return 0.0;
        }
        ((fast_ma - slow_ma) / slow_ma) * 100.0
    }
}

impl Period for VolumeOscillator {
    fn period(&self) -> usize {
        self.slow_period
    }
}

impl Next<&Ohlcv> for VolumeOscillator {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        let volume = candle.volume;

        // Update fast MA
        self.fast_volumes.push_back(volume);
        self.fast_sum += volume;
        if self.fast_volumes.len() > self.fast_period {
            let old = self.fast_volumes.pop_front().unwrap();
            self.fast_sum -= old;
        }

        // Update slow MA
        self.slow_volumes.push_back(volume);
        self.slow_sum += volume;
        if self.slow_volumes.len() > self.slow_period {
            let old = self.slow_volumes.pop_front().unwrap();
            self.slow_sum -= old;
        }

        // Need both MAs to be ready
        if self.fast_volumes.len() >= self.fast_period
            && self.slow_volumes.len() >= self.slow_period
        {
            let fast_ma = self.fast_sum / self.fast_period as f64;
            let slow_ma = self.slow_sum / self.slow_period as f64;
            let vo = self.calculate(fast_ma, slow_ma);
            self.current = Some(vo);
            Some(vo)
        } else {
            None
        }
    }
}

impl Next<Ohlcv> for VolumeOscillator {
    type Output = f64;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for VolumeOscillator {
    type Output = f64;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for VolumeOscillator {
    fn reset(&mut self) {
        self.fast_volumes.clear();
        self.slow_volumes.clear();
        self.fast_sum = 0.0;
        self.slow_sum = 0.0;
        self.current = None;
    }
}

impl Default for VolumeOscillator {
    fn default() -> Self {
        Self::new(5, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(volume: f64) -> Ohlcv {
        Ohlcv::new(100.0, 100.0, 100.0, 100.0, volume)
    }

    #[test]
    fn test_volume_oscillator_new() {
        let vo = VolumeOscillator::new(5, 10);
        assert_eq!(vo.fast_period, 5);
        assert_eq!(vo.slow_period, 10);
        assert_eq!(vo.period(), 10);
    }

    #[test]
    #[should_panic]
    fn test_volume_oscillator_invalid_fast_period() {
        VolumeOscillator::new(0, 10);
    }

    #[test]
    #[should_panic]
    fn test_volume_oscillator_invalid_slow_period() {
        VolumeOscillator::new(5, 0);
    }

    #[test]
    #[should_panic]
    fn test_volume_oscillator_fast_not_less_than_slow() {
        VolumeOscillator::new(10, 5);
    }

    #[test]
    fn test_volume_oscillator_constant_volume() {
        let mut vo = VolumeOscillator::new(3, 5);

        // With constant volume, oscillator should be near zero
        for _ in 0..10 {
            vo.next(&candle(1000.0));
        }

        let result = vo.current().unwrap();
        assert!(result.abs() < 1e-10, "VO should be ~0 with constant volume");
    }

    #[test]
    fn test_volume_oscillator_increasing_volume() {
        let mut vo = VolumeOscillator::new(3, 5);

        // Increasing volume should give positive VO
        for i in 1..=10 {
            vo.next(&candle(i as f64 * 100.0));
        }

        let result = vo.current().unwrap();
        assert!(result > 0.0, "VO should be positive with increasing volume");
    }

    #[test]
    fn test_volume_oscillator_decreasing_volume() {
        let mut vo = VolumeOscillator::new(3, 5);

        // Decreasing volume should give negative VO
        for i in (1..=10).rev() {
            vo.next(&candle(i as f64 * 100.0));
        }

        let result = vo.current().unwrap();
        assert!(result < 0.0, "VO should be negative with decreasing volume");
    }

    #[test]
    fn test_volume_oscillator_calculation() {
        let mut vo = VolumeOscillator::new(2, 3);

        vo.next(&candle(100.0));
        vo.next(&candle(200.0));
        let result = vo.next(&candle(300.0));

        // Fast MA (last 2): (200 + 300) / 2 = 250
        // Slow MA (last 3): (100 + 200 + 300) / 3 = 200
        // VO = ((250 - 200) / 200) * 100 = 25%

        assert!(result.is_some());
        assert!((result.unwrap() - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_volume_oscillator_spike() {
        let mut vo = VolumeOscillator::new(3, 5);

        // Build base
        for _ in 0..10 {
            vo.next(&candle(1000.0));
        }

        // Volume spike
        vo.next(&candle(5000.0));

        let result = vo.current().unwrap();
        assert!(result > 0.0, "Volume spike should create positive VO");
    }

    #[test]
    fn test_volume_oscillator_zero_slow_ma() {
        let mut vo = VolumeOscillator::new(2, 3);

        // Zero volumes
        vo.next(&candle(0.0));
        vo.next(&candle(0.0));
        let result = vo.next(&candle(0.0));

        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_volume_oscillator_reset() {
        let mut vo = VolumeOscillator::new(3, 5);

        for i in 1..=10 {
            vo.next(&candle(i as f64 * 100.0));
        }

        vo.reset();

        assert!(vo.current().is_none());
        assert_eq!(vo.fast_volumes.len(), 0);
        assert_eq!(vo.slow_volumes.len(), 0);
        assert_eq!(vo.fast_sum, 0.0);
        assert_eq!(vo.slow_sum, 0.0);
    }

    #[test]
    fn test_volume_oscillator_default() {
        let vo = VolumeOscillator::default();
        assert_eq!(vo.fast_period, 5);
        assert_eq!(vo.slow_period, 10);
    }

    #[test]
    fn test_volume_oscillator_insufficient_data() {
        let mut vo = VolumeOscillator::new(3, 5);

        assert!(vo.next(&candle(100.0)).is_none());
        assert!(vo.next(&candle(200.0)).is_none());
        assert!(vo.next(&candle(300.0)).is_none());
        assert!(vo.next(&candle(400.0)).is_none());

        // 5th value should return Some (slow period reached)
        assert!(vo.next(&candle(500.0)).is_some());
    }

    #[test]
    fn test_volume_oscillator_crossover() {
        let mut vo = VolumeOscillator::new(2, 4);

        // Start with declining volume
        vo.next(&candle(1000.0));
        vo.next(&candle(900.0));
        vo.next(&candle(800.0));
        vo.next(&candle(700.0));

        let before = vo.current().unwrap();
        assert!(before < 0.0);

        // Sharp volume increase
        vo.next(&candle(1500.0));
        vo.next(&candle(1600.0));

        let after = vo.current().unwrap();
        assert!(after > 0.0, "VO should cross zero with volume reversal");
    }

    #[test]
    fn test_volume_oscillator_rolling_window() {
        let mut vo = VolumeOscillator::new(2, 3);

        vo.next(&candle(100.0));
        vo.next(&candle(200.0));
        vo.next(&candle(300.0));

        // Add 4th value
        let result = vo.next(&candle(400.0));

        // Fast MA (last 2): (300 + 400) / 2 = 350
        // Slow MA (last 3): (200 + 300 + 400) / 3 = 300
        // VO = ((350 - 300) / 300) * 100 = 16.666...

        let expected = ((350.0 - 300.0) / 300.0) * 100.0;
        assert!((result.unwrap() - expected).abs() < 1e-10);
    }
}
