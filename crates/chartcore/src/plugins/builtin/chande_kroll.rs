// Chande Kroll Stop Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct ChandeKrollIndicator;

impl IndicatorPlugin for ChandeKrollIndicator {
    fn id(&self) -> &str {
        "chande_kroll"
    }

    fn name(&self) -> &str {
        "Chande Kroll Stop"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Trend-following stop-loss indicator based on ATR"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("p", "First High Low Period", 10)
                .min(1)
                .max(100),
            InputConfig::int("q", "Stop Period", 9).min(1).max(100),
            InputConfig::float("x", "ATR Multiplier", 1.0)
                .min(0.1)
                .max(10.0),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("stop_long", "Stop Long", "#4CAF50").line_width(2),
            PlotConfig::new("stop_short", "Stop Short", "#F44336").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let p = context.input_int("p").unwrap_or(10) as usize;
        let q = context.input_int("q").unwrap_or(9) as usize;
        let x = context.input_float("x").unwrap_or(1.0);

        let highs = context.high();
        let lows = context.low();
        let closes = context.close();

        let (stop_long, stop_short) = ta::chande_kroll_stop(&highs, &lows, &closes, p, q, x);

        IndicatorResult::new(
            &format!("Chande Kroll ({}, {}, {})", p, q, x),
            &format!("CK({},{})", p, q),
            true,
        )
        .add_plot("stop_long", stop_long)
        .add_plot("stop_short", stop_short)
    }
}
