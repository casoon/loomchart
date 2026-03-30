// Vortex Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct VortexIndicator;

impl IndicatorPlugin for VortexIndicator {
    fn id(&self) -> &str {
        "vortex"
    }

    fn name(&self) -> &str {
        "Vortex Indicator"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Identifies trend direction and strength using VI+ and VI- lines"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("vi_plus", "VI+", "#4CAF50").line_width(2),
            PlotConfig::new("vi_minus", "VI-", "#F44336").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let highs = context.high();
        let lows = context.low();
        let closes = context.close();

        let (vi_plus, vi_minus) = ta::vortex(&highs, &lows, &closes, length);

        IndicatorResult::new(
            &format!("Vortex ({})", length),
            &format!("VTX({})", length),
            false,
        )
        .add_plot("vi_plus", vi_plus)
        .add_plot("vi_minus", vi_minus)
    }
}
