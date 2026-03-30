// MFI Indicator (Money Flow Index)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct MFIIndicator;

impl IndicatorPlugin for MFIIndicator {
    fn id(&self) -> &str {
        "mfi"
    }
    fn name(&self) -> &str {
        "Money Flow Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Volume-weighted RSI"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("mfi", "MFI", "#26A69A").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let mfi = ta::mfi(&high, &low, &close, &volume, length);

        IndicatorResult::new(
            &format!("MFI ({})", length),
            &format!("MFI({})", length),
            false,
        )
        .add_plot("mfi", mfi)
    }
}
