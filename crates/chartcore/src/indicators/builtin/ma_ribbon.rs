// MA Ribbon Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct MARibbonIndicator;

impl IndicatorPlugin for MARibbonIndicator {
    fn id(&self) -> &str {
        "ma_ribbon"
    }
    fn name(&self) -> &str {
        "MA Ribbon"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Multiple EMAs forming a ribbon for trend visualization"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("start", "Start Length", 8).min(1).max(50),
            InputConfig::int("increment", "Increment", 5).min(1).max(20),
            InputConfig::int("count", "Number of MAs", 8).min(2).max(12),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("ma1", "MA 1", "#26A69A").line_width(1),
            PlotConfig::new("ma2", "MA 2", "#43A047").line_width(1),
            PlotConfig::new("ma3", "MA 3", "#66BB6A").line_width(1),
            PlotConfig::new("ma4", "MA 4", "#81C784").line_width(1),
            PlotConfig::new("ma5", "MA 5", "#EF9A9A").line_width(1),
            PlotConfig::new("ma6", "MA 6", "#E57373").line_width(1),
            PlotConfig::new("ma7", "MA 7", "#EF5350").line_width(1),
            PlotConfig::new("ma8", "MA 8", "#F44336").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let start = context.input_int("start").unwrap_or(8) as usize;
        let increment = context.input_int("increment").unwrap_or(5) as usize;
        let count = context.input_int("count").unwrap_or(8) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let mut result = IndicatorResult::new("MA Ribbon", "Ribbon", true);

        for i in 0..count.min(8) {
            let length = start + i * increment;
            let ma = ta::ema(&source, length);
            result = result.add_plot(&format!("ma{}", i + 1), ma);
        }

        result
    }
}
