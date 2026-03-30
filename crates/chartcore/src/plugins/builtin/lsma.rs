// LSMA Indicator (Least Squares Moving Average / Linear Regression)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct LSMAIndicator;

impl IndicatorPlugin for LSMAIndicator {
    fn id(&self) -> &str {
        "lsma"
    }
    fn name(&self) -> &str {
        "Least Squares Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Linear regression line (endpoint)"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 25).min(2).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("lsma", "LSMA", "#3F51B5").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(25) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let lsma = ta::lsma(&source, length);

        IndicatorResult::new(
            &format!("LSMA ({})", length),
            &format!("LSMA({})", length),
            true,
        )
        .add_plot("lsma", lsma)
    }
}
