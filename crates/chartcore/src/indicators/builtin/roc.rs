// ROC Indicator (Rate of Change)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ROCIndicator;

impl IndicatorPlugin for ROCIndicator {
    fn id(&self) -> &str {
        "roc"
    }
    fn name(&self) -> &str {
        "Rate of Change"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Percentage price change over a period"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 9).min(1).max(200),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("roc", "ROC", "#AB47BC").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(9) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let roc = ta::roc(&source, length);

        IndicatorResult::new(
            &format!("ROC ({})", length),
            &format!("ROC({})", length),
            false,
        )
        .add_plot("roc", roc)
    }
}
