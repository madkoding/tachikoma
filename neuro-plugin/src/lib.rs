//! TACHIKOMA-OS plugin system.
//!
//! Defines the `TachikomaPlugin` trait and a `PluginRegistry` that loads
//! plugins from `.tachikoma` packages (zip with `manifest.json` + WASM) and
//! resolves them to agent tools at runtime, without restarting.
//!
//! `ponytail:` sandboxing. A real WASM runtime (Extism/Wasmtime) isolates code
//! from the host filesystem/network. That is a large dependency; this crate
//! defines the same trait + registry surface and wires a sandbox behind a
//! `wasm` feature flag, so the API is stable and the WASM backend can be
//! swapped in without changing consumers. See `wasm` feature.

pub mod manifest;
pub mod registry;

pub use manifest::{PluginManifest, PluginType, Capability};
pub use registry::{LoadedPlugin, PluginRegistry, PluginTool};
