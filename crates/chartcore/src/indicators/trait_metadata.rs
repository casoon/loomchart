//! Trait-based Indicator Metadata System
//!
//! Each indicator implements the `HasMetadata` trait to provide:
//! - Name, description, category
//! - Input parameters (configurable properties)
//! - Output information
//! - Interpretation guides

use serde::{Deserialize, Serialize};

/// Indicator category for UI organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Trend,
    Momentum,
    Volatility,
    Volume,
    Scientific,
}

/// Input parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputParam {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub param_type: ParamType,
}

/// Parameter type with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParamType {
    #[serde(rename = "integer")]
    Integer {
        min: i32,
        max: i32,
        default: i32,
        step: i32,
    },
    #[serde(rename = "float")]
    Float {
        min: f64,
        max: f64,
        default: f64,
        step: f64,
    },
    #[serde(rename = "bool")]
    Bool {
        default: bool,
    },
    #[serde(rename = "select")]
    Select {
        options: Vec<String>,
        default: usize,
    },
}

/// Indicator metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub category: Category,
    pub description: String,
    pub inputs: Vec<InputParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<String>,
}

/// Trait that all indicators must implement
pub trait HasMetadata {
    /// Returns the indicator's metadata
    fn metadata() -> Metadata;
}

// Helper function to create metadata easily
pub fn meta(
    id: &str,
    name: &str,
    category: Category,
    description: &str,
    inputs: Vec<InputParam>,
) -> Metadata {
    Metadata {
        id: id.to_string(),
        name: name.to_string(),
        category,
        description: description.to_string(),
        inputs,
        interpretation: None,
    }
}

// Helper to create integer input
pub fn int_input(id: &str, name: &str, default: i32, min: i32, max: i32) -> InputParam {
    InputParam {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("{} (default: {})", name, default),
        param_type: ParamType::Integer {
            min,
            max,
            default,
            step: 1,
        },
    }
}

// Helper to create float input
pub fn float_input(id: &str, name: &str, default: f64, min: f64, max: f64, step: f64) -> InputParam {
    InputParam {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("{} (default: {})", name, default),
        param_type: ParamType::Float {
            min,
            max,
            default,
            step,
        },
    }
}

// Helper to create boolean input
pub fn bool_input(id: &str, name: &str, default: bool) -> InputParam {
    InputParam {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("{} (default: {})", name, default),
        param_type: ParamType::Bool { default },
    }
}
