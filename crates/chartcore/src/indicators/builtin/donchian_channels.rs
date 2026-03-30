// Donchian Channels Indicator
//
// Donchian Channels show the highest high and lowest low over a period,
// commonly used for breakout trading strategies. Developed by Richard Donchian.
//
// Components:
// - Upper Channel: Highest high over N periods
// - Lower Channel: Lowest low over N periods
// - Middle Line: (Upper + Lower) / 2
//
// Calculation:
// 1. Upper = MAX(high, period)
// 2. Lower = MIN(low, period)
// 3. Middle = (Upper + Lower) / 2
//
// Typical Settings:
// - Period: 20 (default)
// - Some traders use 55 or 89 (Fibonacci numbers)
//
// Uses:
// - Breakout signals (price breaking above/below channels)
// - Trend identification (price relative to middle line)
// - Support/resistance levels
// - Volatility measurement (channel width)
//
// Trading Rules:
// - Buy when price breaks above upper channel
// - Sell when price breaks below lower channel
// - Middle line acts as trailing stop

use crate::core::Candle;
use crate::indicators::output::IndicatorOutput;
use crate::indicators::Indicator;
use crate::primitives::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonchianChannels {
    /// Period for highest/lowest calculation (default: 20)
    pub period: usize,

    /// Upper channel color
    pub upper_color: Color,

    /// Middle line color
    pub middle_color: Color,

    /// Lower channel color
    pub lower_color: Color,

    /// Fill opacity between channels
    pub fill_alpha: f64,
}

impl Default for DonchianChannels {
    fn default() -> Self {
        Self {
            period: 20,
            upper_color: Color::rgb(76, 175, 80), // Green #4CAF50
            middle_color: Color::rgb(76, 175, 80),
            lower_color: Color::rgb(76, 175, 80),
            fill_alpha: 0.1, // 10% opacity
        }
    }
}

impl DonchianChannels {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            ..Default::default()
        }
    }

    pub fn with_colors(mut self, color: Color) -> Self {
        self.upper_color = color.clone();
        self.middle_color = color.clone();
        self.lower_color = color;
        self
    }

    pub fn with_fill_alpha(mut self, alpha: f64) -> Self {
        self.fill_alpha = alpha;
        self
    }

    /// Compute Donchian Channels
    fn compute_channels(
        &self,
        candles: &[Candle],
    ) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
        let mut upper = Vec::with_capacity(candles.len());
        let mut middle = Vec::with_capacity(candles.len());
        let mut lower = Vec::with_capacity(candles.len());

        for i in 0..candles.len() {
            if i + 1 < self.period {
                upper.push(None);
                middle.push(None);
                lower.push(None);
            } else {
                // Find highest high and lowest low over period
                let start = i + 1 - self.period;
                let end = i + 1;

                let highest = candles[start..end]
                    .iter()
                    .map(|c| c.h)
                    .fold(f64::NEG_INFINITY, f64::max);

                let lowest = candles[start..end]
                    .iter()
                    .map(|c| c.l)
                    .fold(f64::INFINITY, f64::min);

                let mid = (highest + lowest) / 2.0;

                upper.push(Some(highest));
                middle.push(Some(mid));
                lower.push(Some(lowest));
            }
        }

        (upper, middle, lower)
    }
}

impl Indicator for DonchianChannels {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        let (upper, middle, lower) = self.compute_channels(candles);

        IndicatorOutput::Bands {
            middle,
            upper,
            lower,
            middle_color: self.middle_color.clone(),
            band_color: self.upper_color.clone(),
            fill_alpha: self.fill_alpha,
        }
    }

    fn get_scale_range(&self, candles: &[Candle]) -> Option<(f64, f64)> {
        if candles.is_empty() {
            return None;
        }

        let min = candles.iter().map(|c| c.l).fold(f64::INFINITY, f64::min);
        let max = candles
            .iter()
            .map(|c| c.h)
            .fold(f64::NEG_INFINITY, f64::max);

        let padding = (max - min) * 0.02;
        Some((min - padding, max + padding))
    }

    fn supports_overlay(&self) -> bool {
        true // Overlays on price chart
    }

    fn name(&self) -> &str {
        "Donchian Channels"
    }

    fn id(&self) -> String {
        format!("donchian_{}", self.period)
    }

    fn get_params(&self) -> serde_json::Value {
        serde_json::json!({
            "period": self.period,
            "upper_color": self.upper_color.to_hex(),
            "middle_color": self.middle_color.to_hex(),
            "lower_color": self.lower_color.to_hex(),
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

        if let Some(color) = params.get("upper_color").and_then(|v| v.as_str()) {
            self.upper_color =
                Color::from_hex(color).map_err(|e| format!("Invalid upper_color: {}", e))?;
        }

        if let Some(color) = params.get("middle_color").and_then(|v| v.as_str()) {
            self.middle_color =
                Color::from_hex(color).map_err(|e| format!("Invalid middle_color: {}", e))?;
        }

        if let Some(color) = params.get("lower_color").and_then(|v| v.as_str()) {
            self.lower_color =
                Color::from_hex(color).map_err(|e| format!("Invalid lower_color: {}", e))?;
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
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles() -> Vec<Candle> {
        vec![
            Candle {
                time: 0,
                o: 100.0,
                h: 105.0,
                l: 95.0,
                c: 102.0,
                v: 1000.0,
            },
            Candle {
                time: 1,
                o: 102.0,
                h: 107.0,
                l: 98.0,
                c: 104.0,
                v: 1100.0,
            },
            Candle {
                time: 2,
                o: 104.0,
                h: 108.0,
                l: 100.0,
                c: 106.0,
                v: 1200.0,
            },
            Candle {
                time: 3,
                o: 106.0,
                h: 110.0,
                l: 102.0,
                c: 108.0,
                v: 1300.0,
            },
            Candle {
                time: 4,
                o: 108.0,
                h: 112.0,
                l: 104.0,
                c: 110.0,
                v: 1400.0,
            },
            Candle {
                time: 5,
                o: 110.0,
                h: 114.0,
                l: 106.0,
                c: 112.0,
                v: 1500.0,
            },
            Candle {
                time: 6,
                o: 112.0,
                h: 116.0,
                l: 108.0,
                c: 114.0,
                v: 1600.0,
            },
            Candle {
                time: 7,
                o: 114.0,
                h: 118.0,
                l: 110.0,
                c: 116.0,
                v: 1700.0,
            },
            Candle {
                time: 8,
                o: 116.0,
                h: 120.0,
                l: 112.0,
                c: 118.0,
                v: 1800.0,
            },
            Candle {
                time: 9,
                o: 118.0,
                h: 122.0,
                l: 114.0,
                c: 120.0,
                v: 1900.0,
            },
            Candle {
                time: 10,
                o: 120.0,
                h: 124.0,
                l: 116.0,
                c: 122.0,
                v: 2000.0,
            },
        ]
    }

    #[test]
    fn test_donchian_default() {
        let indicator = DonchianChannels::default();
        assert_eq!(indicator.period, 20);
    }

    #[test]
    fn test_donchian_calculation() {
        let indicator = DonchianChannels::new(5);
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
                assert_eq!(middle.len(), candles.len());
                assert_eq!(lower.len(), candles.len());

                // First 4 should be None (need 5 for period=5)
                for i in 0..4 {
                    assert!(upper[i].is_none());
                    assert!(middle[i].is_none());
                    assert!(lower[i].is_none());
                }

                // Fifth value should exist
                assert!(upper[4].is_some());
                assert!(middle[4].is_some());
                assert!(lower[4].is_some());

                // Upper should be the highest high
                if let Some(u) = upper[4] {
                    let expected = candles[0..5]
                        .iter()
                        .map(|c| c.h)
                        .fold(f64::NEG_INFINITY, f64::max);
                    assert_eq!(u, expected);
                }

                // Lower should be the lowest low
                if let Some(l) = lower[4] {
                    let expected = candles[0..5]
                        .iter()
                        .map(|c| c.l)
                        .fold(f64::INFINITY, f64::min);
                    assert_eq!(l, expected);
                }

                // Middle should be between upper and lower
                if let (Some(u), Some(m), Some(l)) = (upper[4], middle[4], lower[4]) {
                    assert!(m > l && m < u, "Middle should be between bands");
                    assert_eq!(
                        m,
                        (u + l) / 2.0,
                        "Middle should be average of upper and lower"
                    );
                }
            }
            _ => panic!("Expected Bands output"),
        }
    }

    #[test]
    fn test_donchian_moving_window() {
        let indicator = DonchianChannels::new(3);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        if let IndicatorOutput::Bands { upper, lower, .. } = output {
            // At index 5, should use candles 3-5
            if let (Some(u), Some(l)) = (upper[5], lower[5]) {
                let expected_high = candles[3..6]
                    .iter()
                    .map(|c| c.h)
                    .fold(f64::NEG_INFINITY, f64::max);
                let expected_low = candles[3..6]
                    .iter()
                    .map(|c| c.l)
                    .fold(f64::INFINITY, f64::min);
                assert_eq!(u, expected_high);
                assert_eq!(l, expected_low);
            }
        }
    }

    #[test]
    fn test_donchian_supports_overlay() {
        let indicator = DonchianChannels::default();
        assert!(indicator.supports_overlay());
    }

    #[test]
    fn test_donchian_scale_range() {
        let indicator = DonchianChannels::new(5);
        let candles = create_test_candles();

        let range = indicator.get_scale_range(&candles);
        assert!(range.is_some());

        let (min, max) = range.unwrap();
        assert!(min < max);

        // Range should encompass all price action
        let min_price = candles.iter().map(|c| c.l).fold(f64::INFINITY, f64::min);
        let max_price = candles
            .iter()
            .map(|c| c.h)
            .fold(f64::NEG_INFINITY, f64::max);

        assert!(min <= min_price);
        assert!(max >= max_price);
    }

    #[test]
    fn test_donchian_params() {
        let mut indicator = DonchianChannels::default();

        let params = serde_json::json!({
            "period": 55,
            "fill_alpha": 0.2,
        });

        indicator.set_params(params).unwrap();

        assert_eq!(indicator.period, 55);
        assert_eq!(indicator.fill_alpha, 0.2);
    }

    #[test]
    fn test_donchian_invalid_params() {
        let mut indicator = DonchianChannels::default();

        let params = serde_json::json!({
            "period": 501, // Invalid: > 500
        });

        let result = indicator.set_params(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_donchian_required_candles() {
        let indicator = DonchianChannels::new(20);
        assert_eq!(indicator.required_candles(), 20);
    }

    #[test]
    fn test_donchian_breakout_detection() {
        // Test that channels properly identify breakout levels
        let indicator = DonchianChannels::new(5);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        if let IndicatorOutput::Bands { upper, lower, .. } = output {
            // Check last value
            if let (Some(u), Some(l)) = (upper[10], lower[10]) {
                // Upper should be highest of last 5 candles
                let last_5_high = candles[6..11]
                    .iter()
                    .map(|c| c.h)
                    .fold(f64::NEG_INFINITY, f64::max);
                let last_5_low = candles[6..11]
                    .iter()
                    .map(|c| c.l)
                    .fold(f64::INFINITY, f64::min);

                assert_eq!(u, last_5_high);
                assert_eq!(l, last_5_low);
            }
        }
    }

    #[test]
    fn test_donchian_channel_width() {
        // Channel width should reflect volatility
        let indicator = DonchianChannels::new(5);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        if let IndicatorOutput::Bands { upper, lower, .. } = output {
            if let (Some(u1), Some(l1)) = (upper[4], lower[4]) {
                let width1 = u1 - l1;
                assert!(width1 > 0.0, "Channel width should be positive");

                // Later in uptrend, width might differ
                if let (Some(u2), Some(l2)) = (upper[10], lower[10]) {
                    let width2 = u2 - l2;
                    assert!(width2 > 0.0, "Channel width should be positive");
                }
            }
        }
    }
}
