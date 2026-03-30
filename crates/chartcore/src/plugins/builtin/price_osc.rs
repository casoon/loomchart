// Price Oscillator Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct PriceOscillatorIndicator;

impl IndicatorPlugin for PriceOscillatorIndicator {
    fn id(&self) -> &str {
        "price_osc"
    }
    fn name(&self) -> &str {
        "Price Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Difference between two EMAs"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("short", "Short Length", 12)
                .min(1)
                .max(100),
            InputConfig::int("long", "Long Length", 26).min(1).max(200),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("po", "PO", "#795548").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let short = context.input_int("short").unwrap_or(12) as usize;
        let long = context.input_int("long").unwrap_or(26) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let po = ta::price_oscillator(&source, short, long, true);

        IndicatorResult::new(
            &format!("Price Oscillator ({}, {})", short, long),
            "PO",
            false,
        )
        .add_plot("po", po)
    }
}
