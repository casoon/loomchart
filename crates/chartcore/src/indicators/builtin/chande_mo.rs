// Chande Momentum Oscillator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ChandeMOIndicator;

impl IndicatorPlugin for ChandeMOIndicator {
    fn id(&self) -> &str {
        "chande_mo"
    }
    fn name(&self) -> &str {
        "Chande Momentum Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Measures momentum on both up and down days"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 9).min(1).max(100),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("cmo", "CMO", "#FF5722").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(9) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);
        let cmo = ta::chande_mo(&source, length);

        IndicatorResult::new(
            &format!("CMO ({})", length),
            &format!("CMO({})", length),
            false,
        )
        .add_plot("cmo", cmo)
    }
}
