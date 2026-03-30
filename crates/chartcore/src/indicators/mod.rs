pub mod builtin;
pub mod metadata;
pub mod registry;
pub mod trait_metadata;

/// Indicator system with standardized output interface
///
/// This module provides a unified interface for all technical indicators,
/// allowing them to produce render-ready data in a consistent format.
pub mod output;

pub use builtin::{LempelZivComplexity, PermutationEntropy, ShannonEntropy};
pub use metadata::{
    all_indicators, get_indicator, IndicatorCategory, IndicatorMetadata, InterpretationGuide,
    ParameterDefinition, ParameterType, ParameterValue,
};
pub use output::{Indicator, IndicatorOutput, LineData, LineStyle, MarkerShape};
