// JMA Indicator (Jurik Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct JMAIndicator;

impl IndicatorPlugin for JMAIndicator {
    fn id(&self) -> &str {
        "jma"
    }
    fn name(&self) -> &str {
        "Jurik Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Advanced adaptive moving average with minimal lag and smooth output"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 14).min(1).max(500),
            InputConfig::float("phase", "Phase", 0.0)
                .tooltip("-100 to +100: controls overshooting"),
            InputConfig::float("power", "Power", 2.0).tooltip("Smoothness factor (1 = max smooth)"),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("jma", "JMA", "#FF5722").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;
        let phase = context.input_float("phase").unwrap_or(0.0);
        let power = context.input_float("power").unwrap_or(2.0);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let jma = ta::jma(&source, length, phase, power);

        IndicatorResult::new(
            &format!("Jurik Moving Average ({}, {}, {})", length, phase, power),
            &format!("JMA({})", length),
            true,
        )
        .add_plot("jma", jma)
    }
}
