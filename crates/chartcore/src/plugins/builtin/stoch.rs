// Stochastic Oscillator Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct StochasticIndicator;

impl IndicatorPlugin for StochasticIndicator {
    fn id(&self) -> &str {
        "stoch"
    }
    fn name(&self) -> &str {
        "Stochastic"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillators
    }
    fn description(&self) -> &str {
        "Compares closing price to price range over a period"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("k_period", "%K Period", 14)
                .min(1)
                .max(100),
            InputConfig::int("k_smooth", "%K Smoothing", 1)
                .min(1)
                .max(10),
            InputConfig::int("d_period", "%D Period", 3).min(1).max(10),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("k", "%K", "#2962FF").line_width(2),
            PlotConfig::new("d", "%D", "#FF6D00").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let k_period = context.input_int("k_period").unwrap_or(14) as usize;
        let k_smooth = context.input_int("k_smooth").unwrap_or(1) as usize;
        let d_period = context.input_int("d_period").unwrap_or(3) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (k, d) = ta::stochastic(&high, &low, &close, k_period, k_smooth, d_period);

        IndicatorResult::new(
            &format!("Stochastic ({}, {}, {})", k_period, k_smooth, d_period),
            "Stoch",
            false,
        )
        .add_plot("k", k)
        .add_plot("d", d)
    }
}
