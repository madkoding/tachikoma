//! =============================================================================
//! Checklist Repository Port
//! =============================================================================

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::checklist::{
    Checklist, ChecklistItem, CreateChecklist, CreateChecklistItem,
    UpdateChecklist, UpdateChecklistItem,
};
use crate::domain::errors::DomainError;

/// Abstract repository interface for checklists
#[async_trait]
pub trait ChecklistRepository: Send + Sync {
    // =========================================================================
    // Checklist CRUD
    // =========================================================================
    
    /// Get all checklists with pagination
    async fn get_all_checklists(
        &self,
        limit: usize,
        offset: usize,
        include_archived: bool,
    ) -> Result<Vec<Checklist>, DomainError>;

    /// Count total checklists
    async fn count_checklists(&self, include_archived: bool) -> Result<usize, DomainError>;

    /// Get a single checklist by ID
    async fn get_checklist(&self, id: Uuid) -> Result<Option<Checklist>, DomainError>;

    /// Create a new checklist
    async fn create_checklist(&self, data: CreateChecklist) -> Result<Checklist, DomainError>;

    /// Update an existing checklist
    async fn update_checklist(
        &self,
        id: Uuid,
        data: UpdateChecklist,
    ) -> Result<Option<Checklist>, DomainError>;

    /// Delete a checklist
    async fn delete_checklist(&self, id: Uuid) -> Result<bool, DomainError>;

    // =========================================================================
    // Checklist Items CRUD
    // =========================================================================

    /// Get all items for a checklist
    async fn get_items(&self, checklist_id: Uuid) -> Result<Vec<ChecklistItem>, DomainError>;

    /// Add an item to a checklist
    async fn add_item(
        &self,
        checklist_id: Uuid,
        data: CreateChecklistItem,
    ) -> Result<ChecklistItem, DomainError>;

    /// Update an item
    async fn update_item(
        &self,
        item_id: Uuid,
        data: UpdateChecklistItem,
    ) -> Result<Option<ChecklistItem>, DomainError>;

    /// Toggle item completion
    async fn toggle_item(&self, item_id: Uuid) -> Result<Option<ChecklistItem>, DomainError>;

    /// Delete an item
    async fn delete_item(&self, item_id: Uuid) -> Result<bool, DomainError>;
}
