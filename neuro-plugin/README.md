# TACHIKOMA-OS Plugin System

Sistema de plugins para agentes de TACHIKOMA-OS: herramientas, backends de
memoria y adaptadores de modelo.

## Qué es

`neuro-plugin` define el trait `TachikomaPlugin` y un `PluginRegistry` que
descubre paquetes `.tachikoma` en disco y los resuelve a herramientas de
agente en tiempo de ejecución, sin reiniciar (hot-reload).

## Tipos de plugin

| Tipo | Descripción |
|------|-------------|
| `tool` | Añade herramientas para los agentes |
| `memory_connector` | Backend de memoria alternativo |
| `model_adapter` | Adaptador de formato de modelo |

## Formato de paquete

Un paquete `.tachikoma` es un archivo JSON plano con el manifiesto:

```json
{
  "id": "hello-tool",
  "name": "Hello Tool",
  "version": "0.1.0",
  "type": "tool",
  "entry": "plugin.wasm",
  "capabilities": [],
  "description": "Añade una herramienta de saludo.",
  "tools": ["say_hello"]
}
```

- `entry`: punto de entrada WASM dentro del paquete (metadato por ahora).
- `capabilities`: permisos que solicita el plugin. Por defecto todo se deniega.
- `tools`: nombres de herramientas (solo para plugins `tool`).

## Uso

```rust
use neuro_plugin::PluginRegistry;
use std::path::PathBuf;

let mut reg = PluginRegistry::new(PathBuf::from("plugins"));
reg.scan()?; // carga/re-carga todos los .tachikoma

for p in reg.plugins() {
    println!("{} v{}", p.manifest.name, p.manifest.version);
}

// Buscar una herramienta por id namespaced: "hello-tool/say_hello"
if let Some(tool) = reg.find_tool("hello-tool/say_hello") {
    println!("{}", tool.name);
}
```

## Hot-reload

`PluginRegistry::scan()` es el punto de entrada de recarga: los paquetes
nuevos/cambiados se cargan y los que desaparecieron se eliminan. Llámalo al
arrancar y bajo demanda.

## Sandboxing (WASM)

`ponytail:` el sandbox real (Extism/Wasmtime) aísla el código del sistema de
archivos/red del host. Es una dependencia grande; este crate define la misma
superficie de trait + registry y conecta un sandbox detrás de la feature flag
`wasm`, de modo que la API es estable y el backend WASM puede intercambiarse
sin cambiar a los consumidores.

## Desarrollo

```bash
cargo test -p neuro-plugin
cargo clippy -p neuro-plugin
```
