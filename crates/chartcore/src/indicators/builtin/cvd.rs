// CVD Indicator (Cumulative Volume Delta)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct CVDIndicator;

impl IndicatorPlugin for CVDIndicator {
    fn id(&self) -> &str {
        "cvd"
    }
    fn name(&self) -> &str {
        "Cumulative Volume Delta"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Cumulative difference between buying and selling volume"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("cvd", "CVD", "#2196F3").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let open: Vec<f64> = context.candles.iter().map(|c| c.o).collect();
        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let cvd = ta::cumulative_volume_delta(&open, &close, &high, &low, &volume);
        let cvd_opt: Vec<Option<f64>> = cvd.into_iter().map(Some).collect();

        IndicatorResult::new("Cumulative Volume Delta", "CVD", false).add_plot("cvd", cvd_opt)
    }
}

#[derive(Default)]
pub struct VolumeDeltaIndicator;

impl IndicatorPlugin for VolumeDeltaIndicator {
    fn id(&self) -> &str {
        "volume_delta"
    }
    fn name(&self) -> &str {
        "Volume Delta"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Per-bar difference between buying and selling volume"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("delta", "Delta", "#4CAF50").line_width(1)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let open: Vec<f64> = context.candles.iter().map(|c| c.o).collect();
        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let delta = ta::volume_delta(&open, &close, &high, &low, &volume);
        let delta_opt: Vec<Option<f64>> = delta.into_iter().map(Some).collect();

        IndicatorResult::new("Volume Delta", "VD", false).add_plot("delta", delta_opt)
    }
}
