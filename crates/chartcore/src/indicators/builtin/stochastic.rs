// Stochastic Oscillator Indicator
//
// The Stochastic Oscillator compares the closing price to the price range
// over a given period. It consists of two lines:
// - %K: The main line showing momentum
// - %D: The signal line (SMA of %K)
//
// Calculation:
// 1. Raw %K = ((Close - Lowest Low) / (Highest High - Lowest Low)) * 100
// 2. %K = SMA(Raw %K, k_smooth)
// 3. %D = SMA(%K, d_period)
//
// Range: 0 to 100
// - Above 80: Overbought
// - Below 20: Oversold

use crate::core::Candle;
use crate::indicators::output::{IndicatorOutput, LineData, LineStyle};
use crate::indicators::Indicator;
use crate::primitives::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stochastic {
    /// Period for finding highest/lowest (default: 14)
    pub k_period: usize,

    /// Smoothing period for %K (default: 1)
    pub k_smooth: usize,

    /// Period for %D signal line (default: 3)
    pub d_period: usize,

    /// Color for %K line
    pub k_color: Color,

    /// Color for %D line
    pub d_color: Color,

    /// Overbought level (default: 80)
    pub overbought: f64,

    /// Oversold level (default: 20)
    pub oversold: f64,
}

impl Default for Stochastic {
    fn default() -> Self {
        Self {
            k_period: 14,
            k_smooth: 1,
            d_period: 3,
            k_color: Color::rgb(41, 98, 255), // #2962FF
            d_color: Color::rgb(255, 109, 0), // #FF6D00
            overbought: 80.0,
            oversold: 20.0,
        }
    }
}

impl Stochastic {
    pub fn new(k_period: usize, k_smooth: usize, d_period: usize) -> Self {
        Self {
            k_period,
            k_smooth,
            d_period,
            ..Default::default()
        }
    }

    pub fn with_colors(mut self, k_color: Color, d_color: Color) -> Self {
        self.k_color = k_color;
        self.d_color = d_color;
        self
    }

    pub fn with_levels(mut self, overbought: f64, oversold: f64) -> Self {
        self.overbought = overbought;
        self.oversold = oversold;
        self
    }

    /// Calculate raw %K values
    fn calculate_raw_k(&self, high: &[f64], low: &[f64], close: &[f64]) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(close.len());

        for i in 0..close.len() {
            if i + 1 < self.k_period {
                result.push(None);
            } else {
                let start_idx = i + 1 - self.k_period;
                let highest = high[start_idx..=i]
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max);

                let lowest = low[start_idx..=i]
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min);

                let range = highest - lowest;
                if range > 0.0 {
                    let k = ((close[i] - lowest) / range) * 100.0;
                    result.push(Some(k));
                } else {
                    // No range means no movement - return 50 (neutral)
                    result.push(Some(50.0));
                }
            }
        }

        result
    }

    /// Calculate SMA (Simple Moving Average)
    fn sma(&self, values: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut result = Vec::with_capacity(values.len());

        for i in 0..values.len() {
            if i + 1 < period {
                result.push(None);
            } else {
                let start_idx = i + 1 - period;
                let sum: f64 = values[start_idx..=i].iter().sum();
                result.push(Some(sum / period as f64));
            }
        }

        result
    }

    /// Compute both %K and %D lines
    fn compute_stochastic(&self, candles: &[Candle]) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
        let high: Vec<f64> = candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = candles.iter().map(|c| c.c).collect();

        // Step 1: Calculate raw %K
        let raw_k = self.calculate_raw_k(&high, &low, &close);

        // Step 2: Smooth %K if k_smooth > 1
        let k = if self.k_smooth > 1 {
            let k_values: Vec<f64> = raw_k.iter().map(|v| v.unwrap_or(50.0)).collect();
            self.sma(&k_values, self.k_smooth)
        } else {
            raw_k
        };

        // Step 3: Calculate %D (SMA of %K)
        let k_for_d: Vec<f64> = k.iter().map(|v| v.unwrap_or(50.0)).collect();
        let d = self.sma(&k_for_d, self.d_period);

        (k, d)
    }
}

impl Indicator for Stochastic {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        let (k_values, d_values) = self.compute_stochastic(candles);

        IndicatorOutput::MultiLine {
            lines: vec![
                LineData {
                    values: k_values,
                    label: "%K".to_string(),
                    color: self.k_color.clone(),
                    width: 2.0,
                    style: LineStyle::Solid,
                },
                LineData {
                    values: d_values,
                    label: "%D".to_string(),
                    color: self.d_color.clone(),
                    width: 2.0,
                    style: LineStyle::Solid,
                },
            ],
        }
    }

    fn get_scale_range(&self, _candles: &[Candle]) -> Option<(f64, f64)> {
        // Stochastic is always 0-100
        Some((0.0, 100.0))
    }

    fn supports_overlay(&self) -> bool {
        // Stochastic is an oscillator, displayed in separate pane
        false
    }

    fn name(&self) -> &str {
        "Stochastic"
    }

    fn id(&self) -> String {
        format!(
            "stoch_{}_{}_{})",
            self.k_period, self.k_smooth, self.d_period
        )
    }

    fn get_params(&self) -> serde_json::Value {
        serde_json::json!({
            "k_period": self.k_period,
            "k_smooth": self.k_smooth,
            "d_period": self.d_period,
            "k_color": self.k_color.to_hex(),
            "d_color": self.d_color.to_hex(),
            "overbought": self.overbought,
            "oversold": self.oversold,
        })
    }

    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String> {
        if let Some(k_period) = params.get("k_period").and_then(|v| v.as_u64()) {
            if k_period < 1 || k_period > 100 {
                return Err("k_period must be between 1 and 100".to_string());
            }
            self.k_period = k_period as usize;
        }

        if let Some(k_smooth) = params.get("k_smooth").and_then(|v| v.as_u64()) {
            if k_smooth < 1 || k_smooth > 10 {
                return Err("k_smooth must be between 1 and 10".to_string());
            }
            self.k_smooth = k_smooth as usize;
        }

        if let Some(d_period) = params.get("d_period").and_then(|v| v.as_u64()) {
            if d_period < 1 || d_period > 10 {
                return Err("d_period must be between 1 and 10".to_string());
            }
            self.d_period = d_period as usize;
        }

        if let Some(k_color) = params.get("k_color").and_then(|v| v.as_str()) {
            self.k_color =
                Color::from_hex(k_color).map_err(|e| format!("Invalid k_color: {}", e))?;
        }

        if let Some(d_color) = params.get("d_color").and_then(|v| v.as_str()) {
            self.d_color =
                Color::from_hex(d_color).map_err(|e| format!("Invalid d_color: {}", e))?;
        }

        if let Some(overbought) = params.get("overbought").and_then(|v| v.as_f64()) {
            self.overbought = overbought;
        }

        if let Some(oversold) = params.get("oversold").and_then(|v| v.as_f64()) {
            self.oversold = oversold;
        }

        Ok(())
    }

    fn required_candles(&self) -> usize {
        self.k_period + self.k_smooth + self.d_period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles() -> Vec<Candle> {
        vec![
            Candle {
                o: 46.0,
                h: 48.0,
                l: 46.0,
                c: 47.0,
                v: 1000.0,
                time: 0,
            },
            Candle {
                o: 47.0,
                h: 48.5,
                l: 46.5,
                c: 47.5,
                v: 1100.0,
                time: 1,
            },
            Candle {
                o: 47.5,
                h: 49.0,
                l: 47.0,
                c: 48.0,
                v: 1200.0,
                time: 2,
            },
            Candle {
                o: 48.0,
                h: 49.5,
                l: 47.5,
                c: 48.5,
                v: 1300.0,
                time: 3,
            },
            Candle {
                o: 48.5,
                h: 50.0,
                l: 48.0,
                c: 49.0,
                v: 1400.0,
                time: 4,
            },
            Candle {
                o: 49.0,
                h: 50.5,
                l: 48.5,
                c: 49.5,
                v: 1500.0,
                time: 5,
            },
            Candle {
                o: 49.5,
                h: 51.0,
                l: 49.0,
                c: 50.0,
                v: 1600.0,
                time: 6,
            },
            Candle {
                o: 50.0,
                h: 51.5,
                l: 49.5,
                c: 50.5,
                v: 1700.0,
                time: 7,
            },
            Candle {
                o: 50.5,
                h: 52.0,
                l: 50.0,
                c: 51.0,
                v: 1800.0,
                time: 8,
            },
            Candle {
                o: 51.0,
                h: 52.5,
                l: 50.5,
                c: 51.5,
                v: 1900.0,
                time: 9,
            },
            Candle {
                o: 51.5,
                h: 53.0,
                l: 51.0,
                c: 52.0,
                v: 2000.0,
                time: 10,
            },
            Candle {
                o: 52.0,
                h: 53.5,
                l: 51.5,
                c: 52.5,
                v: 2100.0,
                time: 11,
            },
            Candle {
                o: 52.5,
                h: 54.0,
                l: 52.0,
                c: 53.0,
                v: 2200.0,
                time: 12,
            },
            Candle {
                o: 53.0,
                h: 54.5,
                l: 52.5,
                c: 53.5,
                v: 2300.0,
                time: 13,
            },
            Candle {
                o: 53.5,
                h: 55.0,
                l: 53.0,
                c: 54.0,
                v: 2400.0,
                time: 14,
            },
        ]
    }

    #[test]
    fn test_stochastic_default() {
        let indicator = Stochastic::default();
        assert_eq!(indicator.k_period, 14);
        assert_eq!(indicator.k_smooth, 1);
        assert_eq!(indicator.d_period, 3);
    }

    #[test]
    fn test_stochastic_calculation() {
        let indicator = Stochastic::new(3, 1, 3);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        match output {
            IndicatorOutput::MultiLine { lines } => {
                assert_eq!(lines.len(), 2);
                assert_eq!(lines[0].label, "%K");
                assert_eq!(lines[1].label, "%D");

                // Check that we have values
                let k_values = &lines[0].values;
                let d_values = &lines[1].values;

                assert_eq!(k_values.len(), candles.len());
                assert_eq!(d_values.len(), candles.len());

                // First few should be None due to required period
                assert!(k_values[0].is_none());
                assert!(k_values[1].is_none());

                // Later values should exist
                assert!(k_values[2].is_some());
            }
            _ => panic!("Expected MultiLine output"),
        }
    }

    #[test]
    fn test_stochastic_range() {
        let indicator = Stochastic::new(5, 1, 3);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        if let IndicatorOutput::MultiLine { lines } = output {
            for line in &lines {
                for value in &line.values {
                    if let Some(v) = value {
                        assert!(
                            *v >= 0.0 && *v <= 100.0,
                            "Stochastic value {} out of range [0, 100]",
                            v
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_stochastic_scale_range() {
        let indicator = Stochastic::default();
        let candles = create_test_candles();

        let range = indicator.get_scale_range(&candles);
        assert_eq!(range, Some((0.0, 100.0)));
    }

    #[test]
    fn test_stochastic_not_overlay() {
        let indicator = Stochastic::default();
        assert!(!indicator.supports_overlay());
    }

    #[test]
    fn test_stochastic_params() {
        let mut indicator = Stochastic::default();

        let params = serde_json::json!({
            "k_period": 21,
            "k_smooth": 3,
            "d_period": 5,
            "overbought": 75.0,
            "oversold": 25.0,
        });

        indicator.set_params(params).unwrap();

        assert_eq!(indicator.k_period, 21);
        assert_eq!(indicator.k_smooth, 3);
        assert_eq!(indicator.d_period, 5);
        assert_eq!(indicator.overbought, 75.0);
        assert_eq!(indicator.oversold, 25.0);
    }

    #[test]
    fn test_stochastic_invalid_params() {
        let mut indicator = Stochastic::default();

        let params = serde_json::json!({
            "k_period": 101, // Invalid: > 100
        });

        let result = indicator.set_params(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_stochastic_required_candles() {
        let indicator = Stochastic::new(14, 1, 3);
        assert_eq!(indicator.required_candles(), 18); // 14 + 1 + 3
    }

    #[test]
    fn test_stochastic_with_smoothing() {
        let indicator = Stochastic::new(5, 3, 3);
        let candles = create_test_candles();

        let output = indicator.calculate(&candles);

        if let IndicatorOutput::MultiLine { lines } = output {
            let k_values = &lines[0].values;

            // With k_smooth=3, we need more candles before getting values
            assert!(k_values[6].is_some()); // 5 + 3 - 2
        }
    }
}
