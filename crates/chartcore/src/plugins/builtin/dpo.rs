// DPO Indicator (Detrended Price Oscillator)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct DPOIndicator;

impl IndicatorPlugin for DPOIndicator {
    fn id(&self) -> &str {
        "dpo"
    }
    fn name(&self) -> &str {
        "Detrended Price Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Removes trend to identify cycles"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 21).min(2).max(200),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("dpo", "DPO", "#009688").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(21) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let dpo = ta::dpo(&source, length);

        IndicatorResult::new(
            &format!("DPO ({})", length),
            &format!("DPO({})", length),
            false,
        )
        .add_plot("dpo", dpo)
    }
}
