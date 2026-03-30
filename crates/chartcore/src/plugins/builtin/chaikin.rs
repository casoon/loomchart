// Chaikin Indicators (Money Flow & Oscillator)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ChaikinMFIndicator;

impl IndicatorPlugin for ChaikinMFIndicator {
    fn id(&self) -> &str {
        "cmf"
    }
    fn name(&self) -> &str {
        "Chaikin Money Flow"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Measures accumulation/distribution over period"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 20).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("cmf", "CMF", "#26A69A").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let cmf = ta::chaikin_mf(&high, &low, &close, &volume, length);

        IndicatorResult::new(&format!("CMF ({})", length), "CMF", false).add_plot("cmf", cmf)
    }
}

#[derive(Default)]
pub struct ChaikinOscillatorIndicator;

impl IndicatorPlugin for ChaikinOscillatorIndicator {
    fn id(&self) -> &str {
        "chaikin_osc"
    }
    fn name(&self) -> &str {
        "Chaikin Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Difference between fast and slow EMA of A/D line"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("fast", "Fast Length", 3).min(1).max(50),
            InputConfig::int("slow", "Slow Length", 10).min(1).max(100),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("co", "Chaikin Osc", "#FF9800").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let fast = context.input_int("fast").unwrap_or(3) as usize;
        let slow = context.input_int("slow").unwrap_or(10) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let co = ta::chaikin_oscillator(&high, &low, &close, &volume, fast, slow);

        IndicatorResult::new(&format!("Chaikin Osc ({}, {})", fast, slow), "CO", false)
            .add_plot("co", co)
    }
}
