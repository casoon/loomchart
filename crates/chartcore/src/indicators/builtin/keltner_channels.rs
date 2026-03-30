// Keltner Channels Indicator
//
// Keltner Channels are volatility-based envelopes set above and below an EMA.
// Unlike Bollinger Bands which use standard deviation, Keltner Channels use ATR
// (Average True Range) to measure volatility.
//
// Components:
// - Middle Line: EMA of price (typically 20-period)
// - Upper Channel: Middle + (multiplier × ATR)
// - Lower Channel: Middle - (multiplier × ATR)
//
// Calculation:
// 1. Calculate EMA of typical price (H+L+C)/3
// 2. Calculate ATR
// 3. Upper = EMA + (ATR × multiplier)
// 4. Lower = EMA - (ATR × multiplier)
//
// Typical Settings:
// - Period: 20
// - Multiplier: 2.0
// - ATR Period: 10
//
// Uses:
// - Trend identification (price relative to middle line)
// - Overbought/oversold (price touching bands)
// - Breakout signals (price breaking outside bands)
// - Comparison with Bollinger Bands for confirmation

use crate::core::Candle;
use crate::indicators::builtin::atr::ATR;
use crate::indicators::builtin::ema::EMA;
use crate::indicators::builtin::sma::PriceSource;
use crate::indicators::output::IndicatorOutput;
use crate::indicators::Indicator;
use crate::primitives::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeltnerChannels {
    /// EMA period for middle line (default: 20)
    pub period: usize,

    /// ATR period (default: 10)
    pub atr_period: usize,

    /// ATR multiplier (default: 2.0)
    pub multiplier: f64,

    /// Price source for EMA calculation
    pub source: PriceSource,

    /// Middle line color
    pub middle_color: Color,

    /// Band color (for upper and lower)
    pub band_color: Color,

    /// Fill opacity between bands
    pub fill_alpha: f64,
}

impl Default for KeltnerChannels {
    fn default() -> Self {
        Self {
            period: 20,
            atr_period: 10,
            multiplier: 2.0,
            source: PriceSource::HLC3, // Typical price
            middle_color: Color::rgb(33, 150, 243),
            band_color: Color::rgb(33, 150, 243),
            fill_alpha: 0.1, // 10% opacity
        }
    }
}

impl KeltnerChannels {
    pub fn new(period: usize, atr_period: usize, multiplier: f64) -> Self {
        Self {
            period,
            atr_period,
            multiplier,
            ..Default::default()
        }
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
        self.fill_alpha = alpha;
        self
    }

    /// Compute Keltner Channels
    fn compute_channels(
        &self,
        candles: &[Candle],
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        // Calculate EMA of typical price
        let ema = EMA::new(self.period).with_source(self.source.clone());
        let middle_output = ema.calculate(candles);

        let middle = match middle_output {
            IndicatorOutput::SingleLine { values, .. } => values,
            _ => {
                return (
                    vec![None; candles.len()],
                    vec![None; candles.len()],
                    vec![None; candles.len()],
                )
            }
        };

        // Calculate ATR
        let atr = ATR::new(self.atr_period);
        let atr_output = atr.calculate(candles);

        let atr_values = match atr_output {
            IndicatorOutput::SingleLine { values, .. } => values,
            _ => {
                return (
                    vec![None; candles.len()],
                    vec![None; candles.len()],
                    vec![None; candles.len()],
                )
            }
        };

        // Calculate upper and lower bands
        let mut upper = Vec::with_capacity(candles.len());
        let mut lower = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            match (middle.get(i), atr_values.get(i)) {
                (Some(Some(mid)), Some(Some(atr))) => {
                    let offset = atr * self.multiplier;
                    upper.push(Some(mid + offset));
                    lower.push(Some(mid - offset));
                }
                _ => {
                    upper.push(None);
                    lower.push(None);
                }
            }
        }

        (upper, middle, lower)
    }
}

impl Indicator for KeltnerChannels {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        let (upper, middle, lower) = self.compute_channels(candles);

        IndicatorOutput::Bands {
            middle,
            upper,
            lower,
            middle_color: self.middle_color.clone(),
            band_color: self.band_color.clone(),
            fill_alpha: self.fill_alpha,
        }
    }

    fn get_scale_range(&self, candles: &[Candle]) -> Option<(f64, f64)> {
        let (upper, _, lower) = self.compute_channels(candles);

        let valid_upper: Vec<f64> = upper.iter().filter_map(|&v| v).collect();
        let valid_lower: Vec<f64> = lower.iter().filter_map(|&v| v).collect();

        if valid_upper.is_empty() || valid_lower.is_empty() {
            return None;
        }

        let min = valid_lower.iter().copied().fold(f64::INFINITY, f64::min);
        let max = valid_upper
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        let padding = (max - min) * 0.02;
        Some((min - padding, max + padding))
    }

    fn supports_overlay(&self) -> bool {
        true // Overlays on price chart
    }

    fn name(&self) -> &str {
        "Keltner Channels"
    }

    fn id(&self) -> String {
        format!(
            "keltner_{}_{}_{:.1}",
            self.period, self.atr_period, self.multiplier
        )
    }

    fn get_params(&self) -> serde_json::Value {
        serde_json::json!({
            "period": self.period,
            "atr_period": self.atr_period,
            "multiplier": self.multiplier,
            "source": format!("{:?}", self.source),
            "middle_color": self.middle_color.to_hex(),
            "band_color": self.band_color.to_hex(),
            "fill_alpha": self.fill_alpha,
        })
    }

    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String> {
        if let Some(period) = params.get("period").and_then(|v| v.as_u64()) {
            if period < 1 || period > 500 {
                return Err("period must be between 1 and 500".to_string());
            }
            self.period = period as usize;
        }

        if let Some(atr_period) = params.get("atr_period").and_then(|v| v.as_u64()) {
            if atr_period < 1 || atr_period > 500 {
                return Err("atr_period must be between 1 and 500".to_string());
            }
            self.atr_period = atr_period as usize;
        }

        if let Some(multiplier) = params.get("multiplier").and_then(|v| v.as_f64()) {
            if multiplier <= 0.0 || multiplier > 10.0 {
                return Err("multiplier must be between 0 and 10".to_string());
            }
            self.multiplier = multiplier;
        }

        if let Some(source) = params.get("source").and_then(|v| v.as_str()) {
            self.source = match source {
                "Close" => PriceSource::Close,
                "Open" => PriceSource::Open,
                "High" => PriceSource::High,
                "Low" => PriceSource::Low,
                "HL2" => PriceSource::HL2,
                "HLC3" => PriceSource::HLC3,
                "OHLC4" => PriceSource::OHLC4,
                _ => return Err(format!("Invalid source: {}", source)),
            };
        }

        if let Some(color) = params.get("middle_color").and_then(|v| v.as_str()) {
            self.middle_color =
                Color::from_hex(color).map_err(|e| format!("Invalid middle_color: {}", e))?;
        }

        if let Some(color) = params.get("band_color").and_then(|v| v.as_str()) {
            self.band_color =
                Color::from_hex(color).map_err(|e| format!("Invalid band_color: {}", e))?;
        }

        if let Some(alpha) = params.get("fill_alpha").and_then(|v| v.as_f64()) {
            if alpha < 0.0 || alpha > 1.0 {
                return Err("fill_alpha must be between 0.0 and 1.0".to_string());
            }
            self.fill_alpha = alpha;
        }

        Ok(())
    }

    fn required_candles(&self) -> usize {
        self.period.max(self.atr_period) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles() -> Vec<Candle> {
        let mut candles = Vec::new();
        for i in 0..50 {
            let base = 100.0 + (i as f64 * 0.5);
            candles.push(Candle {
                time: i,
                o: base,
                h: base + 2.0,
                l: base - 2.0,
                c: base + 1.0,
                v: 1000.0,
            });
        }
        candles
    }

    #[test]
    fn test_keltner_default() {
        let indicator = KeltnerChannels::default();
        assert_eq!(indicator.period, 20);
        assert_eq!(indicator.atr_period, 10);
        assert_eq!(indicator.multiplier, 2.0);
    }

    #[test]
    fn test_keltner_calculation() {
        let indicator = KeltnerChannels::new(20, 10, 2.0);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        match output {
            IndicatorOutput::Bands {
                upper,
                middle,
                lower,
                ..
            } => {
                assert_eq!(upper.len(), candles.len());
                assert_eq!(lower.len(), candles.len());

                // Check that we have values after warmup
                assert!(upper[30].is_some());
                assert!(middle[30].is_some());
                assert!(lower[30].is_some());

                // Upper should be greater than lower
                if let (Some(u), Some(l)) = (upper[30], lower[30]) {
                    assert!(u > l, "Upper band should be above lower band");
                }

                // Middle should be between upper and lower
                if let (Some(u), Some(m), Some(l)) = (upper[30], middle[30], lower[30]) {
                    assert!(m > l && m < u, "Middle should be between bands");
                }
            }
            _ => panic!("Expected Bands output"),
        }
    }

    #[test]
    fn test_keltner_bandwidth_increases_with_multiplier() {
        let candles = create_test_candles();

        let kc1 = KeltnerChannels::new(20, 10, 1.0);
        let kc2 = KeltnerChannels::new(20, 10, 3.0);

        let out1 = kc1.calculate(&candles);
        let out2 = kc2.calculate(&candles);

        if let (
            IndicatorOutput::Bands {
                upper: u1,
                lower: l1,
                ..
            },
            IndicatorOutput::Bands {
                upper: u2,
                lower: l2,
                ..
            },
        ) = (out1, out2)
        {
            // Bandwidth with multiplier 3.0 should be wider than 1.0
            if let (Some(upper1), Some(lower1), Some(upper2), Some(lower2)) =
                (u1[40], l1[40], u2[40], l2[40])
            {
                let width1 = upper1 - lower1;
                let width2 = upper2 - lower2;
                assert!(
                    width2 > width1,
                    "Higher multiplier should create wider bands"
                );
            }
        }
    }

    #[test]
    fn test_keltner_supports_overlay() {
        let indicator = KeltnerChannels::default();
        assert!(indicator.supports_overlay());
    }

    #[test]
    fn test_keltner_scale_range() {
        let indicator = KeltnerChannels::new(20, 10, 2.0);
        let candles = create_test_candles();

        let range = indicator.get_scale_range(&candles);
        assert!(range.is_some());

        let (min, max) = range.unwrap();
        assert!(min < max);

        // Range should encompass price action
        let min_price = candles.iter().map(|c| c.l).fold(f64::INFINITY, f64::min);
        let max_price = candles
            .iter()
            .map(|c| c.h)
            .fold(f64::NEG_INFINITY, f64::max);

        assert!(min < min_price);
        assert!(max > max_price);
    }

    #[test]
    fn test_keltner_params() {
        let mut indicator = KeltnerChannels::default();

        let params = serde_json::json!({
            "period": 50,
            "atr_period": 14,
            "multiplier": 2.5,
            "source": "Close",
        });

        indicator.set_params(params).unwrap();

        assert_eq!(indicator.period, 50);
        assert_eq!(indicator.atr_period, 14);
        assert_eq!(indicator.multiplier, 2.5);
    }

    #[test]
    fn test_keltner_invalid_params() {
        let mut indicator = KeltnerChannels::default();

        let params = serde_json::json!({
            "multiplier": 15.0, // Invalid: > 10
        });

        let result = indicator.set_params(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_keltner_required_candles() {
        let indicator = KeltnerChannels::new(20, 14, 2.0);
        assert_eq!(indicator.required_candles(), 21); // max(20, 14) + 1
    }

    #[test]
    fn test_keltner_vs_bollinger() {
        // Keltner uses ATR (absolute volatility)
        // Bollinger uses StdDev (relative volatility)
        // Both should give valid but different results

        let candles = create_test_candles();
        let kc = KeltnerChannels::new(20, 20, 2.0);

        let output = kc.calculate(&candles);

        match output {
            IndicatorOutput::Bands { upper, lower, .. } => {
                // Should have valid values
                assert!(upper[30].is_some());
                assert!(lower[30].is_some());

                // Bands should be meaningful
                if let (Some(u), Some(l)) = (upper[30], lower[30]) {
                    let width = u - l;
                    assert!(width > 0.0);
                    assert!(width < 50.0); // Reasonable width
                }
            }
            _ => panic!("Expected Bands output"),
        }
    }
}
