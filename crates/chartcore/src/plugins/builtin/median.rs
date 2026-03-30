// Median Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig, SourceType,
};
use crate::ta;

#[derive(Default)]
pub struct MedianIndicator;

impl IndicatorPlugin for MedianIndicator {
    fn id(&self) -> &str {
        "median"
    }

    fn name(&self) -> &str {
        "Median"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Rolling median value over a specified period"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 5).min(1).max(200),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("median", "Median", "#9C27B0").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(5) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let median = ta::median(&source, length);

        IndicatorResult::new(
            &format!("Median ({})", length),
            &format!("MED({})", length),
            true,
        )
        .add_plot("median", median)
    }
}
