// Ichimoku Cloud Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct IchimokuIndicator;

impl IndicatorPlugin for IchimokuIndicator {
    fn id(&self) -> &str {
        "ichimoku"
    }
    fn name(&self) -> &str {
        "Ichimoku Cloud"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Comprehensive trend system with support/resistance"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("tenkan", "Tenkan Period", 9)
                .min(1)
                .max(100),
            InputConfig::int("kijun", "Kijun Period", 26)
                .min(1)
                .max(200),
            InputConfig::int("senkou_b", "Senkou B Period", 52)
                .min(1)
                .max(300),
            InputConfig::int("displacement", "Displacement", 26)
                .min(1)
                .max(100),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("tenkan", "Tenkan-sen", "#2962FF").line_width(1),
            PlotConfig::new("kijun", "Kijun-sen", "#B71C1C").line_width(1),
            PlotConfig::new("senkou_a", "Senkou Span A", "#A5D6A7").line_width(1),
            PlotConfig::new("senkou_b", "Senkou Span B", "#EF9A9A").line_width(1),
            PlotConfig::new("chikou", "Chikou Span", "#43A047").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let tenkan_period = context.input_int("tenkan").unwrap_or(9) as usize;
        let kijun_period = context.input_int("kijun").unwrap_or(26) as usize;
        let senkou_b_period = context.input_int("senkou_b").unwrap_or(52) as usize;
        let displacement = context.input_int("displacement").unwrap_or(26) as usize;

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (tenkan, kijun, senkou_a, senkou_b, chikou) = ta::ichimoku(
            &high,
            &low,
            &close,
            tenkan_period,
            kijun_period,
            senkou_b_period,
            displacement,
        );

        IndicatorResult::new("Ichimoku Cloud", "Ichimoku", true)
            .add_plot("tenkan", tenkan)
            .add_plot("kijun", kijun)
            .add_plot("senkou_a", senkou_a)
            .add_plot("senkou_b", senkou_b)
            .add_plot("chikou", chikou)
    }
}
