// Plugin discovery — scans well-known directories for *.wasm plugin files.
//
// Search order:
//  1. ./plugins/          (project-local, highest priority)
//  2. $XDG_DATA_HOME/loomchart/plugins/   (Linux/macOS)
//  3. %APPDATA%/loomchart/plugins/        (Windows)
//
// An optional `--plugins <path>` override is supported via `PluginDiscovery::with_extra_path`.

use std::path::{Path, PathBuf};

/// Collects all plugin search paths for the current platform.
pub fn default_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Always include project-local directory.
    paths.push(PathBuf::from("plugins"));

    // Platform-specific user data directory.
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            paths.push(PathBuf::from(appdata).join("loomchart").join("plugins"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs_home().join(".local").join("share")
            });
        paths.push(base.join("loomchart").join("plugins"));
    }

    paths
}

#[cfg(not(target_os = "windows"))]
fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Result of discovering a single plugin file.
pub struct DiscoveredPlugin {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

/// Scans `search_paths` for `*.wasm` files and returns their raw bytes.
///
/// Files that cannot be read are skipped with a warning to stderr.
pub fn discover_plugins(search_paths: &[PathBuf]) -> Vec<DiscoveredPlugin> {
    let mut found = Vec::new();

    for dir in search_paths {
        if !dir.is_dir() {
            continue;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!(
                    "[loomchart] warning: cannot read plugin directory {}: {err}",
                    dir.display()
                );
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("wasm") {
                continue;
            }
            match std::fs::read(&path) {
                Ok(bytes) => found.push(DiscoveredPlugin { path, bytes }),
                Err(err) => {
                    eprintln!(
                        "[loomchart] warning: cannot read plugin {}: {err}",
                        path.display()
                    );
                }
            }
        }
    }

    found
}

/// High-level helper: discover all plugins and load them into a `WasmPluginLoader`.
///
/// Returns a list of successfully loaded plugin IDs.
/// Plugins that fail validation are skipped (error printed to stderr).
#[cfg(feature = "plugin-runtime")]
pub fn autoload(
    loader: &mut super::loader::WasmPluginLoader,
    extra_path: Option<&Path>,
) -> Vec<String> {
    let mut paths = default_search_paths();
    if let Some(p) = extra_path {
        paths.insert(0, p.to_path_buf());
    }

    let discovered = discover_plugins(&paths);
    let mut loaded_ids = Vec::new();

    for plugin in discovered {
        match loader.load_from_bytes(&plugin.bytes) {
            Ok(id) => {
                println!(
                    "[loomchart] loaded plugin '{id}' from {}",
                    plugin.path.display()
                );
                loaded_ids.push(id);
            }
            Err(err) => {
                eprintln!(
                    "[loomchart] warning: failed to load plugin {}: {err}",
                    plugin.path.display()
                );
            }
        }
    }

    loaded_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_paths_nonempty() {
        let paths = default_search_paths();
        assert!(!paths.is_empty());
        // Project-local path is always first.
        assert_eq!(paths[0], PathBuf::from("plugins"));
    }

    #[test]
    fn test_discover_empty_dir() {
        // Scanning a non-existent directory returns nothing.
        let result = discover_plugins(&[PathBuf::from("/nonexistent/path")]);
        assert!(result.is_empty());
    }
}
