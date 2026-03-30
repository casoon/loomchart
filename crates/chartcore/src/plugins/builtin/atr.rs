// ATR Indicator (Average True Range)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ATRIndicator;

impl IndicatorPlugin for ATRIndicator {
    fn id(&self) -> &str {
        "atr"
    }
    fn name(&self) -> &str {
        "Average True Range"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Measures market volatility"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("atr", "ATR", "#2962FF").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let atr = ta::atr(&high, &low, &close, length);

        IndicatorResult::new(
            &format!("ATR ({})", length),
            &format!("ATR({})", length),
            false,
        )
        .add_plot("atr", atr)
    }
}
