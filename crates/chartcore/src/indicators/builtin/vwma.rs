// VWMA Indicator (Volume Weighted Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct VWMAIndicator;

impl IndicatorPlugin for VWMAIndicator {
    fn id(&self) -> &str {
        "vwma"
    }
    fn name(&self) -> &str {
        "Volume Weighted Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Moving average weighted by volume"
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
        vec![PlotConfig::new("vwma", "VWMA", "#795548").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();
        let vwma = ta::vwma(&source, &volume, length);

        IndicatorResult::new(
            &format!("VWMA ({})", length),
            &format!("VWMA({})", length),
            true,
        )
        .add_plot("vwma", vwma)
    }
}
