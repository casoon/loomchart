//! Chart plugin trait and configuration.
//!
//! Define custom plugins that can draw on the chart.

use loom_core::{Candle, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, boxed::Box};

use crate::context::DrawingContext;

/// Plugin configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PluginConfig {
    /// Plugin is enabled
    pub enabled: bool,
    /// Show on main chart or separate pane
    pub overlay: bool,
    /// Z-index for layering
    pub z_index: i32,
    /// Custom settings (JSON-like)
    pub settings: Vec<(String, ConfigValue)>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            overlay: true,
            z_index: 0,
            settings: Vec::new(),
        }
    }
}

impl PluginConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn overlay(mut self, overlay: bool) -> Self {
        self.overlay = overlay;
        self
    }

    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    pub fn set(mut self, key: impl Into<String>, value: ConfigValue) -> Self {
        self.settings.push((key.into(), value));
        self
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.settings.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// Configuration value types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(crate::primitives::Color),
}

impl ConfigValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Plugin state for persistence
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PluginState {
    /// Stored values
    pub values: Vec<(String, ConfigValue)>,
}

impl PluginState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: ConfigValue) {
        let key = key.into();
        if let Some(pos) = self.values.iter().position(|(k, _)| k == &key) {
            self.values[pos].1 = value;
        } else {
            self.values.push((key, value));
        }
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// Chart plugin trait
///
/// Implement this to create custom chart visualizations.
pub trait ChartPlugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin description
    fn description(&self) -> &str {
        ""
    }

    /// Plugin version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Get current configuration
    fn config(&self) -> &PluginConfig;

    /// Update configuration
    fn set_config(&mut self, config: PluginConfig);

    /// Initialize plugin (called once when added)
    fn init(&mut self, _ctx: &mut DrawingContext) {}

    /// Called when historical data is loaded
    fn on_historical_data(&mut self, _ctx: &mut DrawingContext, _candles: &[Candle]) {}

    /// Called on each new candle
    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle);

    /// Called on candle update (partial candle)
    fn on_candle_update(&mut self, _ctx: &mut DrawingContext, _candle: &Candle) {}

    /// Called when user interacts with the chart
    fn on_interaction(&mut self, _ctx: &mut DrawingContext, _event: &InteractionEvent) {}

    /// Save plugin state
    fn save_state(&self) -> PluginState {
        PluginState::new()
    }

    /// Restore plugin state
    fn load_state(&mut self, _state: PluginState) {}

    /// Cleanup (called when plugin is removed)
    fn destroy(&mut self, _ctx: &mut DrawingContext) {}
}

/// User interaction events
#[derive(Debug, Clone)]
pub enum InteractionEvent {
    /// Click on chart
    Click {
        time: Timestamp,
        price: f64,
        button: MouseButton,
    },
    /// Hover over chart
    Hover {
        time: Timestamp,
        price: f64,
    },
    /// Click on a drawing
    DrawingClick {
        drawing_id: crate::primitives::DrawingId,
        button: MouseButton,
    },
    /// Drag a drawing
    DrawingDrag {
        drawing_id: crate::primitives::DrawingId,
        delta_time: i64,
        delta_price: f64,
    },
    /// Keyboard event
    KeyPress {
        key: String,
        modifiers: KeyModifiers,
    },
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Plugin registry
pub struct PluginRegistry {
    plugins: Vec<Box<dyn ChartPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Add a plugin
    pub fn add<P: ChartPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    /// Remove a plugin by name
    pub fn remove(&mut self, name: &str) {
        self.plugins.retain(|p| p.name() != name);
    }

    /// Get plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn ChartPlugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// Get mutable plugin by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn ChartPlugin> {
        self.plugins.iter_mut().find(|p| p.name() == name).map(|p| p.as_mut())
    }

    /// Iterate over plugins
    pub fn iter(&self) -> impl Iterator<Item = &dyn ChartPlugin> {
        self.plugins.iter().map(|p| p.as_ref())
    }

    /// Iterate mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn ChartPlugin> {
        self.plugins.iter_mut().map(|p| p.as_mut())
    }

    /// Process historical data for all plugins
    pub fn on_historical_data(&mut self, ctx: &mut DrawingContext, candles: &[Candle]) {
        for plugin in &mut self.plugins {
            if plugin.config().enabled {
                plugin.on_historical_data(ctx, candles);
            }
        }
    }

    /// Process new candle for all plugins
    pub fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        for plugin in &mut self.plugins {
            if plugin.config().enabled {
                plugin.on_candle(ctx, candle);
            }
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
