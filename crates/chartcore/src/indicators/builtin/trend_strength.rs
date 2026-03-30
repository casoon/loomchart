// Trend Strength Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct TrendStrengthIndicator;

impl IndicatorPlugin for TrendStrengthIndicator {
    fn id(&self) -> &str {
        "trend_strength"
    }
    fn name(&self) -> &str {
        "Trend Strength"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Measures directional consistency of price movement"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(2).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("strength", "Strength", "#FF9800").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let strength = ta::trend_strength(&close, length);

        IndicatorResult::new(&format!("Trend Strength ({})", length), "TS", false)
            .add_plot("strength", strength)
    }
}
