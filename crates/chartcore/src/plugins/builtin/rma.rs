// RMA (Running Moving Average / Wilder's Smoothing) Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct RMAIndicator;

impl IndicatorPlugin for RMAIndicator {
    fn id(&self) -> &str {
        "rma"
    }
    fn name(&self) -> &str {
        "RMA (Wilder's Smoothing)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Running Moving Average, also known as Wilder's Smoothing Method"
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
        vec![PlotConfig::new("rma", "RMA", "#FF9800").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let rma = ta::rma(&source, length);

        IndicatorResult::new(
            &format!("RMA ({})", length),
            &format!("RMA({})", length),
            true,
        )
        .add_plot("rma", rma)
    }
}
