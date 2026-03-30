// TEMA Indicator (Triple Exponential Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct TEMAIndicator;

impl IndicatorPlugin for TEMAIndicator {
    fn id(&self) -> &str {
        "tema"
    }
    fn name(&self) -> &str {
        "Triple Exponential Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Triple smoothing to further reduce lag"
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
        vec![PlotConfig::new("tema", "TEMA", "#9C27B0").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(21) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let tema = ta::tema(&source, length);

        IndicatorResult::new(
            &format!("TEMA ({})", length),
            &format!("TEMA({})", length),
            true,
        )
        .add_plot("tema", tema)
    }
}
