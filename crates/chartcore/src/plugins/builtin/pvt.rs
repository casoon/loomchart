// PVT Indicator (Price Volume Trend)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct PVTIndicator;

impl IndicatorPlugin for PVTIndicator {
    fn id(&self) -> &str {
        "pvt"
    }
    fn name(&self) -> &str {
        "Price Volume Trend"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Cumulative volume weighted by percentage price change"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("pvt", "PVT", "#00BCD4").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let pvt = ta::pvt(&close, &volume);
        let pvt_opt: Vec<Option<f64>> = pvt.into_iter().map(Some).collect();

        IndicatorResult::new("Price Volume Trend", "PVT", false).add_plot("pvt", pvt_opt)
    }
}
