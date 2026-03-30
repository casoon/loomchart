// Momentum Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct MomentumIndicator;

impl IndicatorPlugin for MomentumIndicator {
    fn id(&self) -> &str {
        "momentum"
    }
    fn name(&self) -> &str {
        "Momentum"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Price change over a period"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 10).min(1).max(200),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("mom", "Momentum", "#FF9800").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(10) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let mom = ta::momentum(&source, length);

        IndicatorResult::new(
            &format!("Momentum ({})", length),
            &format!("Mom({})", length),
            false,
        )
        .add_plot("mom", mom)
    }
}
