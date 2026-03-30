//! Historical Volatility (HV)

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Historical Volatility (HV)
///
/// Measures the dispersion of returns for a security over a given period.
/// It's the standard deviation of logarithmic returns, annualized.
///
/// # Formula
///
/// ```text
/// Log Return = ln(Price_t / Price_{t-1})
/// HV = StdDev(Log Returns) * sqrt(periods_per_year) * 100
/// ```
///
/// Where:
/// - ln is the natural logarithm
/// - StdDev is the standard deviation
/// - periods_per_year: 252 for daily, 52 for weekly, 12 for monthly
///
/// # Interpretation
///
/// - **High HV**: Large price swings, high uncertainty, higher risk
/// - **Low HV**: Small price movements, low uncertainty, lower risk
/// - **Rising HV**: Increasing market volatility (often in downtrends)
/// - **Falling HV**: Decreasing volatility (often in uptrends or consolidation)
///
/// # Common Values
///
/// - < 10%: Very low volatility
/// - 10-20%: Low to moderate volatility
/// - 20-30%: Moderate to high volatility
/// - > 30%: High volatility
///
/// # Parameters
///
/// * `period` - Lookback period (default: 20)
/// * `periods_per_year` - Annualization factor (default: 252 for daily data)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{HistoricalVolatility, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut hv = HistoricalVolatility::new(20, 252);
///
/// for candle in candles.iter() {
///     if let Some(volatility) = hv.next(candle) {
///         println!("Historical Volatility: {:.2}%", volatility);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct HistoricalVolatility {
    period: usize,
    periods_per_year: usize,
    returns: VecDeque<f64>,
    prev_close: Option<f64>,
    current: Option<f64>,
}

impl HistoricalVolatility {
    /// Create a new Historical Volatility indicator
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for calculation
    /// * `periods_per_year` - Annualization factor (252 for daily, 52 for weekly, 12 for monthly)
    pub fn new(period: usize, periods_per_year: usize) -> Self {
        assert!(period > 1, "Period must be greater than 1");
        assert!(
            periods_per_year > 0,
            "Periods per year must be greater than 0"
        );

        Self {
            period,
            periods_per_year,
            returns: VecDeque::with_capacity(period),
            prev_close: None,
            current: None,
        }
    }

    /// Calculate standard deviation of returns
    fn calculate_stddev(&self) -> f64 {
        if self.returns.len() < 2 {
            return 0.0;
        }

        let n = self.returns.len() as f64;
        let mean = self.returns.iter().sum::<f64>() / n;

        let variance = self
            .returns
            .iter()
            .map(|r| {
                let diff = r - mean;
                diff * diff
            })
            .sum::<f64>()
            / n;

        variance.sqrt()
    }

    /// Calculate annualized volatility percentage
    fn calculate_hv(&self) -> f64 {
        let stddev = self.calculate_stddev();
        let annualization_factor = (self.periods_per_year as f64).sqrt();
        stddev * annualization_factor * 100.0
    }
}

impl Period for HistoricalVolatility {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<&Ohlcv> for HistoricalVolatility {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        let close = candle.close;

        // Calculate log return if we have a previous close
        if let Some(prev) = self.prev_close {
            if prev > 0.0 && close > 0.0 {
                let log_return = (close / prev).ln();
                self.returns.push_back(log_return);

                // Remove old returns if we exceed period
                if self.returns.len() > self.period {
                    self.returns.pop_front();
                }
            }
        }

        self.prev_close = Some(close);

        // Need at least 2 returns to calculate volatility
        if self.returns.len() >= 2 {
            let hv = self.calculate_hv();
            self.current = Some(hv);
            Some(hv)
        } else {
            None
        }
    }
}

impl Next<Ohlcv> for HistoricalVolatility {
    type Output = f64;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for HistoricalVolatility {
    type Output = f64;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for HistoricalVolatility {
    fn reset(&mut self) {
        self.returns.clear();
        self.prev_close = None;
        self.current = None;
    }
}

impl Default for HistoricalVolatility {
    fn default() -> Self {
        Self::new(20, 252)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(price: f64) -> Ohlcv {
        Ohlcv::new(price, price, price, price, 1000.0)
    }

    #[test]
    fn test_historical_volatility_new() {
        let hv = HistoricalVolatility::new(20, 252);
        assert_eq!(hv.period(), 20);
        assert_eq!(hv.periods_per_year, 252);
    }

    #[test]
    #[should_panic]
    fn test_historical_volatility_invalid_period() {
        HistoricalVolatility::new(1, 252);
    }

    #[test]
    #[should_panic]
    fn test_historical_volatility_invalid_periods_per_year() {
        HistoricalVolatility::new(20, 0);
    }

    #[test]
    fn test_historical_volatility_flat_prices() {
        let mut hv = HistoricalVolatility::new(10, 252);

        // Flat prices should give very low volatility
        for _ in 0..15 {
            hv.next(&candle(100.0));
        }

        let result = hv.current().unwrap();
        assert!(result < 1.0, "Flat prices should have very low volatility");
    }

    #[test]
    fn test_historical_volatility_high_movement() {
        let mut hv = HistoricalVolatility::new(5, 252);

        // Large price swings
        let prices = vec![100.0, 110.0, 95.0, 115.0, 90.0, 120.0];

        for price in prices {
            hv.next(&candle(price));
        }

        let result = hv.current().unwrap();
        assert!(
            result > 10.0,
            "High price swings should give high volatility"
        );
    }

    #[test]
    fn test_historical_volatility_calculation() {
        let mut hv = HistoricalVolatility::new(3, 252);

        hv.next(&candle(100.0));
        hv.next(&candle(102.0));
        hv.next(&candle(101.0));
        let result = hv.next(&candle(103.0));

        // Should return Some after having enough data
        assert!(result.is_some());
        let volatility = result.unwrap();

        // Volatility should be positive
        assert!(volatility > 0.0);
        // And reasonable for small movements
        assert!(volatility < 100.0);
    }

    #[test]
    fn test_historical_volatility_increasing_trend() {
        let mut hv = HistoricalVolatility::new(10, 252);

        // Steady increasing trend with small volatility
        for i in 0..15 {
            hv.next(&candle(100.0 + i as f64));
        }

        let result = hv.current().unwrap();
        assert!(result > 0.0);
        // Steady trend should have moderate volatility
        assert!(result < 50.0);
    }

    #[test]
    fn test_historical_volatility_zero_price() {
        let mut hv = HistoricalVolatility::new(5, 252);

        hv.next(&candle(100.0));
        hv.next(&candle(0.0)); // Invalid price
        hv.next(&candle(102.0));

        // Should handle zero price gracefully
        // Returns won't be calculated for invalid prices
        assert!(hv.returns.len() <= 2);
    }

    #[test]
    fn test_historical_volatility_reset() {
        let mut hv = HistoricalVolatility::new(10, 252);

        for i in 0..15 {
            hv.next(&candle(100.0 + i as f64));
        }

        hv.reset();

        assert!(hv.current().is_none());
        assert_eq!(hv.returns.len(), 0);
        assert!(hv.prev_close.is_none());
    }

    #[test]
    fn test_historical_volatility_default() {
        let hv = HistoricalVolatility::default();
        assert_eq!(hv.period(), 20);
        assert_eq!(hv.periods_per_year, 252);
    }

    #[test]
    fn test_historical_volatility_insufficient_data() {
        let mut hv = HistoricalVolatility::new(10, 252);

        assert!(hv.next(&candle(100.0)).is_none());
        assert!(hv.next(&candle(101.0)).is_none()); // Only 1 return

        // 3rd value gives 2 returns - minimum for stddev
        assert!(hv.next(&candle(102.0)).is_some());
    }

    #[test]
    fn test_historical_volatility_annualization() {
        let mut hv_daily = HistoricalVolatility::new(10, 252);
        let mut hv_weekly = HistoricalVolatility::new(10, 52);

        let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.0, 104.0];

        for price in &prices {
            hv_daily.next(&candle(*price));
            hv_weekly.next(&candle(*price));
        }

        let daily_vol = hv_daily.current().unwrap();
        let weekly_vol = hv_weekly.current().unwrap();

        // Daily volatility should be higher due to higher annualization factor
        // sqrt(252) vs sqrt(52)
        assert!(daily_vol > weekly_vol);

        // Ratio should be approximately sqrt(252/52) = 2.2
        let ratio = daily_vol / weekly_vol;
        let expected_ratio = (252.0_f64 / 52.0_f64).sqrt();
        assert!((ratio - expected_ratio).abs() < 0.1);
    }

    #[test]
    fn test_historical_volatility_rolling_window() {
        let mut hv = HistoricalVolatility::new(3, 252);

        hv.next(&candle(100.0));
        hv.next(&candle(102.0));
        hv.next(&candle(101.0));
        hv.next(&candle(103.0));

        // Should only use last 3 returns
        assert_eq!(hv.returns.len(), 3);

        hv.next(&candle(104.0));

        // Still only 3 returns
        assert_eq!(hv.returns.len(), 3);
    }

    #[test]
    fn test_historical_volatility_stddev() {
        let hv = HistoricalVolatility::new(3, 252);

        // Test stddev calculation with known values
        let mut test_hv = hv.clone();
        test_hv.returns.push_back(0.01);
        test_hv.returns.push_back(0.02);
        test_hv.returns.push_back(0.03);

        let stddev = test_hv.calculate_stddev();

        // Mean = (0.01 + 0.02 + 0.03) / 3 = 0.02
        // Variance = ((0.01-0.02)^2 + (0.02-0.02)^2 + (0.03-0.02)^2) / 3
        //          = (0.0001 + 0 + 0.0001) / 3 = 0.00006667
        // StdDev = sqrt(0.00006667) ≈ 0.00816

        let expected_mean = 0.02_f64;
        let expected_variance = ((0.01_f64 - expected_mean).powi(2)
            + (0.02_f64 - expected_mean).powi(2)
            + (0.03_f64 - expected_mean).powi(2))
            / 3.0;
        let expected_stddev = expected_variance.sqrt();

        assert!((stddev - expected_stddev).abs() < 1e-10);
    }
}
