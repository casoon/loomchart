// McGinley Dynamic Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct McGinleyIndicator;

impl IndicatorPlugin for McGinleyIndicator {
    fn id(&self) -> &str {
        "mcginley"
    }
    fn name(&self) -> &str {
        "McGinley Dynamic"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Self-adjusting moving average that adapts to market speed"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 14).min(1).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("mcginley", "McGinley", "#009688").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let mcginley = ta::mcginley_dynamic(&source, length);

        IndicatorResult::new(
            &format!("McGinley Dynamic ({})", length),
            &format!("MD({})", length),
            true,
        )
        .add_plot("mcginley", mcginley)
    }
}
