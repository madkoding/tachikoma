//! =============================================================================
//! Model Tier Value Object
//! =============================================================================
//! Represents the different tiers of AI models based on capability and resource
//! requirements. Used by the ModelManager to select appropriate models.
//! 
//! # Model Selection Strategy
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        MODEL SELECTION MATRIX                           │
//! ├─────────────────┬──────────────┬─────────────┬─────────────────────────┤
//! │    Tier         │   VRAM Req   │   Speed     │   Use Cases             │
//! ├─────────────────┼──────────────┼─────────────┼─────────────────────────┤
//! │  Light (3B)     │   < 4GB      │   Fast      │  Terminal, quick tasks  │
//! │  Standard (7B)  │   4-8GB      │   Medium    │  General chat, search   │
//! │  Heavy (14B+)   │   > 8GB      │   Slow      │  Complex coding, reason │
//! └─────────────────┴──────────────┴─────────────┴─────────────────────────┘
//! ```
//! =============================================================================

use serde::{Deserialize, Serialize};

/// =============================================================================
/// ModelTier - Classification of model capability levels
/// =============================================================================
/// Defines the tier of AI model to use based on task complexity and
/// available system resources (primarily VRAM).
/// 
/// # Tiers
/// 
/// * `Light` - 3B parameter models for fast, simple tasks
/// * `Standard` - 7B parameter models for general-purpose use
/// * `Heavy` - 14B+ parameter models for complex reasoning/coding
/// * `Embedding` - Specialized embedding models for vector generation
/// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    /// =======================================================================
    /// Light Tier - 3B Parameter Models
    /// =======================================================================
    /// Fast, efficient models for simple tasks that don't require deep
    /// reasoning. Ideal for terminal commands and quick queries.
    /// 
    /// # Characteristics
    /// - VRAM: ~2-4 GB
    /// - Speed: Very fast (< 1s for short responses)
    /// - Quality: Good for simple tasks
    /// 
    /// # Model Examples
    /// - ministral-3b
    /// - phi-3-mini
    /// - gemma-2b
    /// =======================================================================
    Light,

    /// =======================================================================
    /// Standard Tier - 7B Parameter Models
    /// =======================================================================
    /// Balanced models for general-purpose use. Good quality with
    /// reasonable performance.
    /// 
    /// # Characteristics
    /// - VRAM: ~4-8 GB
    /// - Speed: Medium (1-5s for typical responses)
    /// - Quality: Good for most tasks
    /// 
    /// # Model Examples
    /// - llama3.1:8b
    /// - mistral:7b
    /// - qwen2.5:7b
    /// =======================================================================
    Standard,

    /// =======================================================================
    /// Heavy Tier - 14B+ Parameter Models
    /// =======================================================================
    /// High-capability models for complex tasks requiring deep reasoning,
    /// code generation, or nuanced understanding.
    /// 
    /// # Characteristics
    /// - VRAM: > 8 GB (typically 12-16 GB)
    /// - Speed: Slower (5-15s for complex responses)
    /// - Quality: Excellent for complex tasks
    /// 
    /// # Model Examples
    /// - qwen2.5-coder:14b
    /// - llama3.1:70b (quantized)
    /// - deepseek-coder:33b
    /// =======================================================================
    Heavy,

    /// =======================================================================
    /// Embedding Tier - Specialized Embedding Models
    /// =======================================================================
    /// Dedicated models for generating vector embeddings.
    /// Not used for generation, only for semantic search.
    /// 
    /// # Characteristics
    /// - VRAM: ~1-2 GB
    /// - Speed: Very fast
    /// - Output: Fixed-dimension vectors
    /// 
    /// # Model Examples
    /// - nomic-embed-text
    /// - bge-large
    /// - mxbai-embed-large
    /// =======================================================================
    Embedding,
}

impl ModelTier {
    /// =========================================================================
    /// Get the default model name for this tier
    /// =========================================================================
    /// Returns the recommended model name for each tier.
    /// These are the models that NEURO-OS is optimized for.
    /// 
    /// # Returns
    /// 
    /// The default model name string
    /// =========================================================================
    pub fn default_model(&self) -> &'static str {
        match self {
            ModelTier::Light => "ministral-3b",
            ModelTier::Standard => "qwen2.5:7b",
            ModelTier::Heavy => "qwen2.5-coder:14b",
            ModelTier::Embedding => "nomic-embed-text",
        }
    }

    /// =========================================================================
    /// Get the minimum VRAM required for this tier (in MB)
    /// =========================================================================
    /// Returns the minimum VRAM required to run models in this tier.
    /// 
    /// # Returns
    /// 
    /// VRAM requirement in megabytes
    /// =========================================================================
    pub fn min_vram_mb(&self) -> u64 {
        match self {
            ModelTier::Light => 2048,      // 2 GB
            ModelTier::Standard => 4096,   // 4 GB
            ModelTier::Heavy => 8192,      // 8 GB
            ModelTier::Embedding => 1024,  // 1 GB
        }
    }

    /// =========================================================================
    /// Get the recommended VRAM for this tier (in MB)
    /// =========================================================================
    /// Returns the recommended VRAM for optimal performance.
    /// 
    /// # Returns
    /// 
    /// Recommended VRAM in megabytes
    /// =========================================================================
    pub fn recommended_vram_mb(&self) -> u64 {
        match self {
            ModelTier::Light => 4096,      // 4 GB
            ModelTier::Standard => 8192,   // 8 GB
            ModelTier::Heavy => 16384,     // 16 GB
            ModelTier::Embedding => 2048,  // 2 GB
        }
    }

    /// =========================================================================
    /// Get the expected max context length for this tier
    /// =========================================================================
    /// Returns the typical context length supported by models in this tier.
    /// 
    /// # Returns
    /// 
    /// Context length in tokens
    /// =========================================================================
    pub fn context_length(&self) -> usize {
        match self {
            ModelTier::Light => 4096,
            ModelTier::Standard => 8192,
            ModelTier::Heavy => 32768,
            ModelTier::Embedding => 8192,
        }
    }

    /// =========================================================================
    /// Select appropriate tier based on available VRAM
    /// =========================================================================
    /// Determines the best model tier given the available GPU memory.
    /// Falls back to lighter tiers if resources are insufficient.
    /// 
    /// # Arguments
    /// 
    /// * `available_vram_mb` - Available VRAM in megabytes
    /// 
    /// # Returns
    /// 
    /// The appropriate model tier for the available resources
    /// =========================================================================
    pub fn from_available_vram(available_vram_mb: u64) -> Self {
        if available_vram_mb >= ModelTier::Heavy.min_vram_mb() {
            ModelTier::Heavy
        } else if available_vram_mb >= ModelTier::Standard.min_vram_mb() {
            ModelTier::Standard
        } else {
            ModelTier::Light
        }
    }

    /// =========================================================================
    /// Select appropriate tier based on task complexity
    /// =========================================================================
    /// Determines the best model tier given the task requirements.
    /// 
    /// # Arguments
    /// 
    /// * `is_code_task` - Whether the task involves code generation
    /// * `requires_reasoning` - Whether deep reasoning is needed
    /// * `is_quick_response` - Whether speed is prioritized
    /// 
    /// # Returns
    /// 
    /// The recommended model tier for the task
    /// =========================================================================
    pub fn from_task_requirements(
        is_code_task: bool,
        requires_reasoning: bool,
        is_quick_response: bool,
    ) -> Self {
        if is_quick_response && !is_code_task && !requires_reasoning {
            ModelTier::Light
        } else if is_code_task || requires_reasoning {
            ModelTier::Heavy
        } else {
            ModelTier::Standard
        }
    }

    /// =========================================================================
    /// Get all generation tiers (excludes embedding)
    /// =========================================================================
    pub fn generation_tiers() -> Vec<Self> {
        vec![ModelTier::Light, ModelTier::Standard, ModelTier::Heavy]
    }

    /// =========================================================================
    /// Check if this tier supports code generation
    /// =========================================================================
    pub fn supports_code_generation(&self) -> bool {
        matches!(self, ModelTier::Standard | ModelTier::Heavy)
    }

    /// =========================================================================
    /// Get the expected tokens per second for this tier
    /// =========================================================================
    /// Returns the typical generation speed.
    /// 
    /// # Returns
    /// 
    /// Expected tokens per second
    /// =========================================================================
    pub fn expected_tokens_per_second(&self) -> f64 {
        match self {
            ModelTier::Light => 80.0,
            ModelTier::Standard => 40.0,
            ModelTier::Heavy => 20.0,
            ModelTier::Embedding => 1000.0, // Not really applicable
        }
    }
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Light => write!(f, "Light (3B)"),
            ModelTier::Standard => write!(f, "Standard (7B)"),
            ModelTier::Heavy => write!(f, "Heavy (14B+)"),
            ModelTier::Embedding => write!(f, "Embedding"),
        }
    }
}

impl Default for ModelTier {
    fn default() -> Self {
        ModelTier::Standard
    }
}

/// =============================================================================
/// ModelConfig - Configuration for a specific model
/// =============================================================================
/// Contains all configuration needed to use a model with Ollama.
/// =============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model tier
    pub tier: ModelTier,

    /// Model name/tag as used by Ollama
    pub model_name: String,

    /// Temperature for generation (0.0 - 2.0)
    pub temperature: f32,

    /// Top-p sampling parameter
    pub top_p: f32,

    /// Maximum tokens to generate
    pub max_tokens: usize,

    /// Context window size
    pub context_size: usize,

    /// System prompt template
    pub system_prompt: Option<String>,
}

impl ModelConfig {
    /// =========================================================================
    /// Create a new model configuration
    /// =========================================================================
    pub fn new(tier: ModelTier) -> Self {
        let model_name = tier.default_model().to_string();
        let context_size = tier.context_length();

        Self {
            tier,
            model_name,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2048,
            context_size,
            system_prompt: None,
        }
    }

    /// =========================================================================
    /// Create a configuration for code generation
    /// =========================================================================
    pub fn for_code() -> Self {
        Self {
            tier: ModelTier::Heavy,
            model_name: "qwen2.5-coder:14b".to_string(),
            temperature: 0.3, // Lower for more deterministic code
            top_p: 0.95,
            max_tokens: 4096,
            context_size: 32768,
            system_prompt: Some(
                "You are an expert programmer. Generate clean, efficient, \
                 well-documented code. Follow best practices and explain your reasoning."
                    .to_string(),
            ),
        }
    }

    /// =========================================================================
    /// Create a configuration for quick responses
    /// =========================================================================
    pub fn for_quick_response() -> Self {
        Self {
            tier: ModelTier::Light,
            model_name: "ministral-3b".to_string(),
            temperature: 0.5,
            top_p: 0.9,
            max_tokens: 256,
            context_size: 4096,
            system_prompt: Some(
                "Be concise and direct. Provide brief, helpful responses.".to_string(),
            ),
        }
    }

    /// =========================================================================
    /// Create a configuration for embeddings
    /// =========================================================================
    pub fn for_embeddings() -> Self {
        Self {
            tier: ModelTier::Embedding,
            model_name: "nomic-embed-text".to_string(),
            temperature: 0.0,
            top_p: 1.0,
            max_tokens: 0, // Not used for embeddings
            context_size: 8192,
            system_prompt: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_from_vram() {
        assert_eq!(ModelTier::from_available_vram(16000), ModelTier::Heavy);
        assert_eq!(ModelTier::from_available_vram(6000), ModelTier::Standard);
        assert_eq!(ModelTier::from_available_vram(2000), ModelTier::Light);
    }

    #[test]
    fn test_tier_from_task() {
        let tier = ModelTier::from_task_requirements(true, true, false);
        assert_eq!(tier, ModelTier::Heavy);

        let tier = ModelTier::from_task_requirements(false, false, true);
        assert_eq!(tier, ModelTier::Light);
    }

    #[test]
    fn test_default_models() {
        assert_eq!(ModelTier::Light.default_model(), "ministral-3b");
        assert_eq!(ModelTier::Heavy.default_model(), "qwen2.5-coder:14b");
    }
}
