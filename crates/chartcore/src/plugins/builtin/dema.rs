// DEMA Indicator (Double Exponential Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct DEMAIndicator;

impl IndicatorPlugin for DEMAIndicator {
    fn id(&self) -> &str {
        "dema"
    }
    fn name(&self) -> &str {
        "Double Exponential Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Reduces lag by using double smoothing"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 21).min(1).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("dema", "DEMA", "#E91E63").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(21) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let dema = ta::dema(&source, length);

        IndicatorResult::new(
            &format!("DEMA ({})", length),
            &format!("DEMA({})", length),
            true,
        )
        .add_plot("dema", dema)
    }
}
