// RCI (Rank Correlation Index) Ribbon Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct RCIIndicator;

impl IndicatorPlugin for RCIIndicator {
    fn id(&self) -> &str {
        "rci"
    }

    fn name(&self) -> &str {
        "RCI Ribbon"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Rank Correlation Index ribbon with short, medium, and long-term periods"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("short_length", "Short Length", 9)
                .min(2)
                .max(100),
            InputConfig::int("medium_length", "Medium Length", 26)
                .min(2)
                .max(200),
            InputConfig::int("long_length", "Long Length", 52)
                .min(2)
                .max(300),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("rci_short", "RCI Short", "#4CAF50").line_width(2),
            PlotConfig::new("rci_medium", "RCI Medium", "#FF9800").line_width(2),
            PlotConfig::new("rci_long", "RCI Long", "#2196F3").line_width(2),
            PlotConfig::new("overbought", "+80", "#888888")
                .line_width(1)
                .style("dashed"),
            PlotConfig::new("oversold", "-80", "#888888")
                .line_width(1)
                .style("dashed"),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let short_length = context.input_int("short_length").unwrap_or(9) as usize;
        let medium_length = context.input_int("medium_length").unwrap_or(26) as usize;
        let long_length = context.input_int("long_length").unwrap_or(52) as usize;

        let closes = context.close();

        let rci_short = ta::rci(&closes, short_length);
        let rci_medium = ta::rci(&closes, medium_length);
        let rci_long = ta::rci(&closes, long_length);

        let len = closes.len();
        let overbought: Vec<Option<f64>> = vec![Some(80.0); len];
        let oversold: Vec<Option<f64>> = vec![Some(-80.0); len];

        IndicatorResult::new(
            &format!("RCI ({}, {}, {})", short_length, medium_length, long_length),
            "RCI",
            false,
        )
        .add_plot("rci_short", rci_short)
        .add_plot("rci_medium", rci_medium)
        .add_plot("rci_long", rci_long)
        .add_plot("overbought", overbought)
        .add_plot("oversold", oversold)
    }
}
