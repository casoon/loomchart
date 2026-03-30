// MA Cross Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct MACrossIndicator;

impl IndicatorPlugin for MACrossIndicator {
    fn id(&self) -> &str {
        "ma_cross"
    }
    fn name(&self) -> &str {
        "MA Cross"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Two moving averages with crossover signals"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("fast", "Fast Length", 9).min(1).max(200),
            InputConfig::int("slow", "Slow Length", 21).min(1).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("fast", "Fast MA", "#26A69A").line_width(2),
            PlotConfig::new("slow", "Slow MA", "#EF5350").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let fast_len = context.input_int("fast").unwrap_or(9) as usize;
        let slow_len = context.input_int("slow").unwrap_or(21) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let fast = ta::ema(&source, fast_len);
        let slow = ta::ema(&source, slow_len);

        IndicatorResult::new(
            &format!("MA Cross ({}, {})", fast_len, slow_len),
            "MA Cross",
            true,
        )
        .add_plot("fast", fast)
        .add_plot("slow", slow)
    }
}
