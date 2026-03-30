// ADX Indicator (Average Directional Index)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ADXIndicator;

impl IndicatorPlugin for ADXIndicator {
    fn id(&self) -> &str {
        "adx"
    }
    fn name(&self) -> &str {
        "Average Directional Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Measures trend strength regardless of direction"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("di_length", "DI Length", 14)
                .min(1)
                .max(100),
            InputConfig::int("adx_smooth", "ADX Smoothing", 14)
                .min(1)
                .max(100),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("adx", "ADX", "#FF9800").line_width(2),
            PlotConfig::new("plus_di", "+DI", "#26A69A").line_width(1),
            PlotConfig::new("minus_di", "-DI", "#EF5350").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let di_length = context.input_int("di_length").unwrap_or(14) as usize;
        let adx_smooth = context.input_int("adx_smooth").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (adx, plus_di, minus_di) = ta::adx(&high, &low, &close, di_length, adx_smooth);

        IndicatorResult::new(&format!("ADX ({})", di_length), "ADX", false)
            .add_plot("adx", adx)
            .add_plot("plus_di", plus_di)
            .add_plot("minus_di", minus_di)
    }
}
