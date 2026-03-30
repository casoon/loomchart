// Supertrend Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct SupertrendIndicator;

impl IndicatorPlugin for SupertrendIndicator {
    fn id(&self) -> &str {
        "supertrend"
    }
    fn name(&self) -> &str {
        "Supertrend"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Trend-following indicator using ATR"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("atr_period", "ATR Period", 10)
                .min(1)
                .max(100),
            InputConfig::float("factor", "Factor", 3.0).tooltip("ATR multiplier"),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("supertrend", "Supertrend", "#2962FF").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let atr_period = context.input_int("atr_period").unwrap_or(10) as usize;
        let factor = context.input_float("factor").unwrap_or(3.0);

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (supertrend, _direction) = ta::supertrend(&high, &low, &close, atr_period, factor);

        IndicatorResult::new(
            &format!("Supertrend ({}, {})", atr_period, factor),
            "ST",
            true,
        )
        .add_plot("supertrend", supertrend)
    }
}
