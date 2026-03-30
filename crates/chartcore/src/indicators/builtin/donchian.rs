// Donchian Channels Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct DonchianIndicator;

impl IndicatorPlugin for DonchianIndicator {
    fn id(&self) -> &str {
        "donchian"
    }
    fn name(&self) -> &str {
        "Donchian Channels"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::ChannelsAndBands
    }
    fn description(&self) -> &str {
        "Highest high and lowest low over period"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 20).min(1).max(200)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("upper", "Upper", "#26A69A").line_width(1),
            PlotConfig::new("middle", "Middle", "#787B86")
                .line_width(1)
                .dashed(),
            PlotConfig::new("lower", "Lower", "#EF5350").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();

        let (upper, middle, lower) = ta::donchian_channels(&high, &low, length);

        IndicatorResult::new(&format!("Donchian ({})", length), "DC", true)
            .add_plot("upper", upper)
            .add_plot("middle", middle)
            .add_plot("lower", lower)
    }
}
