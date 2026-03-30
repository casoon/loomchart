// WMA Indicator (Weighted Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct WMAIndicator;

impl IndicatorPlugin for WMAIndicator {
    fn id(&self) -> &str {
        "wma"
    }

    fn name(&self) -> &str {
        "Weighted Moving Average"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }

    fn description(&self) -> &str {
        "Moving average with linear weighting giving more importance to recent prices"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(1).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("wma", "WMA", "#9C27B0").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);

        let source = context.source(source_type);
        let wma = ta::wma(&source, length);

        IndicatorResult::new(
            &format!("Weighted Moving Average ({})", length),
            &format!("WMA({})", length),
            true,
        )
        .add_plot("wma", wma)
    }
}
