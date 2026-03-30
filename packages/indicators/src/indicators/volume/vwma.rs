//! Volume Weighted Moving Average (VWMA)

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Volume Weighted Moving Average (VWMA)
///
/// Similar to a Simple Moving Average (SMA), but weighted by volume.
/// Gives more weight to prices with higher volume, helping identify
/// volume-confirmed price movements.
///
/// # Formula
///
/// ```text
/// VWMA = Sum(Price * Volume) / Sum(Volume)
/// ```
///
/// Where:
/// - Price is typically the close price
/// - Volume is the trading volume for that period
/// - Sum is calculated over the specified period
///
/// # Interpretation
///
/// - VWMA > SMA: Higher volume at higher prices (bullish)
/// - VWMA < SMA: Higher volume at lower prices (bearish)
/// - Rising VWMA: Volume-confirmed uptrend
/// - Falling VWMA: Volume-confirmed downtrend
///
/// # Parameters
///
/// * `period` - Number of periods (default: 20)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{Vwma, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut vwma = Vwma::new(20);
///
/// for candle in candles.iter() {
///     if let Some(value) = vwma.next(candle) {
///         println!("VWMA: {:.2}", value);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Vwma {
    period: usize,
    prices: VecDeque<f64>,
    volumes: VecDeque<f64>,
    price_volume_sum: f64,
    volume_sum: f64,
    current: Option<f64>,
}

impl Vwma {
    /// Create a new VWMA with the specified period
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");

        Self {
            period,
            prices: VecDeque::with_capacity(period),
            volumes: VecDeque::with_capacity(period),
            price_volume_sum: 0.0,
            volume_sum: 0.0,
            current: None,
        }
    }

    fn calculate(&self) -> f64 {
        if self.volume_sum == 0.0 {
            return 0.0;
        }
        self.price_volume_sum / self.volume_sum
    }
}

impl Period for Vwma {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<&Ohlcv> for Vwma {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        let price = candle.close;
        let volume = candle.volume;

        // Add new price and volume
        self.prices.push_back(price);
        self.volumes.push_back(volume);
        self.price_volume_sum += price * volume;
        self.volume_sum += volume;

        // Remove old values if we exceed period
        if self.prices.len() > self.period {
            let old_price = self.prices.pop_front().unwrap();
            let old_volume = self.volumes.pop_front().unwrap();
            self.price_volume_sum -= old_price * old_volume;
            self.volume_sum -= old_volume;
        }

        // Calculate VWMA once we have enough data
        if self.prices.len() >= self.period {
            let vwma = self.calculate();
            self.current = Some(vwma);
            Some(vwma)
        } else {
            None
        }
    }
}

impl Next<Ohlcv> for Vwma {
    type Output = f64;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for Vwma {
    type Output = f64;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for Vwma {
    fn reset(&mut self) {
        self.prices.clear();
        self.volumes.clear();
        self.price_volume_sum = 0.0;
        self.volume_sum = 0.0;
        self.current = None;
    }
}

impl Default for Vwma {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(price: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(price, price, price, price, volume)
    }

    #[test]
    fn test_vwma_new() {
        let vwma = Vwma::new(20);
        assert_eq!(vwma.period(), 20);
        assert!(vwma.current().is_none());
    }

    #[test]
    #[should_panic]
    fn test_vwma_invalid_period() {
        Vwma::new(0);
    }

    #[test]
    fn test_vwma_equal_volumes() {
        let mut vwma = Vwma::new(3);

        // With equal volumes, VWMA should equal SMA
        vwma.next(&candle(100.0, 1000.0));
        vwma.next(&candle(110.0, 1000.0));
        let result = vwma.next(&candle(120.0, 1000.0));

        assert!(result.is_some());
        let vwma_value = result.unwrap();
        let sma_value = (100.0 + 110.0 + 120.0) / 3.0;

        assert!((vwma_value - sma_value).abs() < 1e-10);
    }

    #[test]
    fn test_vwma_higher_volume_at_higher_prices() {
        let mut vwma = Vwma::new(3);

        // Higher volume at higher prices should pull VWMA up
        vwma.next(&candle(100.0, 1000.0));
        vwma.next(&candle(110.0, 2000.0));
        let result = vwma.next(&candle(120.0, 3000.0));

        let vwma_value = result.unwrap();
        let sma_value = (100.0 + 110.0 + 120.0) / 3.0;

        // VWMA should be higher than SMA
        assert!(vwma_value > sma_value);
    }

    #[test]
    fn test_vwma_higher_volume_at_lower_prices() {
        let mut vwma = Vwma::new(3);

        // Higher volume at lower prices should pull VWMA down
        vwma.next(&candle(120.0, 1000.0));
        vwma.next(&candle(110.0, 2000.0));
        let result = vwma.next(&candle(100.0, 3000.0));

        let vwma_value = result.unwrap();
        let sma_value = (120.0 + 110.0 + 100.0) / 3.0;

        // VWMA should be lower than SMA
        assert!(vwma_value < sma_value);
    }

    #[test]
    fn test_vwma_calculation() {
        let mut vwma = Vwma::new(3);

        vwma.next(&candle(100.0, 1000.0));
        vwma.next(&candle(110.0, 2000.0));
        let result = vwma.next(&candle(120.0, 3000.0));

        // Manual calculation:
        // Sum(Price * Volume) = (100 * 1000) + (110 * 2000) + (120 * 3000)
        //                     = 100000 + 220000 + 360000 = 680000
        // Sum(Volume) = 1000 + 2000 + 3000 = 6000
        // VWMA = 680000 / 6000 = 113.333...

        let expected = 680000.0 / 6000.0;
        assert!((result.unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_vwma_rolling_window() {
        let mut vwma = Vwma::new(3);

        vwma.next(&candle(100.0, 1000.0));
        vwma.next(&candle(110.0, 2000.0));
        vwma.next(&candle(120.0, 3000.0));

        // Add 4th value, should remove first
        let result = vwma.next(&candle(130.0, 4000.0));

        // Should use last 3: 110, 120, 130
        // Sum(Price * Volume) = (110 * 2000) + (120 * 3000) + (130 * 4000)
        //                     = 220000 + 360000 + 520000 = 1100000
        // Sum(Volume) = 2000 + 3000 + 4000 = 9000
        // VWMA = 1100000 / 9000 = 122.222...

        let expected = 1100000.0 / 9000.0;
        assert!((result.unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_vwma_zero_volume() {
        let mut vwma = Vwma::new(3);

        vwma.next(&candle(100.0, 0.0));
        vwma.next(&candle(110.0, 0.0));
        let result = vwma.next(&candle(120.0, 0.0));

        // With zero volume, result should be 0
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_vwma_mixed_zero_volume() {
        let mut vwma = Vwma::new(3);

        vwma.next(&candle(100.0, 1000.0));
        vwma.next(&candle(110.0, 0.0));
        let result = vwma.next(&candle(120.0, 2000.0));

        // Should only weight non-zero volumes
        // Sum(Price * Volume) = (100 * 1000) + (110 * 0) + (120 * 2000)
        //                     = 100000 + 0 + 240000 = 340000
        // Sum(Volume) = 1000 + 0 + 2000 = 3000
        // VWMA = 340000 / 3000 = 113.333...

        let expected = 340000.0 / 3000.0;
        assert!((result.unwrap() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_vwma_reset() {
        let mut vwma = Vwma::new(3);

        vwma.next(&candle(100.0, 1000.0));
        vwma.next(&candle(110.0, 2000.0));
        vwma.next(&candle(120.0, 3000.0));

        vwma.reset();

        assert!(vwma.current().is_none());
        assert_eq!(vwma.prices.len(), 0);
        assert_eq!(vwma.volumes.len(), 0);
        assert_eq!(vwma.price_volume_sum, 0.0);
        assert_eq!(vwma.volume_sum, 0.0);
    }

    #[test]
    fn test_vwma_default() {
        let vwma = Vwma::default();
        assert_eq!(vwma.period(), 20);
    }

    #[test]
    fn test_vwma_insufficient_data() {
        let mut vwma = Vwma::new(5);

        assert!(vwma.next(&candle(100.0, 1000.0)).is_none());
        assert!(vwma.next(&candle(110.0, 2000.0)).is_none());
        assert!(vwma.next(&candle(120.0, 3000.0)).is_none());
        assert!(vwma.next(&candle(130.0, 4000.0)).is_none());

        // 5th value should return Some
        assert!(vwma.next(&candle(140.0, 5000.0)).is_some());
    }
}
