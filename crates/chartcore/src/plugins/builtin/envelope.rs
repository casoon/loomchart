// Envelope Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct EnvelopeIndicator;

impl IndicatorPlugin for EnvelopeIndicator {
    fn id(&self) -> &str {
        "envelope"
    }
    fn name(&self) -> &str {
        "Envelope"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::ChannelsAndBands
    }
    fn description(&self) -> &str {
        "Percentage bands around moving average"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(1).max(200),
            InputConfig::float("percent", "Percent", 2.5).tooltip("Percentage distance from MA"),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("upper", "Upper", "#26A69A").line_width(1),
            PlotConfig::new("basis", "Basis", "#787B86").line_width(1),
            PlotConfig::new("lower", "Lower", "#EF5350").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let percent = context.input_float("percent").unwrap_or(2.5);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let (upper, basis, lower) = ta::envelope(&source, length, percent, false);

        IndicatorResult::new(&format!("Envelope ({}, {}%)", length, percent), "Env", true)
            .add_plot("upper", upper)
            .add_plot("basis", basis)
            .add_plot("lower", lower)
    }
}
