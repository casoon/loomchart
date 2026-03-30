// Klinger Volume Oscillator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct KlingerIndicator;

impl IndicatorPlugin for KlingerIndicator {
    fn id(&self) -> &str {
        "klinger"
    }
    fn name(&self) -> &str {
        "Klinger Volume Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Long-term money flow trend"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("fast", "Fast Length", 34).min(1).max(100),
            InputConfig::int("slow", "Slow Length", 55).min(1).max(200),
            InputConfig::int("signal", "Signal Length", 13)
                .min(1)
                .max(50),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("kvo", "KVO", "#2962FF").line_width(2),
            PlotConfig::new("signal", "Signal", "#43A047").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let fast = context.input_int("fast").unwrap_or(34) as usize;
        let slow = context.input_int("slow").unwrap_or(55) as usize;
        let signal_len = context.input_int("signal").unwrap_or(13) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let (kvo, signal) =
            ta::klinger_oscillator(&high, &low, &close, &volume, fast, slow, signal_len);

        IndicatorResult::new("Klinger Volume Oscillator", "KVO", false)
            .add_plot("kvo", kvo)
            .add_plot("signal", signal)
    }
}
