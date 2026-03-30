// HMA Indicator (Hull Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct HMAIndicator;

impl IndicatorPlugin for HMAIndicator {
    fn id(&self) -> &str {
        "hma"
    }
    fn name(&self) -> &str {
        "Hull Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Fast and smooth moving average with reduced lag"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(2).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("hma", "HMA", "#00BCD4").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let hma = ta::hma(&source, length);

        IndicatorResult::new(
            &format!("HMA ({})", length),
            &format!("HMA({})", length),
            true,
        )
        .add_plot("hma", hma)
    }
}
