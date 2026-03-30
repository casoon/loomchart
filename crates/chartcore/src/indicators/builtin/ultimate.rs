// Ultimate Oscillator Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct UltimateOscillatorIndicator;

impl IndicatorPlugin for UltimateOscillatorIndicator {
    fn id(&self) -> &str {
        "ultimate"
    }
    fn name(&self) -> &str {
        "Ultimate Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillators
    }
    fn description(&self) -> &str {
        "Multi-timeframe oscillator using three periods"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("short", "Short Length", 7).min(1).max(50),
            InputConfig::int("medium", "Medium Length", 14)
                .min(1)
                .max(100),
            InputConfig::int("long", "Long Length", 28).min(1).max(200),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("uo", "UO", "#00BCD4").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let short = context.input_int("short").unwrap_or(7) as usize;
        let medium = context.input_int("medium").unwrap_or(14) as usize;
        let long = context.input_int("long").unwrap_or(28) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let uo = ta::ultimate_oscillator(&high, &low, &close, short, medium, long);

        IndicatorResult::new(
            &format!("Ultimate Oscillator ({}, {}, {})", short, medium, long),
            "UO",
            false,
        )
        .add_plot("uo", uo)
    }
}
