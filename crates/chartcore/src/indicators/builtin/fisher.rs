// Fisher Transform Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct FisherIndicator;

impl IndicatorPlugin for FisherIndicator {
    fn id(&self) -> &str {
        "fisher"
    }

    fn name(&self) -> &str {
        "Fisher Transform"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Converts price to Gaussian normal distribution for clearer turning point signals"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 9).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("fisher", "Fisher", "#2196F3").line_width(2),
            PlotConfig::new("trigger", "Trigger", "#FF9800").line_width(1),
            PlotConfig::new("zero", "Zero", "#888888")
                .line_width(1)
                .style("dashed"),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(9) as usize;

        let highs = context.high();
        let lows = context.low();

        let (fisher, trigger) = ta::fisher_transform(&highs, &lows, length);

        let len = highs.len();
        let zero_line: Vec<Option<f64>> = vec![Some(0.0); len];

        IndicatorResult::new(
            &format!("Fisher Transform ({})", length),
            &format!("FISH({})", length),
            false,
        )
        .add_plot("fisher", fisher)
        .add_plot("trigger", trigger)
        .add_plot("zero", zero_line)
    }
}
