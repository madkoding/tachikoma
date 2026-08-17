//! Hardware detection for onboarding (Fase 1.2).
//!
//! Detects RAM, free disk, and GPU (NVIDIA via `nvidia-smi`, else none).
//! `ponytail:` naive detection — reads `nvidia-smi` output; AMD/Apple Metal
//! detection is deferred until a GPU is actually reported by the backend.

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct HardwareProfile {
    pub cpu_cores: usize,
    pub ram_gb: f64,
    pub disk_free_gb: f64,
    pub gpu_vendor: String,
    pub gpu_model: String,
    pub vram_gb: f64,
}

/// Best-effort hardware snapshot. Never blocks on GPU tooling for long.
pub fn detect() -> HardwareProfile {
    use sysinfo::{Disks, System};
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_cores = sys.cpus().len();
    let ram_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    let disks = Disks::new_with_refreshed_list();
    let disk_free_gb = disks
        .iter()
        .map(|d| d.available_space() as f64 / 1024.0 / 1024.0 / 1024.0)
        .fold(0.0, f64::max);

    let (gpu_vendor, gpu_model, vram_gb) = detect_gpu();

    HardwareProfile {
        cpu_cores,
        ram_gb,
        disk_free_gb,
        gpu_vendor,
        gpu_model,
        vram_gb,
    }
}

fn detect_gpu() -> (String, String, f64) {
    // NVIDIA via nvidia-smi (fast, present on all NVIDIA systems).
    let out = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            let line = text.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let vram = parts[1].parse::<f64>().unwrap_or(0.0) / 1024.0; // MiB -> GiB
                return ("nvidia".into(), parts[0].to_string(), vram);
            }
        }
    }
    (String::new(), String::new(), 0.0)
}
