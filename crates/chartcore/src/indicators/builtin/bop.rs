// Balance of Power Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct BOPIndicator;

impl IndicatorPlugin for BOPIndicator {
    fn id(&self) -> &str {
        "bop"
    }

    fn name(&self) -> &str {
        "Balance of Power"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Measures the strength of buyers vs sellers by comparing close to open relative to range"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("smooth", "Smoothing Period", 14)
            .min(1)
            .max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("bop", "BOP", "#2196F3").line_width(2),
            PlotConfig::new("zero", "Zero", "#888888")
                .line_width(1)
                .style("dashed"),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let smooth = context.input_int("smooth").unwrap_or(14) as usize;

        let opens = context.open();
        let highs = context.high();
        let lows = context.low();
        let closes = context.close();

        let bop = ta::bop(&opens, &highs, &lows, &closes);
        let bop_smoothed = if smooth > 1 {
            ta::sma(&bop, smooth)
        } else {
            bop.into_iter().map(Some).collect()
        };

        let len = closes.len();
        let zero_line: Vec<Option<f64>> = vec![Some(0.0); len];

        IndicatorResult::new(
            &format!("BOP ({})", smooth),
            &format!("BOP({})", smooth),
            false,
        )
        .add_plot("bop", bop_smoothed)
        .add_plot("zero", zero_line)
    }
}
