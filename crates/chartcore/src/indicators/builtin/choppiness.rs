// Choppiness Index Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ChoppinessIndicator;

impl IndicatorPlugin for ChoppinessIndicator {
    fn id(&self) -> &str {
        "choppiness"
    }
    fn name(&self) -> &str {
        "Choppiness Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Measures if market is trending or ranging"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(2).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("chop", "Choppiness", "#00BCD4").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let chop = ta::choppiness_index(&high, &low, &close, length);

        IndicatorResult::new(&format!("Choppiness ({})", length), "CHOP", false)
            .add_plot("chop", chop)
    }
}
