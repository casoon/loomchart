// Mass Index Indicator

use crate::plugins::*;

#[derive(Default)]
pub struct MassIndex;

impl IndicatorPlugin for MassIndex {
    fn id(&self) -> &str {
        "mass_index"
    }

    fn name(&self) -> &str {
        "Mass Index"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }

    fn description(&self) -> &str {
        "Identifies trend reversals using high-low range analysis. Values >27 indicate potential reversal"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![InputConfig::int("length", "Length", 25).min(1).max(500)]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("mass_index", "Mass Index", "#E91E63").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(25) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let n = high.len();

        // Calculate high-low range
        let mut range: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            range.push(high[i] - low[i]);
        }

        // Calculate EMA(range, 9)
        let mut ema1 = Vec::with_capacity(n);
        let alpha = 2.0 / (9.0 + 1.0);
        let mut ema_value = range[0];
        ema1.push(ema_value);

        for i in 1..n {
            ema_value = alpha * range[i] + (1.0 - alpha) * ema_value;
            ema1.push(ema_value);
        }

        // Calculate EMA(EMA(range, 9), 9)
        let mut ema2 = Vec::with_capacity(n);
        let mut ema_value2 = ema1[0];
        ema2.push(ema_value2);

        for i in 1..n {
            ema_value2 = alpha * ema1[i] + (1.0 - alpha) * ema_value2;
            ema2.push(ema_value2);
        }

        // Calculate ratio = EMA1 / EMA2
        let mut ratio = Vec::with_capacity(n);
        for i in 0..n {
            if ema2[i] != 0.0 {
                ratio.push(ema1[i] / ema2[i]);
            } else {
                ratio.push(1.0);
            }
        }

        // Calculate Mass Index = Sum(ratio, length)
        let mut mass_index = Vec::with_capacity(n);
        for i in 0..n {
            if i < length - 1 {
                mass_index.push(None);
                continue;
            }

            let mut sum = 0.0;
            for j in (i + 1 - length)..=i {
                sum += ratio[j];
            }

            mass_index.push(Some(sum));
        }

        IndicatorResult::new(&format!("Mass Index ({})", length), "MI", false)
            .add_plot("mass_index", mass_index)
    }
}
