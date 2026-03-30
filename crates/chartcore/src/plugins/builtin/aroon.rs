// Aroon Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct AroonIndicator;

impl IndicatorPlugin for AroonIndicator {
    fn id(&self) -> &str {
        "aroon"
    }
    fn name(&self) -> &str {
        "Aroon"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Identifies trend changes and strength"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 25).min(1).max(200)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("up", "Aroon Up", "#26A69A").line_width(2),
            PlotConfig::new("down", "Aroon Down", "#EF5350").line_width(2),
            PlotConfig::new("osc", "Oscillator", "#2196F3").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(25) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();

        let (up, down, osc) = ta::aroon(&high, &low, length);

        IndicatorResult::new(&format!("Aroon ({})", length), "Aroon", false)
            .add_plot("up", up)
            .add_plot("down", down)
            .add_plot("osc", osc)
    }
}
