// Plugin Registry
//
// Manages both built-in (Rust) and external (WASM) indicator plugins.

use super::builtin;
use super::{CalculationContext, IndicatorPlugin, IndicatorResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Plugin registry that holds all available plugins
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Arc<dyn IndicatorPlugin>>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register built-in plugins
    pub fn register_builtins(&self) {
        let builtins = builtin::get_builtin_plugins();

        let mut plugins = self.plugins.write().unwrap();
        for plugin in builtins {
            plugins.insert(plugin.id().to_string(), plugin);
        }
    }

    /// Register a plugin
    pub fn register(&self, plugin: Arc<dyn IndicatorPlugin>) {
        let id = plugin.id().to_string();
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(id, plugin);
    }

    /// Unregister a plugin
    pub fn unregister(&self, id: &str) -> bool {
        let mut plugins = self.plugins.write().unwrap();
        plugins.remove(id).is_some()
    }

    /// Get a plugin by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn IndicatorPlugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(id).cloned()
    }

    /// Get all plugin IDs
    pub fn list_ids(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }

    /// Get all plugins
    pub fn list_plugins(&self) -> Vec<Arc<dyn IndicatorPlugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    /// Calculate an indicator by ID
    pub fn calculate(&self, id: &str, context: &CalculationContext) -> Option<IndicatorResult> {
        let plugin = self.get(id)?;
        Some(plugin.calculate(context))
    }

    /// Check if a plugin exists
    pub fn contains(&self, id: &str) -> bool {
        let plugins = self.plugins.read().unwrap();
        plugins.contains_key(id)
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        let plugins = self.plugins.read().unwrap();
        plugins.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let registry = Self::new();
        registry.register_builtins();
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Candle;
    use crate::plugins::InputValue;

    #[test]
    fn test_registry_builtins() {
        let registry = PluginRegistry::default();

        assert!(registry.count() > 0);
        assert!(registry.contains("rsi"));
        assert!(registry.contains("ema"));
        assert!(registry.contains("sma"));
    }

    #[test]
    fn test_registry_get() {
        let registry = PluginRegistry::default();

        let rsi = registry.get("rsi");
        assert!(rsi.is_some());
        assert_eq!(rsi.unwrap().id(), "rsi");
    }

    #[test]
    fn test_registry_calculate() {
        let registry = PluginRegistry::default();

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

        let context = CalculationContext::new(&candles).with_input("length", InputValue::Int(14));

        let result = registry.calculate("rsi", &context);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(result.plots.contains_key("rsi"));
    }

    #[test]
    fn test_registry_list() {
        let registry = PluginRegistry::default();

        let ids = registry.list_ids();
        assert!(ids.contains(&"rsi".to_string()));
        assert!(ids.contains(&"ema".to_string()));
        assert!(ids.contains(&"sma".to_string()));
    }
}
