// Elder Force Index

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ElderForceIndicator;

impl IndicatorPlugin for ElderForceIndicator {
    fn id(&self) -> &str {
        "elder_force"
    }
    fn name(&self) -> &str {
        "Elder Force Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Price change times volume"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 13).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("efi", "EFI", "#E91E63").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(13) as usize;

        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let efi = ta::elder_force_index(&close, &volume, length);

        IndicatorResult::new(&format!("EFI ({})", length), "EFI", false).add_plot("efi", efi)
    }
}
