// SMMA Indicator (Smoothed Moving Average / RMA)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct SMMAIndicator;

impl IndicatorPlugin for SMMAIndicator {
    fn id(&self) -> &str {
        "smma"
    }
    fn name(&self) -> &str {
        "Smoothed Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Wilder's smoothed moving average (RMA)"
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
        vec![PlotConfig::new("smma", "SMMA", "#607D8B").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let smma = ta::smma(&source, length);

        IndicatorResult::new(
            &format!("SMMA ({})", length),
            &format!("SMMA({})", length),
            true,
        )
        .add_plot("smma", smma)
    }
}
