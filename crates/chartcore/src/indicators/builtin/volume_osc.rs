// Volume Oscillator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct VolumeOscillatorIndicator;

impl IndicatorPlugin for VolumeOscillatorIndicator {
    fn id(&self) -> &str {
        "vol_osc"
    }
    fn name(&self) -> &str {
        "Volume Oscillator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Percentage difference between two volume MAs"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("short", "Short Length", 5).min(1).max(50),
            InputConfig::int("long", "Long Length", 10).min(1).max(100),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("vo", "Volume Osc", "#FF5722").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let short = context.input_int("short").unwrap_or(5) as usize;
        let long = context.input_int("long").unwrap_or(10) as usize;

        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();
        let vo = ta::volume_oscillator(&volume, short, long);

        IndicatorResult::new(&format!("Volume Osc ({}, {})", short, long), "VO", false)
            .add_plot("vo", vo)
    }
}

#[derive(Default)]
pub struct NetVolumeIndicator;

impl IndicatorPlugin for NetVolumeIndicator {
    fn id(&self) -> &str {
        "net_volume"
    }
    fn name(&self) -> &str {
        "Net Volume"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volume
    }
    fn description(&self) -> &str {
        "Volume with sign based on price direction"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("net", "Net Volume", "#9C27B0").line_width(1)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();
        let volume: Vec<f64> = context.candles.iter().map(|c| c.v).collect();

        let net = ta::net_volume(&close, &volume);
        let net_opt: Vec<Option<f64>> = net.into_iter().map(Some).collect();

        IndicatorResult::new("Net Volume", "NV", false).add_plot("net", net_opt)
    }
}
