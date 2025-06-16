use mm_memory::{MemoryRepository, MemoryService};
use std::sync::Arc;

/// Ports struct containing all required services for operations
///
/// This struct serves as a dependency injection container for all operations.
/// Each operation function will receive this struct to access the services it needs.
pub struct Ports<R>
where
    R: MemoryRepository + Send + Sync + 'static,
{
    /// Memory service for entity operations
    pub memory_service: Arc<MemoryService<R>>,
}

impl<R> Ports<R>
where
    R: MemoryRepository + Send + Sync + 'static,
{
    /// Create a new Ports instance
    pub fn new(memory_service: Arc<MemoryService<R>>) -> Self {
        Self { memory_service }
    }
}
