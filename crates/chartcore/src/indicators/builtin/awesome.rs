// Awesome Oscillator Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct AwesomeOscillatorIndicator;

impl IndicatorPlugin for AwesomeOscillatorIndicator {
    fn id(&self) -> &str {
        "awesome"
    }
    fn name(&self) -> &str {
        "Awesome Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Difference between 5-period and 34-period SMA of midpoint"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("fast", "Fast Length", 5).min(1).max(50),
            InputConfig::int("slow", "Slow Length", 34).min(1).max(100),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("ao", "AO", "#2196F3").line_width(1)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let fast = context.input_int("fast").unwrap_or(5) as usize;
        let slow = context.input_int("slow").unwrap_or(34) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();

        let ao = ta::awesome_oscillator(&high, &low, fast, slow);

        IndicatorResult::new("Awesome Oscillator", "AO", false).add_plot("ao", ao)
    }
}
