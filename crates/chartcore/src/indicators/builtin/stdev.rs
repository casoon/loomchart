// Standard Deviation Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct StdDevIndicator;

impl IndicatorPlugin for StdDevIndicator {
    fn id(&self) -> &str {
        "stdev"
    }
    fn name(&self) -> &str {
        "Standard Deviation"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Statistical dispersion measure"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(2).max(200),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("stdev", "StdDev", "#FF5722").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let stdev = ta::stdev_simple(&source, length);

        IndicatorResult::new(&format!("StdDev ({})", length), "StdDev", false)
            .add_plot("stdev", stdev)
    }
}
