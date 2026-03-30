// ADR Indicator (Average Daily Range)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ADRIndicator;

impl IndicatorPlugin for ADRIndicator {
    fn id(&self) -> &str {
        "adr"
    }
    fn name(&self) -> &str {
        "Average Daily Range"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Average of high-low range over period"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("adr", "ADR", "#00BCD4").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();

        let adr = ta::adr(&high, &low, length);

        IndicatorResult::new(
            &format!("ADR ({})", length),
            &format!("ADR({})", length),
            false,
        )
        .add_plot("adr", adr)
    }
}

#[derive(Default)]
pub struct ADRPercentIndicator;

impl IndicatorPlugin for ADRPercentIndicator {
    fn id(&self) -> &str {
        "adr_percent"
    }
    fn name(&self) -> &str {
        "ADR %"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Average Daily Range as percentage"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 14).min(1).max(100)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("adr_pct", "ADR %", "#FF9800").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let adr_pct = ta::adr_percent(&high, &low, &close, length);

        IndicatorResult::new(&format!("ADR% ({})", length), "ADR%", false)
            .add_plot("adr_pct", adr_pct)
    }
}
