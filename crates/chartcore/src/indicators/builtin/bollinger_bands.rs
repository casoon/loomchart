/// Bollinger Bands Indicator
///
/// Volatility indicator consisting of a middle band (SMA) and two outer bands
/// at a specified number of standard deviations.
use crate::core::Candle;
use crate::indicators::output::*;
use crate::primitives::Color;
use serde_json::json;

use super::sma::{PriceSource, SMA};

pub struct BollingerBands {
    period: usize,
    std_dev: f64,
    source: PriceSource,
    middle_color: Color,
    band_color: Color,
    fill_alpha: f64,
}

impl BollingerBands {
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            source: PriceSource::Close,
            middle_color: Color::rgb(33, 150, 243),     // Blue
            band_color: Color::rgba(33, 150, 243, 0.3), // Light blue
            fill_alpha: 0.1,
        }
    }

    /// Standard Bollinger Bands(20, 2)
    pub fn standard() -> Self {
        Self::new(20, 2.0)
    }

    pub fn with_source(mut self, source: PriceSource) -> Self {
        self.source = source;
        self
    }

    pub fn with_colors(mut self, middle: Color, band: Color) -> Self {
        self.middle_color = middle;
        self.band_color = band;
        self
    }

    pub fn with_fill_alpha(mut self, alpha: f64) -> Self {
        self.fill_alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Compute Bollinger Bands
    fn compute_bands(
        &self,
        candles: &[Candle],
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        let len = candles.len();

        if len < self.period {
            return (vec![None; len], vec![None; len], vec![None; len]);
        }

        // Calculate middle band (SMA)
        let sma = SMA::new(self.period).with_source(self.source);
        let middle = if let IndicatorOutput::SingleLine { values, .. } = sma.calculate(candles) {
            values
        } else {
            vec![None; len]
        };

        // Calculate upper and lower bands
        let mut upper = vec![None; len];
        let mut lower = vec![None; len];

        for i in (self.period - 1)..len {
            if let Some(sma_val) = middle[i] {
                // Calculate standard deviation for this period
                let std_dev = self.calculate_std_dev(candles, i);

                upper[i] = Some(sma_val + self.std_dev * std_dev);
                lower[i] = Some(sma_val - self.std_dev * std_dev);
            }
        }

        (middle, upper, lower)
    }

    /// Calculate standard deviation for a period ending at index
    fn calculate_std_dev(&self, candles: &[Candle], end_idx: usize) -> f64 {
        let start_idx = end_idx + 1 - self.period;

        // Calculate mean
        let mut sum = 0.0;
        for i in start_idx..=end_idx {
            sum += self.source.extract(&candles[i]);
        }
        let mean = sum / self.period as f64;

        // Calculate variance
        let mut variance_sum = 0.0;
        for i in start_idx..=end_idx {
            let diff = self.source.extract(&candles[i]) - mean;
            variance_sum += diff * diff;
        }
        let variance = variance_sum / self.period as f64;

        variance.sqrt()
    }
}

impl Indicator for BollingerBands {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        let (middle, upper, lower) = self.compute_bands(candles);

        IndicatorOutput::Bands {
            middle,
            upper,
            lower,
            middle_color: self.middle_color.clone(),
            band_color: self.band_color.clone(),
            fill_alpha: self.fill_alpha,
        }
    }

    fn get_scale_range(&self, _candles: &[Candle]) -> Option<(f64, f64)> {
        None // Use price scale
    }

    fn supports_overlay(&self) -> bool {
        true // Bollinger Bands overlay on price chart
    }

    fn name(&self) -> &str {
        "Bollinger Bands"
    }

    fn id(&self) -> String {
        format!("bb_{}_{}", self.period, self.std_dev)
    }

    fn get_params(&self) -> serde_json::Value {
        json!({
            "period": self.period,
            "std_dev": self.std_dev,
            "source": match self.source {
                PriceSource::Close => "close",
                PriceSource::Open => "open",
                PriceSource::High => "high",
                PriceSource::Low => "low",
                PriceSource::HL2 => "hl2",
                PriceSource::HLC3 => "hlc3",
                PriceSource::OHLC4 => "ohlc4",
            },
        })
    }

    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String> {
        if let Some(period) = params.get("period").and_then(|v| v.as_u64()) {
            if period < 2 {
                return Err("Bollinger Bands period must be at least 2".to_string());
            }
            self.period = period as usize;
        }

        if let Some(std_dev) = params.get("std_dev").and_then(|v| v.as_f64()) {
            if std_dev <= 0.0 {
                return Err("Standard deviation must be positive".to_string());
            }
            self.std_dev = std_dev;
        }

        if let Some(source) = params.get("source").and_then(|v| v.as_str()) {
            self.source = match source {
                "close" => PriceSource::Close,
                "open" => PriceSource::Open,
                "high" => PriceSource::High,
                "low" => PriceSource::Low,
                "hl2" => PriceSource::HL2,
                "hlc3" => PriceSource::HLC3,
                "ohlc4" => PriceSource::OHLC4,
                _ => return Err(format!("Unknown price source: {}", source)),
            };
        }

        Ok(())
    }

    fn required_candles(&self) -> usize {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles() -> Vec<Candle> {
        // Create candles with some volatility
        vec![
            Candle::new(1000, 100.0, 101.0, 99.0, 100.0, 1000.0),
            Candle::new(2000, 100.0, 102.0, 99.0, 101.0, 1100.0),
            Candle::new(3000, 101.0, 103.0, 100.0, 102.0, 1200.0),
            Candle::new(4000, 102.0, 104.0, 101.0, 103.0, 1300.0),
            Candle::new(5000, 103.0, 105.0, 102.0, 104.0, 1400.0),
            Candle::new(6000, 104.0, 106.0, 103.0, 105.0, 1500.0),
            Candle::new(7000, 105.0, 107.0, 104.0, 106.0, 1600.0),
            Candle::new(8000, 106.0, 108.0, 105.0, 107.0, 1700.0),
            Candle::new(9000, 107.0, 109.0, 106.0, 108.0, 1800.0),
            Candle::new(10000, 108.0, 110.0, 107.0, 109.0, 1900.0),
            Candle::new(11000, 109.0, 111.0, 108.0, 110.0, 2000.0),
            Candle::new(12000, 110.0, 112.0, 109.0, 111.0, 2100.0),
            Candle::new(13000, 111.0, 113.0, 110.0, 112.0, 2200.0),
            Candle::new(14000, 112.0, 114.0, 111.0, 113.0, 2300.0),
            Candle::new(15000, 113.0, 115.0, 112.0, 114.0, 2400.0),
            Candle::new(16000, 114.0, 116.0, 113.0, 115.0, 2500.0),
            Candle::new(17000, 115.0, 117.0, 114.0, 116.0, 2600.0),
            Candle::new(18000, 116.0, 118.0, 115.0, 117.0, 2700.0),
            Candle::new(19000, 117.0, 119.0, 116.0, 118.0, 2800.0),
            Candle::new(20000, 118.0, 120.0, 117.0, 119.0, 2900.0),
        ]
    }

    #[test]
    fn test_bb_creation() {
        let bb = BollingerBands::new(20, 2.0);
        assert_eq!(bb.period, 20);
        assert_eq!(bb.std_dev, 2.0);
        assert_eq!(bb.name(), "Bollinger Bands");
        assert_eq!(bb.id(), "bb_20_2");
    }

    #[test]
    fn test_bb_standard() {
        let bb = BollingerBands::standard();
        assert_eq!(bb.period, 20);
        assert_eq!(bb.std_dev, 2.0);
    }

    #[test]
    fn test_bb_calculation() {
        let candles = create_test_candles();
        let bb = BollingerBands::standard();

        let output = bb.calculate(&candles);

        if let IndicatorOutput::Bands {
            middle,
            upper,
            lower,
            ..
        } = output
        {
            assert_eq!(middle.len(), candles.len());
            assert_eq!(upper.len(), candles.len());
            assert_eq!(lower.len(), candles.len());

            // First 19 values should be None
            for i in 0..19 {
                assert!(middle[i].is_none());
                assert!(upper[i].is_none());
                assert!(lower[i].is_none());
            }

            // 20th value should exist
            assert!(middle[19].is_some());
            assert!(upper[19].is_some());
            assert!(lower[19].is_some());

            let mid = middle[19].unwrap();
            let up = upper[19].unwrap();
            let low = lower[19].unwrap();

            // Upper should be above middle, middle above lower
            assert!(up > mid);
            assert!(mid > low);

            // Bands should be symmetric around middle
            let upper_distance = up - mid;
            let lower_distance = mid - low;
            assert!((upper_distance - lower_distance).abs() < 0.001);
        } else {
            panic!("Expected Bands output");
        }
    }

    #[test]
    fn test_bb_overlay() {
        let bb = BollingerBands::standard();
        assert!(bb.supports_overlay());
    }

    #[test]
    fn test_bb_params() {
        let mut bb = BollingerBands::standard();

        // Update period
        let params = json!({"period": 50, "std_dev": 2.5});
        assert!(bb.set_params(params).is_ok());
        assert_eq!(bb.period, 50);
        assert_eq!(bb.std_dev, 2.5);

        // Invalid period
        let params = json!({"period": 1});
        assert!(bb.set_params(params).is_err());

        // Invalid std_dev
        let params = json!({"std_dev": -1.0});
        assert!(bb.set_params(params).is_err());
    }

    #[test]
    fn test_bb_required_candles() {
        let bb = BollingerBands::standard();
        assert_eq!(bb.required_candles(), 20);
    }
}
