//! =============================================================================
//! Model Manager Service
//! =============================================================================
//! Manages AI model selection based on available VRAM and task complexity.
//! =============================================================================

use std::sync::Arc;
use tracing::{info, warn};

use crate::domain::{
    errors::DomainError,
    ports::llm_provider::LlmProvider,
    value_objects::model_tier::{ModelConfig, ModelTier},
};

/// =============================================================================
/// ModelManager - AI Model Selection Service
/// =============================================================================
pub struct ModelManager {
    llm_provider: Arc<dyn LlmProvider>,
}

impl ModelManager {
    pub fn new(llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self { llm_provider }
    }

    /// Select model based on task requirements
    pub async fn select_model(
        &self,
        is_code_task: bool,
        requires_reasoning: bool,
        is_quick_response: bool,
    ) -> Result<ModelConfig, DomainError> {
        let tier = if is_code_task {
            ModelTier::Heavy
        } else if requires_reasoning {
            ModelTier::Standard
        } else if is_quick_response {
            ModelTier::Light
        } else {
            ModelTier::Standard
        };

        info!("Selected model tier: {:?}", tier);
        Ok(ModelConfig::new(tier))
    }

    /// Get embedding model configuration
    pub async fn get_embedding_config(&self) -> Result<ModelConfig, DomainError> {
        Ok(ModelConfig::for_embeddings())
    }

    /// Ensure required models are available
    pub async fn ensure_models_available(&self, tier: ModelTier) -> Result<(), DomainError> {
        let model_name = tier.default_model();
        
        // Check if model exists
        let models = self.llm_provider.list_models().await?;
        let exists = models.iter().any(|m| m.name == model_name);
        
        if !exists {
            info!("Pulling model: {}", model_name);
            self.llm_provider.pull_model(model_name).await?;
        }

        Ok(())
    }

    /// Get LLM provider reference
    pub fn llm_provider(&self) -> &Arc<dyn LlmProvider> {
        &self.llm_provider
    }
}
