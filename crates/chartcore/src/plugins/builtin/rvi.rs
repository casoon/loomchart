// RVI Indicator (Relative Vigor Index)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct RVIIndicator;

impl IndicatorPlugin for RVIIndicator {
    fn id(&self) -> &str {
        "rvi"
    }
    fn name(&self) -> &str {
        "Relative Vigor Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Measures conviction of price move using open/close vs high/low"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 10).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("rvi", "RVI", "#4CAF50").line_width(2),
            PlotConfig::new("signal", "Signal", "#FF5722").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(10) as usize;

        let open: Vec<f64> = context.candles.iter().map(|c| c.o).collect();
        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (rvi, signal) = ta::rvi(&open, &high, &low, &close, length);

        IndicatorResult::new(
            &format!("RVI ({})", length),
            &format!("RVI({})", length),
            false,
        )
        .add_plot("rvi", rvi)
        .add_plot("signal", signal)
    }
}
