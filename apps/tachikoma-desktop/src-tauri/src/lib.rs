//! TACHIKOMA-OS Desktop - process orchestration.
//!
//! Spawns and supervises the Rust sidecar processes that make up the
//! backend (SurrealDB + API gateway). Sidecars are bundled as `externalBin`
//! and auto-restarted with backoff if they crash.

mod hardware;

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// A single sidecar process definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SidecarSpec {
    pub name: String,
    /// Binary file name (without target triple suffix); resolved via `tauri` sidecar lookup.
    pub bin: String,
    pub args: Vec<String>,
    /// Port used for the /health check.
    pub health_port: u16,
    pub restart_on_exit: bool,
}

/// Snapshot of a running sidecar, exposed to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct SidecarStatus {
    pub name: String,
    pub pid: Option<u32>,
    pub state: String,
    pub restarts: u32,
    pub port: u16,
    pub health_url: String,
}

/// Managed state for all sidecar processes.
pub struct SidecarManager {
    specs: Vec<SidecarSpec>,
    statuses: std::sync::Mutex<Vec<SidecarStatus>>,
    app_data_dir: PathBuf,
}

impl SidecarManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let mut specs = default_specs(&app_data_dir);
        let statuses = specs
            .iter()
            .map(|s| SidecarStatus {
                name: s.name.clone(),
                pid: None,
                state: "stopped".into(),
                restarts: 0,
                port: s.health_port,
                health_url: format!("http://localhost:{}/health", s.health_port),
            })
            .collect();
        // SurrealDB owns the data dir; spawn order matters.
        specs.sort_by_key(|s| if s.name == "surrealdb" { 0 } else { 1 });
        Self {
            specs,
            statuses: std::sync::Mutex::new(statuses),
            app_data_dir,
        }
    }

    pub fn statuses(&self) -> Vec<SidecarStatus> {
        self.statuses.lock().unwrap().clone()
    }

    /// Spawn every sidecar and supervise them in the background.
    pub fn start_all(&self, app_handle: tauri::AppHandle) {
        for spec in &self.specs {
            let spec = spec.clone();
            let data_dir = self.app_data_dir.clone();
            let handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                Self::supervise(handle, spec, data_dir).await;
            });
        }
    }

    async fn supervise(handle: tauri::AppHandle, spec: SidecarSpec, data_dir: PathBuf) {
        let mut restarts = 0u32;
        loop {
            let child = match spawn_sidecar(&spec, &data_dir) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("[{}/{}] failed to spawn: {}", spec.name, spec.bin, e);
                    set_state(&handle, &spec.name, "spawn-error", None, restarts);
                    if spec.restart_on_exit {
                        sleep(Duration::from_secs(5)).await;
                        restarts += 1;
                        continue;
                    }
                    return;
                }
            };

            let pid = child.id();
            set_state(&handle, &spec.name, "running", pid, restarts);
            // Wait for exit. The child is moved here; health check is skipped for
            // simplicity — SurrealDB readiness is polled by the backend on connect.
            let _ = wait_for_exit(child).await;
            set_state(&handle, &spec.name, "stopped", None, restarts);

            if spec.restart_on_exit {
                tracing::warn!("[{}/{}] exited; restarting ({})", spec.name, spec.bin, restarts + 1);
                sleep(Duration::from_secs(backoff(restarts))).await;
                restarts += 1;
            } else {
                return;
            }
        }
    }
}

fn backoff(restarts: u32) -> u64 {
    // ponytail: linear backoff capped at 30s. Exponential if crash-looping matters.
    (2 * restarts).clamp(2, 30) as u64
}

fn spawn_sidecar(
    spec: &SidecarSpec,
    data_dir: &Path,
) -> std::io::Result<Child> {
    // Resolve the bundled sidecar binary: <bin>-<target-triple> next to the
    // current executable (Tauri `externalBin` convention).
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no exe dir"))?
        .to_path_buf();
    let triple = tauri_utils::platform::target_triple()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    #[cfg(windows)]
    let sidecar_path = exe_dir.join(format!("{}-{}.exe", spec.bin, triple));
    #[cfg(not(windows))]
    let sidecar_path = exe_dir.join(format!("{}-{}", spec.bin, triple));

    let args = spec.args.iter().map(|a| resolve_args(a, data_dir)).collect::<Vec<_>>();
    Command::new(&sidecar_path)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

/// Resolve `{DATA_DIR}` placeholders in argument strings.
fn resolve_args(arg: &str, data_dir: &Path) -> String {
    arg.replace("{DATA_DIR}", &data_dir.to_string_lossy())
}

async fn wait_for_exit(mut child: Child) -> i32 {
    child.wait().await.map(|s| s.code().unwrap_or(1)).unwrap_or(1)
}

fn set_state(handle: &tauri::AppHandle, name: &str, state: &str, pid: Option<u32>, restarts: u32) {
    let _ = handle.emit("sidecar://status", &SidecarStatus {
        name: name.into(),
        pid,
        state: state.into(),
        restarts,
        port: 0,
        health_url: String::new(),
    });
}

fn default_specs(data_dir: &Path) -> Vec<SidecarSpec> {
    let db_path = data_dir.join("surrealdb.db");
    // Microservices read env vars with sane defaults (see neuro_common::Config),
    // so no args are needed. Voice is excluded: it needs the external Piper
    // binary + voice models, and the backend already embeds a VoiceEngine.
    vec![
        SidecarSpec {
            name: "surrealdb".into(),
            bin: "surrealdb".into(),
            args: vec![
                "start".into(),
                "--user".into(),
                std::env::var("SURREAL_USER").unwrap_or_else(|_| "root".into()),
                "--pass".into(),
                std::env::var("SURREAL_PASS").unwrap_or_else(|_| "root".into()),
                "--log".into(),
                "warn".into(),
                format!("file:{}", db_path.display()),
            ],
            health_port: 8000,
            restart_on_exit: true,
        },
        SidecarSpec {
            name: "backend".into(),
            bin: "tachikoma-backend".into(),
            args: vec![],
            health_port: 3000,
            restart_on_exit: true,
        },
        SidecarSpec {
            name: "checklists".into(),
            bin: "tachikoma-checklists".into(),
            args: vec![],
            health_port: 3001,
            restart_on_exit: true,
        },
        SidecarSpec {
            name: "music".into(),
            bin: "tachikoma-music".into(),
            args: vec![],
            health_port: 3002,
            restart_on_exit: true,
        },
        SidecarSpec {
            name: "chat".into(),
            bin: "tachikoma-chat".into(),
            args: vec![],
            health_port: 3003,
            restart_on_exit: true,
        },
        SidecarSpec {
            name: "memory".into(),
            bin: "tachikoma-memory".into(),
            args: vec![],
            health_port: 3004,
            restart_on_exit: true,
        },
        SidecarSpec {
            name: "agent".into(),
            bin: "tachikoma-agent".into(),
            args: vec![],
            health_port: 3005,
            restart_on_exit: true,
        },
    ]
}

/// Commands exposed to the frontend via the `sidecars` invoke handler.
#[tauri::command]
fn sidecar_status(manager: State<'_, SidecarManager>) -> Vec<SidecarStatus> {
    manager.statuses()
}

/// Fase 1.2 — hardware detection for the onboarding wizard.
#[tauri::command]
fn hardware_detect() -> hardware::HardwareProfile {
    hardware::detect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_bounds() {
        assert_eq!(backoff(0), 2);
        assert_eq!(backoff(1), 2);
        assert_eq!(backoff(5), 10);
        assert_eq!(backoff(50), 30);
    }

    #[test]
    fn resolve_args_substitutes_data_dir() {
        let dir = PathBuf::from("/tmp/tachikoma");
        assert_eq!(resolve_args("file:{DATA_DIR}/db", &dir), "file:/tmp/tachikoma/db");
        assert_eq!(resolve_args("no-placeholder", &dir), "no-placeholder");
    }

    #[test]
    fn default_specs_orders_surrealdb_first() {
        let specs = default_specs(Path::new("/tmp"));
        assert_eq!(specs.first().unwrap().name, "surrealdb");
        assert_eq!(specs[1].name, "backend");
        assert!(specs.iter().all(|s| s.restart_on_exit));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let manager = SidecarManager::new(data_dir.clone());
            manager.start_all(app.handle().clone());
            app.manage(manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![sidecar_status, hardware_detect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
