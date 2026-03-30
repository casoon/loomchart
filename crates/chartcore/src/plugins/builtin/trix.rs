// TRIX Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct TRIXIndicator;

impl IndicatorPlugin for TRIXIndicator {
    fn id(&self) -> &str {
        "trix"
    }
    fn name(&self) -> &str {
        "TRIX"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Rate of change of triple-smoothed EMA"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 18).min(1).max(100),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("trix", "TRIX", "#E91E63").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(18) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let trix = ta::trix(&source, length);

        IndicatorResult::new(
            &format!("TRIX ({})", length),
            &format!("TRIX({})", length),
            false,
        )
        .add_plot("trix", trix)
    }
}
