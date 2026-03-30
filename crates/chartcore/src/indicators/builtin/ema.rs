// EMA Indicator (Built-in Rust implementation)

use crate::core::Candle;
use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct EMAIndicator;

impl IndicatorPlugin for EMAIndicator {
    fn id(&self) -> &str {
        "ema"
    }

    fn name(&self) -> &str {
        "Exponential Moving Average"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::MovingAverages
    }

    fn description(&self) -> &str {
        "Weighted moving average giving more weight to recent prices"
    }

    fn overlay(&self) -> bool {
        true // Overlay on price chart
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 21).min(1).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("ema", "EMA", "#2196F3").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(21) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);

        let source = context.source(source_type);
        let ema = ta::ema(&source, length);

        IndicatorResult::new(
            &format!("Exponential Moving Average ({})", length),
            &format!("EMA({})", length),
            true,
        )
        .add_plot("ema", ema)
    }

    fn calculate_incremental(
        &self,
        context: &CalculationContext,
        new_candle: &Candle,
        previous_result: &IndicatorResult,
    ) -> Option<IndicatorResult> {
        let length = context.input_int("length").unwrap_or(21) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);

        // Get previous EMA value
        let prev_ema_values = previous_result.plots.get("ema")?;
        let last_ema = prev_ema_values.last()?.as_ref()?;

        // Calculate new EMA incrementally
        let k = 2.0 / (length as f64 + 1.0);
        let new_value = source_type.extract(new_candle);
        let new_ema = new_value * k + last_ema * (1.0 - k);

        // Append new value to result
        let mut new_ema_values = prev_ema_values.clone();
        new_ema_values.push(Some(new_ema));

        Some(
            IndicatorResult::new(
                &format!("Exponential Moving Average ({})", length),
                &format!("EMA({})", length),
                true,
            )
            .add_plot("ema", new_ema_values),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_indicator() {
        let indicator = EMAIndicator::default();

        assert_eq!(indicator.id(), "ema");
        assert_eq!(indicator.category(), IndicatorCategory::MovingAverages);
        assert_eq!(indicator.overlay(), true);
    }

    #[test]
    fn test_ema_calculation() {
        let indicator = EMAIndicator::default();

        let candles: Vec<Candle> = (0..50)
            .map(|i| Candle {
                time: i * 60,
                o: 100.0 + (i as f64),
                h: 101.0 + (i as f64),
                l: 99.0 + (i as f64),
                c: 100.0 + (i as f64),
                v: 1000.0,
            })
            .collect();

        let context = CalculationContext::new(&candles)
            .with_input("length", InputValue::Int(21))
            .with_input("source", InputValue::Source(SourceType::Close));

        let result = indicator.calculate(&context);

        assert!(result.plots.contains_key("ema"));
        assert_eq!(result.plots["ema"].len(), 50);

        // First 20 values should be None
        for i in 0..20 {
            assert!(result.plots["ema"][i].is_none());
        }

        // Value at index 20 should be SMA
        assert!(result.plots["ema"][20].is_some());
    }

    #[test]
    fn test_ema_incremental() {
        let indicator = EMAIndicator::default();

        let candles: Vec<Candle> = (0..30)
            .map(|i| Candle {
                time: i * 60,
                o: 100.0,
                h: 101.0,
                l: 99.0,
                c: 100.0,
                v: 1000.0,
            })
            .collect();

        let context = CalculationContext::new(&candles)
            .with_input("length", InputValue::Int(21))
            .with_input("source", InputValue::Source(SourceType::Close));

        let initial_result = indicator.calculate(&context);

        let new_candle = Candle {
            time: 30 * 60,
            o: 100.0,
            h: 101.0,
            l: 99.0,
            c: 105.0, // Price jump
            v: 1000.0,
        };

        let incremental_result = indicator
            .calculate_incremental(&context, &new_candle, &initial_result)
            .unwrap();

        assert_eq!(incremental_result.plots["ema"].len(), 31);
        assert!(incremental_result.plots["ema"][30].is_some());
    }
}
