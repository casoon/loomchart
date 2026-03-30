// Woodies CCI Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct WoodiesCCIIndicator;

impl IndicatorPlugin for WoodiesCCIIndicator {
    fn id(&self) -> &str {
        "woodies_cci"
    }

    fn name(&self) -> &str {
        "Woodies CCI"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Ken Wood's CCI system with trend CCI, turbo CCI, and histogram"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("cci_length", "CCI Length", 14)
                .min(2)
                .max(100),
            InputConfig::int("turbo_length", "Turbo Length", 6)
                .min(2)
                .max(50),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("cci", "CCI", "#2196F3").line_width(2),
            PlotConfig::new("turbo", "Turbo CCI", "#FF9800").line_width(1),
            PlotConfig::new("histogram", "Histogram", "#9C27B0").histogram(),
            PlotConfig::new("plus100", "+100", "#888888")
                .line_width(1)
                .style("dashed"),
            PlotConfig::new("minus100", "-100", "#888888")
                .line_width(1)
                .style("dashed"),
            PlotConfig::new("zero", "Zero", "#888888")
                .line_width(1)
                .style("dashed"),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let cci_length = context.input_int("cci_length").unwrap_or(14) as usize;
        let turbo_length = context.input_int("turbo_length").unwrap_or(6) as usize;

        let highs = context.high();
        let lows = context.low();
        let closes = context.close();

        let (cci, turbo) = ta::woodies_cci(&highs, &lows, &closes, cci_length, turbo_length);

        // Histogram is CCI - Turbo
        let histogram: Vec<Option<f64>> = cci
            .iter()
            .zip(turbo.iter())
            .map(|(c, t)| match (c, t) {
                (Some(cv), Some(tv)) => Some(cv - tv),
                _ => None,
            })
            .collect();

        let len = closes.len();
        let plus100: Vec<Option<f64>> = vec![Some(100.0); len];
        let minus100: Vec<Option<f64>> = vec![Some(-100.0); len];
        let zero: Vec<Option<f64>> = vec![Some(0.0); len];

        IndicatorResult::new(
            &format!("Woodies CCI ({}, {})", cci_length, turbo_length),
            &format!("WCCI({},{})", cci_length, turbo_length),
            false,
        )
        .add_plot("cci", cci)
        .add_plot("turbo", turbo)
        .add_plot("histogram", histogram)
        .add_plot("plus100", plus100)
        .add_plot("minus100", minus100)
        .add_plot("zero", zero)
    }
}
