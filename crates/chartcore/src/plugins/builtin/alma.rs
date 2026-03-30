// ALMA Indicator (Arnaud Legoux Moving Average)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ALMAIndicator;

impl IndicatorPlugin for ALMAIndicator {
    fn id(&self) -> &str {
        "alma"
    }
    fn name(&self) -> &str {
        "Arnaud Legoux Moving Average"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }
    fn description(&self) -> &str {
        "Gaussian-weighted MA with reduced lag"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 9).min(1).max(500),
            InputConfig::float("offset", "Offset", 0.85)
                .tooltip("0-1: tradeoff between smoothness and responsiveness"),
            InputConfig::float("sigma", "Sigma", 6.0)
                .tooltip("Standard deviation for Gaussian sharpness"),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("alma", "ALMA", "#673AB7").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(9) as usize;
        let offset = context.input_float("offset").unwrap_or(0.85);
        let sigma = context.input_float("sigma").unwrap_or(6.0);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let alma = ta::alma(&source, length, offset, sigma);

        IndicatorResult::new(
            &format!("ALMA ({}, {}, {})", length, offset, sigma),
            &format!("ALMA({})", length),
            true,
        )
        .add_plot("alma", alma)
    }
}
