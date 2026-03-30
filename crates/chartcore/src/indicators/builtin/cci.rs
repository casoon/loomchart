// CCI Indicator (Commodity Channel Index)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct CCIIndicator;

impl IndicatorPlugin for CCIIndicator {
    fn id(&self) -> &str {
        "cci"
    }
    fn name(&self) -> &str {
        "Commodity Channel Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Measures price deviation from average"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 20).min(1).max(200)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("cci", "CCI", "#7E57C2").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let cci = ta::cci(&high, &low, &close, length);

        IndicatorResult::new(
            &format!("CCI ({})", length),
            &format!("CCI({})", length),
            false,
        )
        .add_plot("cci", cci)
    }
}
