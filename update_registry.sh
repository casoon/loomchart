#!/bin/bash

# Extract all indicator struct names
INDICATORS=$(grep -h "^pub struct.*Indicator" crates/chartcore/src/indicators/builtin/*.rs | \
    sed 's/pub struct //' | \
    sed 's/[; ].*//' | \
    sort -u)

# Generate registry code
cat > crates/chartcore/src/indicators/registry.rs << 'REGEOF'
//! Indicator Registry - Auto-generated from plugin implementations

use crate::indicators::builtin::*;
use crate::indicators::metadata::{IndicatorMetadata, IndicatorCategory, ParameterDefinition, ParameterType};
use crate::plugins::IndicatorPlugin;

/// Collect all indicator metadata from plugin implementations
pub fn collect_all_indicators() -> Vec<IndicatorMetadata> {
    let mut indicators = vec![];
    
    // Helper to convert plugin to metadata
    fn plugin_to_metadata<T: IndicatorPlugin + Default>(plugin: T) -> IndicatorMetadata {
        let inputs = plugin.inputs();
        
        let parameters: Vec<ParameterDefinition> = inputs
            .iter()
            .map(|input| {
                let param_type = match &input.config_type {
                    crate::plugins::InputType::Integer { default, min, max, .. } => {
                        ParameterType::Integer {
                            min: min.unwrap_or(0),
                            max: max.unwrap_or(1000),
                            default: *default,
                            step: 1,
                        }
                    }
                    crate::plugins::InputType::Float { default, min, max, step, .. } => {
                        ParameterType::Float {
                            min: min.unwrap_or(0.0),
                            max: max.unwrap_or(1000.0),
                            default: *default,
                            step: step.unwrap_or(0.1),
                        }
                    }
                    crate::plugins::InputType::Bool { default } => {
                        ParameterType::Boolean { default: *default }
                    }
                    crate::plugins::InputType::Source { .. } => {
                        ParameterType::Choice {
                            options: vec!["close".to_string(), "open".to_string(), "high".to_string(), "low".to_string()],
                            default: 0,
                        }
                    }
                    crate::plugins::InputType::Color { .. } => {
                        ParameterType::Color { default: "#000000".to_string() }
                    }
                };
                
                ParameterDefinition {
                    id: input.id.clone(),
                    name: input.name.clone(),
                    description: input.tooltip.clone().unwrap_or_else(|| input.name.clone()),
                    param_type,
                    tooltip: input.tooltip.clone(),
                }
            })
            .collect();
        
        let category = match plugin.category() {
            crate::plugins::IndicatorCategory::Momentum => IndicatorCategory::Momentum,
            crate::plugins::IndicatorCategory::Trend => IndicatorCategory::Trend,
            crate::plugins::IndicatorCategory::Volatility => IndicatorCategory::Volatility,
            crate::plugins::IndicatorCategory::Volume => IndicatorCategory::Volume,
            _ => IndicatorCategory::Custom,
        };
        
        IndicatorMetadata {
            id: plugin.id().to_string(),
            name: plugin.name().to_string(),
            short_description: plugin.description().to_string(),
            long_description: plugin.description().to_string(),
            category,
            parameters,
            interpretation: None,
            use_cases: vec![],
            output_range: (f64::NEG_INFINITY, f64::INFINITY),
            normalized: false,
            recommended_timeframes: vec!["5m".to_string(), "15m".to_string(), "1h".to_string(), "4h".to_string(), "1d".to_string()],
            complexity: "O(n)".to_string(),
            related: vec![],
        }
    }
    
    // Auto-generated indicator list
REGEOF

# Add all indicators
echo "$INDICATORS" | while read -r indicator; do
    echo "    indicators.push(plugin_to_metadata(${indicator}::default()));" >> crates/chartcore/src/indicators/registry.rs
done

# Close function
cat >> crates/chartcore/src/indicators/registry.rs << 'REGEOF'
    
    indicators
}
REGEOF

echo "Registry updated with $(echo "$INDICATORS" | wc -l) indicators"
