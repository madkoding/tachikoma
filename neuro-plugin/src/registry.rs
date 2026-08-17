//! Plugin registry: discovers `.tachikoma` packages on disk and resolves them
//! to agent tools, hot-loadable at runtime.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::manifest::{PluginManifest, PluginType};

/// A tool exposed by a plugin, resolved for agent use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTool {
    /// Namespaced id, e.g. `hello-tool/say_hello`.
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// A plugin loaded into memory.
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub tools: Vec<PluginTool>,
}

/// In-memory registry of discovered plugins.
pub struct PluginRegistry {
    dir: PathBuf,
    plugins: HashMap<String, LoadedPlugin>,
}

impl PluginRegistry {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir, plugins: HashMap::new() }
    }

    pub fn plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }

    pub fn get(&self, plugin_id: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(plugin_id)
    }

    pub fn find_tool(&self, tool_id: &str) -> Option<&PluginTool> {
        self.plugins
            .values()
            .flat_map(|p| p.tools.iter())
            .find(|t| t.id == tool_id)
    }

    /// Scan the plugins directory and (re)load all `.tachikoma` packages.
    /// Missing packages are dropped; new/changed ones are loaded. This is the
    /// hot-reload entrypoint — call it on startup and on demand.
    pub fn scan(&mut self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.dir)?;

        let mut seen = HashSet::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "tachikoma") {
                continue;
            }
            seen.insert(path.clone());
            match load_package(&path) {
                Ok(loaded) => {
                    let id = loaded.manifest.id.clone();
                    debug!(plugin = %id, path = %path.display(), "loaded plugin");
                    self.plugins.insert(id, loaded);
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "failed to load plugin");
                }
            }
        }

        // Drop plugins whose package files disappeared.
        self.plugins.retain(|_, p| seen.contains(&p.path));

        Ok(())
    }
}

/// Load a single `.tachikoma` package file.
///
/// `ponytail:` package format. A real package is a zip of `manifest.json` +
/// `plugin.wasm`. To avoid a zip dependency now, we accept a flat JSON file
/// (`.tachikoma` extension) holding the manifest, and treat the WASM entry as
/// metadata. Swap for `zip` + `wasmtime` behind a `wasm` feature.
fn load_package(path: &std::path::Path) -> anyhow::Result<LoadedPlugin> {
    let data = fs::read(path)
        .with_context(|| format!("read plugin package {}", path.display()))?;
    let manifest: PluginManifest = serde_json::from_slice(&data)
        .with_context(|| format!("parse manifest in {}", path.display()))?;
    manifest.validate()?;

    let tools = resolve_tools(&manifest);
    Ok(LoadedPlugin {
        manifest,
        path: path.to_path_buf(),
        tools,
    })
}

fn resolve_tools(manifest: &PluginManifest) -> Vec<PluginTool> {
    if manifest.plugin_type != PluginType::Tool {
        return Vec::new();
    }
    manifest
        .tools
        .iter()
        .map(|name| PluginTool {
            id: format!("{}/{}", manifest.id, name),
            plugin_id: manifest.id.clone(),
            name: name.clone(),
            description: manifest.description.clone(),
            input_schema: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_pkg(dir: &std::path::Path, id: &str, tools: &[&str]) -> std::path::PathBuf {
        let manifest = PluginManifest {
            id: id.into(),
            name: id.into(),
            version: "0.1.0".into(),
            plugin_type: PluginType::Tool,
            entry: "plugin.wasm".into(),
            capabilities: vec![],
            description: "test".into(),
            tools: tools.iter().map(|s| s.to_string()).collect(),
        };
        let path = dir.join(format!("{}.tachikoma", id));
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        path
    }

    #[test]
    fn loads_tool_plugins_from_dir() {
        let dir = std::env::temp_dir().join(format!("plugins-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        write_pkg(&dir, "hello", &["say_hello", "goodbye"]);

        let mut reg = PluginRegistry::new(dir.clone());
        reg.scan().unwrap();
        assert_eq!(reg.plugins().len(), 1);

        let hello = reg.get("hello").unwrap();
        assert_eq!(hello.tools.len(), 2);
        assert_eq!(hello.tools[0].id, "hello/say_hello");

        let tool = reg.find_tool("hello/say_hello").unwrap();
        assert_eq!(tool.name, "say_hello");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_non_tachikoma_files() {
        let dir = std::env::temp_dir().join(format!("plugins-non-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("readme.txt"), "hello").unwrap();

        let mut reg = PluginRegistry::new(dir.clone());
        reg.scan().unwrap();
        assert_eq!(reg.plugins().len(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn drops_removed_packages_on_rescan() {
        let dir = std::env::temp_dir().join(format!("plugins-drop-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let p = write_pkg(&dir, "temp", &["x"]);

        let mut reg = PluginRegistry::new(dir.clone());
        reg.scan().unwrap();
        assert_eq!(reg.plugins().len(), 1);

        fs::remove_file(&p).unwrap();
        reg.scan().unwrap();
        assert_eq!(reg.plugins().len(), 0);
        let _ = fs::remove_dir_all(&dir);
    }
}
