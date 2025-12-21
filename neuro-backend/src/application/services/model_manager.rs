//! =============================================================================
//! Model Manager Service
//! =============================================================================
//! Manages AI model selection based on available VRAM and task complexity.
//! Implements the "Quantization Manager" requirement for auto-selecting
//! between 3B, 7B, and 14B models.
//! 
//! # Model Selection Strategy
//! 
//! The ModelManager implements a two-factor selection algorithm:
//! 
//! 1. **Hardware Constraint**: Check available VRAM to determine max tier
//! 2. **Task Requirement**: Determine minimum tier needed for the task
//! 3. **Selection**: Choose the intersection (highest available that meets task)
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                    MODEL SELECTION ALGORITHM                            │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  Task Analysis                    Hardware Check                        │
//! │       │                                 │                               │
//! │       ▼                                 ▼                               │
//! │  ┌──────────┐                    ┌──────────────┐                       │
//! │  │ Required │                    │  Available   │                       │
//! │  │   Tier   │                    │    VRAM      │                       │
//! │  └────┬─────┘                    └──────┬───────┘                       │
//! │       │                                 │                               │
//! │       └──────────────┬──────────────────┘                               │
//! │                      ▼                                                  │
//! │              ┌──────────────┐                                           │
//! │              │   Select     │                                           │
//! │              │  Best Match  │                                           │
//! │              └──────────────┘                                           │
//! │                      │                                                  │
//! │           ┌──────────┼──────────┐                                       │
//! │           ▼          ▼          ▼                                       │
//! │      ministral   qwen2.5    qwen2.5-coder                               │
//! │         3B         7B          14B                                      │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//! =============================================================================

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::domain::{
    errors::DomainError,
    ports::llm_provider::LlmProvider,
    value_objects::model_tier::{ModelConfig, ModelTier},
};

/// =============================================================================
/// GpuInfo - GPU hardware information
/// =============================================================================
/// Contains information about available GPU resources for model selection.
/// =============================================================================
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// GPU name/model
    pub name: String,
    /// Total VRAM in MB
    pub total_vram_mb: u64,
    /// Available/free VRAM in MB
    pub available_vram_mb: u64,
    /// GPU utilization percentage
    pub utilization_percent: f32,
    /// CUDA compute capability (if NVIDIA)
    pub compute_capability: Option<String>,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            total_vram_mb: 0,
            available_vram_mb: 0,
            utilization_percent: 0.0,
            compute_capability: None,
        }
    }
}

/// =============================================================================
/// SystemResources - System resource information
/// =============================================================================
/// Contains information about available system resources.
/// =============================================================================
#[derive(Debug, Clone)]
pub struct SystemResources {
    /// GPU information (if available)
    pub gpu: Option<GpuInfo>,
    /// Total system RAM in MB
    pub total_ram_mb: u64,
    /// Available system RAM in MB
    pub available_ram_mb: u64,
    /// Number of CPU cores
    pub cpu_cores: usize,
}

/// =============================================================================
/// ModelManager - AI Model Selection Service
/// =============================================================================
/// Manages model selection based on hardware resources and task requirements.
/// Caches hardware info and maintains model availability state.
/// 
/// # Thread Safety
/// 
/// The ModelManager is thread-safe and can be shared across async tasks.
/// Hardware information is cached and refreshed periodically.
/// 
/// # Example Usage
/// 
/// ```rust
/// let manager = ModelManager::new(llm_provider);
/// 
/// // Get best model for a task
/// let config = manager.select_model_for_task(&task).await?;
/// 
/// // Force a specific tier
/// let config = manager.get_model_config(ModelTier::Heavy).await?;
/// ```
/// =============================================================================
pub struct ModelManager {
    /// The LLM provider for model operations
    llm_provider: Arc<dyn LlmProvider>,
    
    /// Cached system resources (refreshed periodically)
    cached_resources: RwLock<Option<SystemResources>>,
    
    /// Cache timestamp for resource refresh
    cache_timestamp: RwLock<std::time::Instant>,
    
    /// Cache duration in seconds
    cache_duration_secs: u64,
    
    /// Fallback tier when detection fails
    fallback_tier: ModelTier,
}

impl ModelManager {
    /// =========================================================================
    /// Create a new ModelManager
    /// =========================================================================
    /// Initializes the model manager with an LLM provider.
    /// Hardware detection is deferred until first use.
    /// 
    /// # Arguments
    /// 
    /// * `llm_provider` - The LLM provider for model operations
    /// 
    /// # Returns
    /// 
    /// A new ModelManager instance
    /// =========================================================================
    pub fn new(llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm_provider,
            cached_resources: RwLock::new(None),
            cache_timestamp: RwLock::new(std::time::Instant::now()),
            cache_duration_secs: 60, // Refresh every minute
            fallback_tier: ModelTier::Standard,
        }
    }

    /// =========================================================================
    /// Detect system resources
    /// =========================================================================
    /// Detects available GPU and system resources using sysinfo and nvml.
    /// Results are cached for performance.
    /// 
    /// # Returns
    /// 
    /// * `Ok(SystemResources)` - Detected resources
    /// * `Err(DomainError)` - If detection fails
    /// =========================================================================
    pub async fn detect_resources(&self) -> Result<SystemResources, DomainError> {
        // Check cache validity
        {
            let timestamp = self.cache_timestamp.read().await;
            let cache = self.cached_resources.read().await;
            
            if cache.is_some() 
                && timestamp.elapsed().as_secs() < self.cache_duration_secs 
            {
                return Ok(cache.clone().unwrap());
            }
        }

        // Detect resources
        let resources = self.detect_resources_impl().await?;

        // Update cache
        {
            let mut cache = self.cached_resources.write().await;
            let mut timestamp = self.cache_timestamp.write().await;
            *cache = Some(resources.clone());
            *timestamp = std::time::Instant::now();
        }

        Ok(resources)
    }

    /// =========================================================================
    /// Internal resource detection implementation
    /// =========================================================================
    async fn detect_resources_impl(&self) -> Result<SystemResources, DomainError> {
        use sysinfo::System;

        // Detect system RAM
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_ram_mb = sys.total_memory() / 1024 / 1024;
        let available_ram_mb = sys.available_memory() / 1024 / 1024;
        let cpu_cores = sys.cpus().len();

        // Try to detect GPU using NVML
        let gpu = self.detect_nvidia_gpu().await;

        let resources = SystemResources {
            gpu,
            total_ram_mb,
            available_ram_mb,
            cpu_cores,
        };

        info!(
            "System resources detected: RAM={}/{}MB, CPUs={}, GPU={:?}",
            available_ram_mb, total_ram_mb, cpu_cores, 
            resources.gpu.as_ref().map(|g| &g.name)
        );

        Ok(resources)
    }

    /// =========================================================================
    /// Detect NVIDIA GPU using NVML
    /// =========================================================================
    async fn detect_nvidia_gpu(&self) -> Option<GpuInfo> {
        // Try using nvml-wrapper for NVIDIA GPU detection
        #[cfg(feature = "nvml")]
        {
            use nvml_wrapper::Nvml;

            match Nvml::init() {
                Ok(nvml) => {
                    if let Ok(device) = nvml.device_by_index(0) {
                        let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());
                        let memory = device.memory_info().ok()?;
                        let utilization = device.utilization_rates().ok();

                        return Some(GpuInfo {
                            name,
                            total_vram_mb: memory.total / 1024 / 1024,
                            available_vram_mb: memory.free / 1024 / 1024,
                            utilization_percent: utilization
                                .map(|u| u.gpu as f32)
                                .unwrap_or(0.0),
                            compute_capability: device
                                .cuda_compute_capability()
                                .ok()
                                .map(|c| format!("{}.{}", c.major, c.minor)),
                        });
                    }
                }
                Err(e) => {
                    debug!("NVML init failed: {}", e);
                }
            }
        }

        // Fallback: Try to get GPU info from Ollama health check
        if let Ok(health) = self.llm_provider.health_check().await {
            if let (Some(total), Some(available)) = 
                (health.gpu_memory_total_mb, health.gpu_memory_available_mb) 
            {
                return Some(GpuInfo {
                    name: "GPU (via Ollama)".to_string(),
                    total_vram_mb: total,
                    available_vram_mb: available,
                    utilization_percent: ((total - available) as f32 / total as f32) * 100.0,
                    compute_capability: None,
                });
            }
        }

        // No GPU detected or CPU-only mode
        warn!("No GPU detected, falling back to CPU inference");
        None
    }

    /// =========================================================================
    /// Get the maximum available model tier
    /// =========================================================================
    /// Determines the highest model tier that can run on available hardware.
    /// 
    /// # Returns
    /// 
    /// The maximum ModelTier supported by the hardware
    /// =========================================================================
    pub async fn get_max_available_tier(&self) -> ModelTier {
        match self.detect_resources().await {
            Ok(resources) => {
                if let Some(gpu) = resources.gpu {
                    ModelTier::from_available_vram(gpu.available_vram_mb)
                } else {
                    // CPU-only: use light models
                    ModelTier::Light
                }
            }
            Err(_) => self.fallback_tier.clone(),
        }
    }

    /// =========================================================================
    /// Select model for a task
    /// =========================================================================
    /// Selects the optimal model based on task requirements and hardware.
    /// 
    /// # Arguments
    /// 
    /// * `is_code_task` - Whether the task involves code generation
    /// * `requires_reasoning` - Whether deep reasoning is needed
    /// * `is_quick_response` - Whether speed is prioritized
    /// 
    /// # Returns
    /// 
    /// * `Ok(ModelConfig)` - Configuration for the selected model
    /// * `Err(DomainError)` - If no suitable model is available
    /// =========================================================================
    pub async fn select_model(
        &self,
        is_code_task: bool,
        requires_reasoning: bool,
        is_quick_response: bool,
    ) -> Result<ModelConfig, DomainError> {
        // Determine task requirement
        let required_tier = ModelTier::from_task_requirements(
            is_code_task,
            requires_reasoning,
            is_quick_response,
        );

        // Determine hardware capability
        let max_tier = self.get_max_available_tier().await;

        // Select the appropriate tier
        let selected_tier = self.reconcile_tiers(required_tier, max_tier);

        info!(
            "Model selection: required={}, max={}, selected={}",
            required_tier, max_tier, selected_tier
        );

        // Get model configuration
        self.get_model_config(selected_tier).await
    }

    /// =========================================================================
    /// Reconcile required and available tiers
    /// =========================================================================
    /// Determines the best tier given requirements and constraints.
    /// 
    /// # Logic
    /// 
    /// - If required > max: downgrade to max (with warning)
    /// - If required <= max: use required (don't over-provision)
    /// =========================================================================
    fn reconcile_tiers(&self, required: ModelTier, max_available: ModelTier) -> ModelTier {
        let tier_priority = |tier: &ModelTier| -> u8 {
            match tier {
                ModelTier::Light => 1,
                ModelTier::Standard => 2,
                ModelTier::Heavy => 3,
                ModelTier::Embedding => 0,
            }
        };

        if tier_priority(&required) > tier_priority(&max_available) {
            warn!(
                "Task requires {} but only {} available, downgrading",
                required, max_available
            );
            max_available
        } else {
            required
        }
    }

    /// =========================================================================
    /// Get model configuration for a tier
    /// =========================================================================
    /// Returns the configuration for the best available model in a tier.
    /// 
    /// # Arguments
    /// 
    /// * `tier` - The model tier to get configuration for
    /// 
    /// # Returns
    /// 
    /// * `Ok(ModelConfig)` - Configuration for the model
    /// * `Err(DomainError)` - If no model is available for the tier
    /// =========================================================================
    pub async fn get_model_config(&self, tier: ModelTier) -> Result<ModelConfig, DomainError> {
        // Check if the preferred model is available
        let model_name = self.llm_provider.get_best_model_for_tier(tier.clone()).await?;

        let mut config = ModelConfig::new(tier);
        config.model_name = model_name;

        Ok(config)
    }

    /// =========================================================================
    /// Get embedding model configuration
    /// =========================================================================
    /// Returns the configuration for the embedding model.
    /// 
    /// # Returns
    /// 
    /// * `Ok(ModelConfig)` - Configuration for embeddings
    /// * `Err(DomainError)` - If embedding model is not available
    /// =========================================================================
    pub async fn get_embedding_config(&self) -> Result<ModelConfig, DomainError> {
        let config = ModelConfig::for_embeddings();

        // Verify the embedding model is available
        if !self.llm_provider.is_model_available(&config.model_name).await {
            warn!(
                "Embedding model {} not available, attempting to pull",
                config.model_name
            );
            self.llm_provider.pull_model(&config.model_name).await?;
        }

        Ok(config)
    }

    /// =========================================================================
    /// Ensure required models are available
    /// =========================================================================
    /// Checks and pulls required models for a given tier.
    /// 
    /// # Arguments
    /// 
    /// * `tier` - The model tier to ensure
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Models are available
    /// * `Err(DomainError)` - If model pull fails
    /// =========================================================================
    pub async fn ensure_models_available(&self, tier: ModelTier) -> Result<(), DomainError> {
        let model_name = tier.default_model();

        if !self.llm_provider.is_model_available(model_name).await {
            info!("Pulling model: {}", model_name);
            self.llm_provider.pull_model(model_name).await?;
        }

        Ok(())
    }

    /// =========================================================================
    /// Get resource status
    /// =========================================================================
    /// Returns current system resource status for monitoring.
    /// 
    /// # Returns
    /// 
    /// Current system resources or None if detection fails
    /// =========================================================================
    pub async fn get_resource_status(&self) -> Option<SystemResources> {
        self.detect_resources().await.ok()
    }

    /// =========================================================================
    /// Invalidate resource cache
    /// =========================================================================
    /// Forces a refresh of the resource cache on next access.
    /// =========================================================================
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cached_resources.write().await;
        *cache = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_reconciliation() {
        // This would need a mock LlmProvider for full testing
        // Here we just test the tier priority logic
        assert!(matches!(
            ModelTier::from_task_requirements(true, true, false),
            ModelTier::Heavy
        ));

        assert!(matches!(
            ModelTier::from_task_requirements(false, false, true),
            ModelTier::Light
        ));
    }
}
