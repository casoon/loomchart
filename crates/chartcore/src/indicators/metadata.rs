//! Indicator Metadata System
//!
//! Provides type-safe configuration and description for all indicators.
//! Each indicator can define its parameters, ranges, defaults, and descriptions.

use serde::{Deserialize, Serialize};

/// Indicator category for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorCategory {
    /// Traditional momentum indicators (RSI, Stochastic, etc.)
    Momentum,
    /// Trend indicators (MA, MACD, etc.)
    Trend,
    /// Volatility indicators (Bollinger, ATR, etc.)
    Volatility,
    /// Volume indicators
    Volume,
    /// Scientific/complexity indicators (Entropy, etc.)
    Scientific,
    /// Custom user-defined indicators
    Custom,
}

/// Parameter type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterType {
    /// Integer parameter with min, max, default
    Integer {
        min: i32,
        max: i32,
        default: i32,
        step: i32,
    },
    /// Float parameter with min, max, default
    Float {
        min: f64,
        max: f64,
        default: f64,
        step: f64,
    },
    /// Boolean parameter
    Boolean { default: bool },
    /// Choice from list of options
    Choice {
        options: Vec<String>,
        default: usize,
    },
    /// Color parameter
    Color { default: String },
    /// String parameter
    String { default: String },
}

/// Single parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter identifier (e.g., "period", "bins")
    pub id: String,
    /// Display name (e.g., "Period", "Number of Bins")
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Parameter type and constraints
    pub param_type: ParameterType,
    /// Tooltip help text
    pub tooltip: Option<String>,
}

/// Indicator interpretation guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretationGuide {
    /// High value interpretation (e.g., "> 0.8: Random market")
    pub high: String,
    /// Medium value interpretation
    pub medium: String,
    /// Low value interpretation
    pub low: String,
}

/// Indicator metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMetadata {
    /// Unique identifier (e.g., "shannon_entropy")
    pub id: String,
    /// Display name (e.g., "Shannon Entropy")
    pub name: String,
    /// Short description (1 line)
    pub short_description: String,
    /// Long description (multiple paragraphs)
    pub long_description: String,
    /// Category for UI organization
    pub category: IndicatorCategory,
    /// Parameter definitions
    pub parameters: Vec<ParameterDefinition>,
    /// Interpretation guide
    pub interpretation: Option<InterpretationGuide>,
    /// Recommended use cases
    pub use_cases: Vec<String>,
    /// Output range (e.g., [0, 1], [0, 100])
    pub output_range: (f64, f64),
    /// Whether output is normalized
    pub normalized: bool,
    /// Recommended timeframes
    pub recommended_timeframes: Vec<String>,
    /// Algorithm complexity (for performance info)
    pub complexity: String,
    /// Related indicators
    pub related: Vec<String>,
}

impl IndicatorMetadata {
    /// Get parameter by ID
    pub fn get_parameter(&self, id: &str) -> Option<&ParameterDefinition> {
        self.parameters.iter().find(|p| p.id == id)
    }

    /// Get default parameter value
    pub fn get_default_value(&self, id: &str) -> Option<ParameterValue> {
        let param = self.get_parameter(id)?;
        match &param.param_type {
            ParameterType::Integer { default, .. } => Some(ParameterValue::Integer(*default)),
            ParameterType::Float { default, .. } => Some(ParameterValue::Float(*default)),
            ParameterType::Boolean { default } => Some(ParameterValue::Boolean(*default)),
            ParameterType::Choice { options, default } => {
                Some(ParameterValue::Choice(options[*default].clone()))
            }
            ParameterType::Color { default } => Some(ParameterValue::Color(default.clone())),
            ParameterType::String { default } => Some(ParameterValue::String(default.clone())),
        }
    }
}

/// Parameter value (runtime)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    Choice(String),
    Color(String),
    String(String),
}

/// Scientific Indicators Metadata
pub mod scientific {
    use super::*;

    /// Shannon Entropy metadata
    pub fn shannon_entropy() -> IndicatorMetadata {
        IndicatorMetadata {
            id: "shannon_entropy".to_string(),
            name: "Shannon Entropy".to_string(),
            short_description:
                "Measures information content and unpredictability of price movements".to_string(),
            long_description:
                r#"Shannon Entropy quantifies the randomness in price data using information theory.
Higher entropy indicates more random, less predictable markets.
Lower entropy indicates structured, more predictable patterns.

Based on Claude Shannon's information theory (1948), this indicator measures the
average information content per price bin. It's particularly useful for regime detection
and strategy selection."#
                    .to_string(),
            category: IndicatorCategory::Scientific,
            parameters: vec![
                ParameterDefinition {
                    id: "period".to_string(),
                    name: "Period".to_string(),
                    description: "Number of bars to analyze".to_string(),
                    param_type: ParameterType::Integer {
                        min: 10,
                        max: 200,
                        default: 20,
                        step: 1,
                    },
                    tooltip: Some(
                        "Larger periods give more stable readings but slower response".to_string(),
                    ),
                },
                ParameterDefinition {
                    id: "bins".to_string(),
                    name: "Bins".to_string(),
                    description: "Number of histogram bins for probability calculation".to_string(),
                    param_type: ParameterType::Integer {
                        min: 5,
                        max: 50,
                        default: 10,
                        step: 1,
                    },
                    tooltip: Some(
                        "More bins = finer granularity but require more data".to_string(),
                    ),
                },
            ],
            interpretation: Some(InterpretationGuide {
                high: "> 0.8: Random, unpredictable market. Avoid trend-following strategies."
                    .to_string(),
                medium: "0.4-0.8: Normal market behavior with some structure.".to_string(),
                low: "< 0.4: Highly structured patterns. Good for trend-following.".to_string(),
            }),
            use_cases: vec![
                "Detect market regime changes".to_string(),
                "Select appropriate trading strategy".to_string(),
                "Measure market efficiency".to_string(),
                "Risk assessment based on unpredictability".to_string(),
            ],
            output_range: (0.0, 1.0),
            normalized: true,
            recommended_timeframes: vec![
                "5m".to_string(),
                "15m".to_string(),
                "1h".to_string(),
                "4h".to_string(),
            ],
            complexity: "O(n) per update, O(bins) memory".to_string(),
            related: vec!["lempel_ziv".to_string(), "permutation_entropy".to_string()],
        }
    }

    /// Lempel-Ziv Complexity metadata
    pub fn lempel_ziv() -> IndicatorMetadata {
        IndicatorMetadata {
            id: "lempel_ziv".to_string(),
            name: "Lempel-Ziv Complexity".to_string(),
            short_description:
                "Measures complexity by counting unique patterns (compression-based)".to_string(),
            long_description:
                r#"Lempel-Ziv Complexity measures how compressible a price sequence is.
Random data is incompressible (high complexity).
Structured data with patterns is compressible (low complexity).

Based on the LZ76 compression algorithm, this indicator counts unique patterns
in the binary representation of price changes. It's more robust to outliers
than Shannon Entropy and better at detecting repeating patterns."#
                    .to_string(),
            category: IndicatorCategory::Scientific,
            parameters: vec![
                ParameterDefinition {
                    id: "period".to_string(),
                    name: "Period".to_string(),
                    description: "Number of bars to analyze".to_string(),
                    param_type: ParameterType::Integer {
                        min: 20,
                        max: 500,
                        default: 100,
                        step: 1,
                    },
                    tooltip: Some(
                        "Longer periods recommended for LZ complexity (50-200)".to_string(),
                    ),
                },
                ParameterDefinition {
                    id: "threshold".to_string(),
                    name: "Threshold".to_string(),
                    description: "Binary conversion threshold (0 = auto/median)".to_string(),
                    param_type: ParameterType::Float {
                        min: -1.0,
                        max: 1.0,
                        default: 0.0,
                        step: 0.01,
                    },
                    tooltip: Some(
                        "0 uses adaptive median threshold. Set manually for fixed threshold."
                            .to_string(),
                    ),
                },
            ],
            interpretation: Some(InterpretationGuide {
                high: "> 0.7: Random, chaotic behavior. Patterns are breaking down.".to_string(),
                medium: "0.4-0.7: Normal market with some repeating patterns.".to_string(),
                low: "< 0.4: Highly structured, repeating patterns detected.".to_string(),
            }),
            use_cases: vec![
                "Detect repeating price patterns".to_string(),
                "Measure market efficiency".to_string(),
                "Identify regime transitions".to_string(),
                "Complement Shannon Entropy for confirmation".to_string(),
            ],
            output_range: (0.0, 1.0),
            normalized: true,
            recommended_timeframes: vec![
                "15m".to_string(),
                "1h".to_string(),
                "4h".to_string(),
                "1d".to_string(),
            ],
            complexity: "O(n²) worst case, O(n log n) typical".to_string(),
            related: vec![
                "shannon_entropy".to_string(),
                "permutation_entropy".to_string(),
            ],
        }
    }

    /// Permutation Entropy metadata
    pub fn permutation_entropy() -> IndicatorMetadata {
        IndicatorMetadata {
            id: "permutation_entropy".to_string(),
            name: "Permutation Entropy".to_string(),
            short_description: "Measures complexity through ordinal patterns (robust to noise)"
                .to_string(),
            long_description:
                r#"Permutation Entropy analyzes the order of values rather than their magnitudes.
This makes it more robust to outliers and noise than Shannon Entropy.

The indicator looks at sequences of prices and counts how often different
ordinal patterns appear. For example, with dimension 3, it detects patterns like
"up-up", "up-down", "down-up", etc. High entropy means random patterns,
low entropy means predictable sequences."#
                    .to_string(),
            category: IndicatorCategory::Scientific,
            parameters: vec![
                ParameterDefinition {
                    id: "period".to_string(),
                    name: "Period".to_string(),
                    description: "Number of bars to analyze".to_string(),
                    param_type: ParameterType::Integer {
                        min: 20,
                        max: 500,
                        default: 100,
                        step: 1,
                    },
                    tooltip: Some(
                        "Should be >> factorial(dimension) for reliable statistics".to_string(),
                    ),
                },
                ParameterDefinition {
                    id: "dimension".to_string(),
                    name: "Embedding Dimension".to_string(),
                    description: "Length of ordinal patterns to detect".to_string(),
                    param_type: ParameterType::Integer {
                        min: 2,
                        max: 7,
                        default: 3,
                        step: 1,
                    },
                    tooltip: Some(
                        "3-5 recommended. Higher = more detailed but needs more data".to_string(),
                    ),
                },
                ParameterDefinition {
                    id: "delay".to_string(),
                    name: "Time Delay".to_string(),
                    description: "Delay between pattern elements".to_string(),
                    param_type: ParameterType::Integer {
                        min: 1,
                        max: 10,
                        default: 1,
                        step: 1,
                    },
                    tooltip: Some("Usually 1. Increase for longer-term patterns".to_string()),
                },
            ],
            interpretation: Some(InterpretationGuide {
                high: "> 0.8: Random, unpredictable sequences. Stochastic behavior.".to_string(),
                medium: "0.4-0.8: Normal market with some ordinal structure.".to_string(),
                low: "< 0.4: Strong ordinal patterns. Deterministic behavior.".to_string(),
            }),
            use_cases: vec![
                "Detect deterministic vs stochastic behavior".to_string(),
                "Measure complexity in noisy markets".to_string(),
                "Early warning for regime changes".to_string(),
                "More robust alternative to Shannon Entropy".to_string(),
            ],
            output_range: (0.0, 1.0),
            normalized: true,
            recommended_timeframes: vec![
                "15m".to_string(),
                "1h".to_string(),
                "4h".to_string(),
                "1d".to_string(),
            ],
            complexity: "O(n * d!) where d is dimension".to_string(),
            related: vec!["shannon_entropy".to_string(), "lempel_ziv".to_string()],
        }
    }
}

/// Get all available indicator metadata
pub fn all_indicators() -> Vec<IndicatorMetadata> {
    let mut indicators = vec![];

    // Add scientific indicators (manual metadata)
    indicators.push(scientific::shannon_entropy());
    indicators.push(scientific::lempel_ziv());
    indicators.push(scientific::permutation_entropy());

    // Add all builtin indicators from plugin registry
    indicators.extend(crate::indicators::registry::collect_all_indicators());

    indicators
}

// Deprecated - kept for reference

/// Get indicator metadata by ID
pub fn get_indicator(id: &str) -> Option<IndicatorMetadata> {
    all_indicators().into_iter().find(|ind| ind.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy_metadata() {
        let meta = scientific::shannon_entropy();

        assert_eq!(meta.id, "shannon_entropy");
        assert_eq!(meta.name, "Shannon Entropy");
        assert_eq!(meta.category, IndicatorCategory::Scientific);
        assert_eq!(meta.parameters.len(), 2);
        assert!(meta.interpretation.is_some());
        assert!(meta.normalized);
        assert_eq!(meta.output_range, (0.0, 1.0));
    }

    #[test]
    fn test_parameter_retrieval() {
        let meta = scientific::shannon_entropy();

        let period = meta.get_parameter("period").unwrap();
        assert_eq!(period.name, "Period");

        if let ParameterType::Integer { default, .. } = period.param_type {
            assert_eq!(default, 20);
        } else {
            panic!("Expected Integer type");
        }
    }

    #[test]
    fn test_default_value() {
        let meta = scientific::lempel_ziv();

        let threshold_value = meta.get_default_value("threshold").unwrap();
        if let ParameterValue::Float(val) = threshold_value {
            assert_eq!(val, 0.0);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_all_indicators() {
        let indicators = all_indicators();
        assert_eq!(indicators.len(), 3);

        let ids: Vec<_> = indicators.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"shannon_entropy"));
        assert!(ids.contains(&"lempel_ziv"));
        assert!(ids.contains(&"permutation_entropy"));
    }

    #[test]
    fn test_get_indicator_by_id() {
        let meta = get_indicator("permutation_entropy").unwrap();
        assert_eq!(meta.name, "Permutation Entropy");

        let none = get_indicator("nonexistent");
        assert!(none.is_none());
    }
}
