// ZigZag Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ZigZagIndicator;

impl IndicatorPlugin for ZigZagIndicator {
    fn id(&self) -> &str {
        "zigzag"
    }
    fn name(&self) -> &str {
        "ZigZag"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Filters out minor price movements to show significant swings"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::float("deviation", "Deviation %", 5.0)
            .tooltip("Minimum percentage move to create new pivot")]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("zigzag", "ZigZag", "#2196F3").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let deviation = context.input_float("deviation").unwrap_or(5.0);

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();

        let pivots = ta::zigzag(&high, &low, deviation);

        // Convert pivots to line values (interpolate between pivots)
        let len = context.candles.len();
        let mut zigzag_line: Vec<Option<f64>> = vec![None; len];

        if pivots.len() >= 2 {
            for i in 0..pivots.len() - 1 {
                let (start_idx, start_price, _) = pivots[i];
                let (end_idx, end_price, _) = pivots[i + 1];

                // Interpolate between pivots
                for j in start_idx..=end_idx {
                    if j < len {
                        let progress = (j - start_idx) as f64 / (end_idx - start_idx).max(1) as f64;
                        zigzag_line[j] = Some(start_price + (end_price - start_price) * progress);
                    }
                }
            }
        }

        IndicatorResult::new(&format!("ZigZag ({}%)", deviation), "ZZ", true)
            .add_plot("zigzag", zigzag_line)
    }
}
