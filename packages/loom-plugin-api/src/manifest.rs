use serde::{Deserialize, Serialize};

/// Plugin type — determines which trait the plugin implements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginType {
    Indicator,
    Strategy,
    Renderer,
    DataSource,
}

/// Capability flag — plugins must declare which capabilities they need.
/// The host grants only the declared capabilities in the sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Read/write to the filesystem
    Fs,
    /// Outbound network access
    Network,
    /// System clock access
    Clock,
}

/// Plugin manifest — metadata declared by the plugin and validated by the host.
///
/// In Rust plugins this is embedded via `[package.metadata.loomchart]` in
/// Cargo.toml and serialised into the binary by `loom-plugin-sdk`. The host
/// reads it via the `loom_manifest` export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PluginManifest {
    /// API version this plugin was built against. Currently "1".
    pub api_version: String,

    /// Which plugin trait this binary implements.
    pub plugin_type: PluginType,

    /// Short machine-readable identifier, e.g. "supertrend".
    pub id: String,

    /// Human-readable display name.
    pub display_name: String,

    /// Short optional description.
    #[serde(default)]
    pub description: String,

    /// Semver version string.
    #[serde(default = "default_version")]
    pub version: String,

    /// Required capabilities. Empty by default (most indicators need none).
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// Name of the WASM export that serves as the plugin entry point.
    /// Defaults to "plugin_entry".
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_entry() -> String {
    "plugin_entry".to_string()
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_version != "1" {
            return Err(format!(
                "unsupported api-version '{}'; only '1' is supported",
                self.api_version
            ));
        }
        if self.id.is_empty() {
            return Err("plugin id must not be empty".to_string());
        }
        if self.display_name.is_empty() {
            return Err("plugin display-name must not be empty".to_string());
        }
        Ok(())
    }
}
