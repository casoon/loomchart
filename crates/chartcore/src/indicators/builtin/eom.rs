// Ease of Movement Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct EaseOfMovementIndicator;

impl IndicatorPlugin for EaseOfMovementIndicator {
    fn id(&self) -> &str {
        "eom"
    }
    fn name(&self) -> &str {
        "Ease of Movement"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Relates price change to volume"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("eom", "EoM", "#795548").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let eom = ta::ease_of_movement(&high, &low, &volume, length);

        IndicatorResult::new(&format!("EoM ({})", length), "EoM", false).add_plot("eom", eom)
    }
}
