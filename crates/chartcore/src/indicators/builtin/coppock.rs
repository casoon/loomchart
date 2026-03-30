// Coppock Curve Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct CoppockIndicator;

impl IndicatorPlugin for CoppockIndicator {
    fn id(&self) -> &str {
        "coppock"
    }
    fn name(&self) -> &str {
        "Coppock Curve"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Long-term momentum indicator for identifying market bottoms"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("wma", "WMA Length", 10).min(1).max(50),
            InputConfig::int("long_roc", "Long ROC", 14).min(1).max(100),
            InputConfig::int("short_roc", "Short ROC", 11)
                .min(1)
                .max(100),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("coppock", "Coppock", "#673AB7").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let wma_len = context.input_int("wma").unwrap_or(10) as usize;
        let long_roc = context.input_int("long_roc").unwrap_or(14) as usize;
        let short_roc = context.input_int("short_roc").unwrap_or(11) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let coppock = ta::coppock_curve(&source, wma_len, long_roc, short_roc);

        IndicatorResult::new("Coppock Curve", "Coppock", false).add_plot("coppock", coppock)
    }
}
