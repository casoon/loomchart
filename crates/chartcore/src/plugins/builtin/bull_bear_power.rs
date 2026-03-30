// Bull Bear Power Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct BullBearPowerIndicator;

impl IndicatorPlugin for BullBearPowerIndicator {
    fn id(&self) -> &str {
        "bull_bear_power"
    }

    fn name(&self) -> &str {
        "Bull Bear Power"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Measures buying and selling pressure using EMA reference"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 13).min(1).max(200)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("bull", "Bull Power", "#4CAF50").histogram(),
            PlotConfig::new("bear", "Bear Power", "#F44336").histogram(),
            PlotConfig::new("total", "Total Power", "#2196F3").line_width(2),
            PlotConfig::new("zero", "Zero", "#888888")
                .line_width(1)
                .style("dashed"),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(13) as usize;

        let highs = context.high();
        let lows = context.low();
        let closes = context.close();

        let (bull_power, bear_power) = ta::bull_bear_power(&highs, &lows, &closes, length);

        // Total power is bull + bear
        let total_power: Vec<Option<f64>> = bull_power
            .iter()
            .zip(bear_power.iter())
            .map(|(b, br)| match (b, br) {
                (Some(bull), Some(bear)) => Some(bull + bear),
                _ => None,
            })
            .collect();

        let len = closes.len();
        let zero_line: Vec<Option<f64>> = vec![Some(0.0); len];

        IndicatorResult::new(
            &format!("Bull Bear Power ({})", length),
            &format!("BBP({})", length),
            false,
        )
        .add_plot("bull", bull_power)
        .add_plot("bear", bear_power)
        .add_plot("total", total_power)
        .add_plot("zero", zero_line)
    }
}
