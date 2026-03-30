// SMI Ergodic Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct SMIErgodicIndicator;

impl IndicatorPlugin for SMIErgodicIndicator {
    fn id(&self) -> &str {
        "smi_ergodic"
    }
    fn name(&self) -> &str {
        "SMI Ergodic Indicator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Stochastic Momentum Index with signal line"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("long", "Long Length", 20).min(1).max(100),
            InputConfig::int("short", "Short Length", 5).min(1).max(50),
            InputConfig::int("signal", "Signal Length", 5)
                .min(1)
                .max(50),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("smi", "SMI", "#2196F3").line_width(2),
            PlotConfig::new("signal", "Signal", "#FF9800").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let long = context.input_int("long").unwrap_or(20) as usize;
        let short = context.input_int("short").unwrap_or(5) as usize;
        let signal_len = context.input_int("signal").unwrap_or(5) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let (smi, signal) = ta::smi_ergodic(&source, long, short, signal_len);

        IndicatorResult::new("SMI Ergodic", "SMI", false)
            .add_plot("smi", smi)
            .add_plot("signal", signal)
    }
}

#[derive(Default)]
pub struct SMIErgodicOscillatorIndicator;

impl IndicatorPlugin for SMIErgodicOscillatorIndicator {
    fn id(&self) -> &str {
        "smi_ergodic_osc"
    }
    fn name(&self) -> &str {
        "SMI Ergodic Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "SMI Ergodic histogram (SMI - Signal)"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("long", "Long Length", 20).min(1).max(100),
            InputConfig::int("short", "Short Length", 5).min(1).max(50),
            InputConfig::int("signal", "Signal Length", 5)
                .min(1)
                .max(50),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("osc", "Oscillator", "#26A69A").line_width(1)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let long = context.input_int("long").unwrap_or(20) as usize;
        let short = context.input_int("short").unwrap_or(5) as usize;
        let signal_len = context.input_int("signal").unwrap_or(5) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let osc = ta::smi_ergodic_oscillator(&source, long, short, signal_len);

        IndicatorResult::new("SMI Ergodic Oscillator", "SMI Osc", false).add_plot("osc", osc)
    }
}
