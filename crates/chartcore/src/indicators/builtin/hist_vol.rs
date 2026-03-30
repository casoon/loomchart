// Historical Volatility Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct HistoricalVolatilityIndicator;

impl IndicatorPlugin for HistoricalVolatilityIndicator {
    fn id(&self) -> &str {
        "hist_vol"
    }
    fn name(&self) -> &str {
        "Historical Volatility"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Annualized standard deviation of log returns"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(2).max(200),
            InputConfig::float("annual", "Annual Periods", 252.0).tooltip("Trading days per year"),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("hv", "HV", "#673AB7").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let annual = context.input_float("annual").unwrap_or(252.0);

        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let hv = ta::historical_volatility(&close, length, annual);

        IndicatorResult::new(&format!("HV ({})", length), "HV", false).add_plot("hv", hv)
    }
}
