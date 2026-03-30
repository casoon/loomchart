// Williams %R Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct WilliamsRIndicator;

impl IndicatorPlugin for WilliamsRIndicator {
    fn id(&self) -> &str {
        "williams_r"
    }
    fn name(&self) -> &str {
        "Williams %R"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillators
    }
    fn description(&self) -> &str {
        "Momentum indicator showing overbought/oversold levels"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("wr", "%R", "#9C27B0").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let wr = ta::williams_r(&high, &low, &close, length);

        IndicatorResult::new(
            &format!("Williams %R ({})", length),
            &format!("%R({})", length),
            false,
        )
        .add_plot("wr", wr)
    }
}
