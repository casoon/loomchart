// RSI Indicator (Built-in Rust implementation)

use crate::plugins::*;
use crate::ta;
use std::collections::HashMap;

#[derive(Default)]
pub struct RSIIndicator;

impl IndicatorPlugin for RSIIndicator {
    fn id(&self) -> &str {
        "rsi"
    }

    fn name(&self) -> &str {
        "Relative Strength Index"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }

    fn description(&self) -> &str {
        "Momentum oscillator measuring the speed and magnitude of price changes"
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "RSI Length", 14)
                .min(1)
                .max(500)
                .tooltip("Number of periods for RSI calculation"),
            InputConfig::source("source", "Source", SourceType::Close),
            InputConfig::float("overbought", "Overbought Level", 70.0),
            InputConfig::float("oversold", "Oversold Level", 30.0),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("rsi", "RSI", "#7E57C2").line_width(2),
            PlotConfig::new("overbought", "Overbought", "#ef5350")
                .line_width(1)
                .dashed(),
            PlotConfig::new("oversold", "Oversold", "#26a69a")
                .line_width(1)
                .dashed(),
            PlotConfig::new("midline", "Midline", "#666666").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(14) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let overbought = context.input_float("overbought").unwrap_or(70.0);
        let oversold = context.input_float("oversold").unwrap_or(30.0);

        // Get source series
        let source = context.source(source_type);

        // Calculate RSI using TA library
        let rsi = ta::rsi(&source, length);

        // Create constant lines
        let overbought_line: Vec<Option<f64>> = vec![Some(overbought); context.candles.len()];
        let oversold_line: Vec<Option<f64>> = vec![Some(oversold); context.candles.len()];
        let midline: Vec<Option<f64>> = vec![Some(50.0); context.candles.len()];

        let mut result = IndicatorResult::new(
            &format!("Relative Strength Index ({})", length),
            &format!("RSI({})", length),
            false,
        );

        result.plots.insert("rsi".to_string(), rsi);
        result
            .plots
            .insert("overbought".to_string(), overbought_line);
        result.plots.insert("oversold".to_string(), oversold_line);
        result.plots.insert("midline".to_string(), midline);

        result
    }

    fn validate_inputs(&self, inputs: &HashMap<String, InputValue>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(InputValue::Int(length)) = inputs.get("length") {
            if *length < 1 {
                errors.push("Length must be at least 1".to_string());
            }
        }

        if let (Some(InputValue::Float(ob)), Some(InputValue::Float(os))) =
            (inputs.get("overbought"), inputs.get("oversold"))
        {
            if ob <= os {
                errors.push("Overbought level must be greater than oversold level".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_indicator() {
        let indicator = RSIIndicator::default();

        assert_eq!(indicator.id(), "rsi");
        assert_eq!(indicator.category(), IndicatorCategory::Momentum);
        assert_eq!(indicator.overlay(), false);

        // Test inputs
        let inputs = indicator.inputs();
        assert!(inputs.iter().any(|i| i.id == "length"));
        assert!(inputs.iter().any(|i| i.id == "source"));

        // Test plots
        let plots = indicator.plots();
        assert!(plots.iter().any(|p| p.id == "rsi"));
        assert!(plots.iter().any(|p| p.id == "overbought"));
    }

    #[test]
    fn test_rsi_calculation() {
        let indicator = RSIIndicator::default();

        let candles: Vec<Candle> = (0..30)
            .map(|i| Candle {
                time: i * 60,
                o: 100.0 + (i as f64 * 0.5),
                h: 101.0 + (i as f64 * 0.5),
                l: 99.0 + (i as f64 * 0.5),
                c: 100.0 + (i as f64 * 0.5),
                v: 1000.0,
            })
            .collect();

        let context = CalculationContext::new(&candles)
            .with_input("length", InputValue::Int(14))
            .with_input("source", InputValue::Source(SourceType::Close));

        let result = indicator.calculate(&context);

        assert!(result.plots.contains_key("rsi"));
        assert_eq!(result.plots["rsi"].len(), 30);

        // RSI should be between 0 and 100
        for (i, &val) in result.plots["rsi"].iter().enumerate() {
            if let Some(rsi_val) = val {
                assert!(
                    rsi_val >= 0.0 && rsi_val <= 100.0,
                    "RSI at index {} = {}",
                    i,
                    rsi_val
                );
            }
        }
    }

    #[test]
    fn test_input_validation() {
        let indicator = RSIIndicator::default();

        let mut inputs = HashMap::new();
        inputs.insert("length".to_string(), InputValue::Int(14));
        inputs.insert("overbought".to_string(), InputValue::Float(70.0));
        inputs.insert("oversold".to_string(), InputValue::Float(30.0));

        assert!(indicator.validate_inputs(&inputs).is_ok());

        // Invalid: overbought <= oversold
        inputs.insert("overbought".to_string(), InputValue::Float(30.0));
        assert!(indicator.validate_inputs(&inputs).is_err());
    }
}
