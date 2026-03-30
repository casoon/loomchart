// OBV Indicator (On Balance Volume)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct OBVIndicator;

impl IndicatorPlugin for OBVIndicator {
    fn id(&self) -> &str {
        "obv"
    }
    fn name(&self) -> &str {
        "On Balance Volume"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Cumulative volume based on price direction"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("obv", "OBV", "#2962FF").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let obv = ta::obv(&close, &volume);
        let obv_opt: Vec<Option<f64>> = obv.into_iter().map(Some).collect();

        IndicatorResult::new("On Balance Volume", "OBV", false).add_plot("obv", obv_opt)
    }
}
