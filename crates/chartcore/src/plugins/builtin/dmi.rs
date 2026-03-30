// DMI Indicator (Directional Movement Index)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct DMIIndicator;

impl IndicatorPlugin for DMIIndicator {
    fn id(&self) -> &str {
        "dmi"
    }
    fn name(&self) -> &str {
        "Directional Movement Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Shows +DI and -DI for trend direction"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("plus_di", "+DI", "#26A69A").line_width(2),
            PlotConfig::new("minus_di", "-DI", "#EF5350").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (plus_di, minus_di) = ta::dmi(&high, &low, &close, length);

        IndicatorResult::new(&format!("DMI ({})", length), "DMI", false)
            .add_plot("plus_di", plus_di)
            .add_plot("minus_di", minus_di)
    }
}
