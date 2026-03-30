// SMA Indicator (Built-in Rust implementation)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct SMAIndicator;

impl IndicatorPlugin for SMAIndicator {
    fn id(&self) -> &str {
        "sma"
    }

    fn name(&self) -> &str {
        "Simple Moving Average"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }

    fn description(&self) -> &str {
        "Average price over a specified period"
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
        vec![PlotConfig::new("sma", "SMA", "#FFA726").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);

        let source = context.source(source_type);
        let sma = ta::sma(&source, length);

        IndicatorResult::new(
            &format!("Simple Moving Average ({})", length),
            &format!("SMA({})", length),
            true,
        )
        .add_plot("sma", sma)
    }
}
