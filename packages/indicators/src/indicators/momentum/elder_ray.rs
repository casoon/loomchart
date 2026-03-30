//! Elder Ray Index

use crate::indicators::trend::Ema;
use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Elder Ray Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ElderRayOutput {
    /// Bull Power (High - EMA)
    pub bull_power: f64,
    /// Bear Power (Low - EMA)
    pub bear_power: f64,
    /// EMA value
    pub ema: f64,
}

impl ElderRayOutput {
    pub fn new(bull_power: f64, bear_power: f64, ema: f64) -> Self {
        Self {
            bull_power,
            bear_power,
            ema,
        }
    }

    /// Check if bulls are in control (both positive)
    pub fn is_bullish(&self) -> bool {
        self.bull_power > 0.0 && self.bear_power > 0.0
    }

    /// Check if bears are in control (both negative)
    pub fn is_bearish(&self) -> bool {
        self.bull_power < 0.0 && self.bear_power < 0.0
    }

    /// Check if strong bullish divergence (bull power rising, bear power declining)
    pub fn is_strong_bullish(&self, prev: &ElderRayOutput) -> bool {
        self.bull_power > prev.bull_power && self.bear_power > prev.bear_power
    }

    /// Check if strong bearish divergence
    pub fn is_strong_bearish(&self, prev: &ElderRayOutput) -> bool {
        self.bull_power < prev.bull_power && self.bear_power < prev.bear_power
    }
}

/// Elder Ray Index
///
/// Developed by Dr. Alexander Elder, this indicator measures the power of bulls
/// and bears in the market. It uses an EMA to represent the market consensus
/// and measures how far highs and lows deviate from it.
///
/// # Components
///
/// - **Bull Power**: High - EMA (measures buyers' strength)
/// - **Bear Power**: Low - EMA (measures sellers' strength)
///
/// # Interpretation
///
/// - **Bull Power > 0**: Bulls pushed price above consensus (bullish)
/// - **Bear Power < 0**: Bears pushed price below consensus (bearish)
/// - **Both positive**: Strong uptrend
/// - **Both negative**: Strong downtrend
/// - **Divergences**: Can signal trend reversals
///
/// # Trading Signals
///
/// - Buy: Bear Power negative but rising, Bull Power positive
/// - Sell: Bull Power positive but falling, Bear Power negative
/// - Strong Buy: Both rising from negative territory
/// - Strong Sell: Both falling from positive territory
///
/// # Parameters
///
/// * `period` - EMA period (default: 13)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{ElderRay, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut elder = ElderRay::new(13);
///
/// for candle in candles.iter() {
///     if let Some(output) = elder.next(candle) {
///         println!("Bull Power: {:.2}, Bear Power: {:.2}",
///                  output.bull_power, output.bear_power);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct ElderRay {
    period: usize,
    ema: Ema,
    current: Option<ElderRayOutput>,
    prev: Option<ElderRayOutput>,
}

impl ElderRay {
    /// Create a new Elder Ray Index with the specified EMA period
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");

        Self {
            period,
            ema: Ema::new(period),
            current: None,
            prev: None,
        }
    }

    /// Get previous output for divergence detection
    pub fn previous(&self) -> Option<ElderRayOutput> {
        self.prev
    }
}

impl Period for ElderRay {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<&Ohlcv> for ElderRay {
    type Output = ElderRayOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        let ema_opt = self.ema.next(candle.close);

        if let Some(ema) = ema_opt {
            let bull_power = candle.high - ema;
            let bear_power = candle.low - ema;

            let output = ElderRayOutput::new(bull_power, bear_power, ema);

            self.prev = self.current;
            self.current = Some(output);

            Some(output)
        } else {
            None
        }
    }
}

impl Next<Ohlcv> for ElderRay {
    type Output = ElderRayOutput;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for ElderRay {
    type Output = ElderRayOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for ElderRay {
    fn reset(&mut self) {
        self.ema.reset();
        self.current = None;
        self.prev = None;
    }
}

impl Default for ElderRay {
    fn default() -> Self {
        Self::new(13)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_elder_ray_new() {
        let elder = ElderRay::new(13);
        assert_eq!(elder.period(), 13);
    }

    #[test]
    #[should_panic]
    fn test_elder_ray_invalid_period() {
        ElderRay::new(0);
    }

    #[test]
    fn test_elder_ray_uptrend() {
        let mut elder = ElderRay::new(5);

        // Strong uptrend
        for i in 0..20 {
            let base = 100.0 + i as f64 * 2.0;
            elder.next(&candle(base + 3.0, base - 1.0, base + 1.0));
        }

        let result = elder.current().unwrap();

        // In uptrend, both should be positive
        assert!(result.is_bullish(), "Should be bullish in uptrend");
    }

    #[test]
    fn test_elder_ray_downtrend() {
        let mut elder = ElderRay::new(5);

        // Strong downtrend
        for i in 0..20 {
            let base = 150.0 - i as f64 * 2.0;
            elder.next(&candle(base + 1.0, base - 3.0, base - 1.0));
        }

        let result = elder.current().unwrap();

        // In downtrend, both should be negative
        assert!(result.is_bearish(), "Should be bearish in downtrend");
    }

    #[test]
    fn test_elder_ray_calculation() {
        let mut elder = ElderRay::new(3);

        // Build up EMA
        elder.next(&candle(105.0, 95.0, 100.0));
        elder.next(&candle(110.0, 100.0, 105.0));
        let result = elder.next(&candle(115.0, 105.0, 110.0));

        assert!(result.is_some());
        let output = result.unwrap();

        // Bull Power = High - EMA
        // Bear Power = Low - EMA
        assert!(output.bull_power > 0.0);
        assert!(output.ema > 0.0);
    }

    #[test]
    fn test_elder_ray_bull_power() {
        let mut elder = ElderRay::new(3);

        for _ in 0..5 {
            elder.next(&candle(105.0, 95.0, 100.0));
        }

        let result = elder.current().unwrap();

        // High = 105, EMA should be around 100
        // Bull Power = 105 - EMA should be positive
        assert!(result.bull_power > 0.0);
    }

    #[test]
    fn test_elder_ray_bear_power() {
        let mut elder = ElderRay::new(3);

        for _ in 0..5 {
            elder.next(&candle(105.0, 95.0, 100.0));
        }

        let result = elder.current().unwrap();

        // Low = 95, EMA should be around 100
        // Bear Power = 95 - EMA should be negative
        assert!(result.bear_power < 0.0);
    }

    #[test]
    fn test_elder_ray_divergence() {
        let mut elder = ElderRay::new(5);

        // Build base
        for _ in 0..10 {
            elder.next(&candle(105.0, 95.0, 100.0));
        }

        let prev = elder.current().unwrap();

        // Increasing strength
        elder.next(&candle(110.0, 98.0, 105.0));
        let curr = elder.current().unwrap();

        // Check if strength is increasing
        assert!(curr.is_strong_bullish(&prev));
    }

    #[test]
    fn test_elder_ray_output_methods() {
        let output1 = ElderRayOutput::new(5.0, 2.0, 100.0);
        assert!(output1.is_bullish());
        assert!(!output1.is_bearish());

        let output2 = ElderRayOutput::new(-5.0, -2.0, 100.0);
        assert!(!output2.is_bullish());
        assert!(output2.is_bearish());

        let output3 = ElderRayOutput::new(5.0, -2.0, 100.0);
        assert!(!output3.is_bullish());
        assert!(!output3.is_bearish());
    }

    #[test]
    fn test_elder_ray_reset() {
        let mut elder = ElderRay::new(5);

        elder.next(&candle(105.0, 95.0, 100.0));
        elder.next(&candle(110.0, 100.0, 105.0));
        assert!(elder.current().is_some());

        elder.reset();

        assert!(elder.current().is_none());
        assert!(elder.previous().is_none());
    }

    #[test]
    fn test_elder_ray_default() {
        let elder = ElderRay::default();
        assert_eq!(elder.period(), 13);
    }

    #[test]
    fn test_elder_ray_insufficient_data() {
        let mut elder = ElderRay::new(5);

        // First few values return None until EMA is ready
        assert!(elder.next(&candle(100.0, 95.0, 98.0)).is_none());
        assert!(elder.next(&candle(102.0, 97.0, 100.0)).is_none());
    }

    #[test]
    fn test_elder_ray_previous_tracking() {
        let mut elder = ElderRay::new(3);

        for _ in 0..5 {
            elder.next(&candle(105.0, 95.0, 100.0));
        }

        let first = elder.current().unwrap();

        elder.next(&candle(110.0, 100.0, 105.0));

        let prev = elder.previous().unwrap();
        assert_eq!(prev.bull_power, first.bull_power);
        assert_eq!(prev.bear_power, first.bear_power);
    }

    #[test]
    fn test_elder_ray_consolidation() {
        let mut elder = ElderRay::new(5);

        // Consolidating market (low volatility)
        for _ in 0..15 {
            elder.next(&candle(101.0, 99.0, 100.0));
        }

        let result = elder.current().unwrap();

        // In consolidation, powers should be small
        assert!(result.bull_power.abs() < 2.0);
        assert!(result.bear_power.abs() < 2.0);
    }

    #[test]
    fn test_elder_ray_strong_divergence() {
        let mut elder = ElderRay::new(5);

        for _ in 0..10 {
            elder.next(&candle(105.0, 95.0, 100.0));
        }

        let prev = elder.current().unwrap();

        // Weakening
        elder.next(&candle(103.0, 93.0, 98.0));
        let curr = elder.current().unwrap();

        assert!(curr.is_strong_bearish(&prev));
    }
}
