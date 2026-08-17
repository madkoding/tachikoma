//! Plugin package manifest (`.tachikoma` package: zip with `manifest.json` + WASM).

use serde::{Deserialize, Serialize};

/// What kind of plugin this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Adds new tools for agents.
    Tool,
    /// Alternative memory backend.
    MemoryConnector,
    /// Model format adapter.
    ModelAdapter,
}

/// Sandbox capability requested by the plugin. The host grants access only
/// for declared capabilities. Default is `deny` for everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Read filesystem (whitelisted paths only).
    FilesystemRead,
    /// Outbound network to declared domains.
    Network,
    /// Execute host commands (restricted).
    Command,
}

/// Parsed `manifest.json` inside a `.tachikoma` package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    /// WASM entrypoint file inside the package (e.g. `plugin.wasm`).
    pub entry: String,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default)]
    pub description: String,
    /// Tool names this plugin provides (only meaningful for `Tool` plugins).
    #[serde(default)]
    pub tools: Vec<String>,
}

impl PluginManifest {
    /// Basic sanity checks before loading.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.id.is_empty(), "plugin id is required");
        anyhow::ensure!(!self.entry.is_empty(), "plugin entry is required");
        anyhow::ensure!(!self.version.is_empty(), "plugin version is required");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_tool_manifest() {
        let json = r#"{
            "id": "hello-tool",
            "name": "Hello Tool",
            "version": "0.1.0",
            "type": "tool",
            "entry": "plugin.wasm",
            "tools": ["say_hello"]
        }"#;
        let m: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.plugin_type, PluginType::Tool);
        assert!(m.capabilities.is_empty());
        assert_eq!(m.tools, vec!["say_hello"]);
        m.validate().unwrap();
    }

    #[test]
    fn reject_empty_id() {
        let json = r#"{"id":"","name":"x","version":"1.0","type":"tool","entry":"p.wasm"}"#;
        let m: PluginManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn capabilities_roundtrip() {
        let m = PluginManifest {
            id: "p".into(),
            name: "p".into(),
            version: "1".into(),
            plugin_type: PluginType::MemoryConnector,
            entry: "p.wasm".into(),
            capabilities: vec![Capability::FilesystemRead, Capability::Network],
            description: String::new(),
            tools: vec![],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.capabilities.len(), 2);
    }
}
