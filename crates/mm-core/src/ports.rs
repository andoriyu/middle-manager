use mm_memory::{MemoryRepository, MemoryService};
use std::sync::{Arc, RwLock};

use crate::RootCollection;

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
    /// Collection of client-provided roots
    pub roots: Arc<RwLock<RootCollection>>,
}

impl<R> Ports<R>
where
    R: MemoryRepository + Send + Sync + 'static,
{
    /// Create a new Ports instance with the given memory service and roots
    pub fn with_roots(
        memory_service: Arc<MemoryService<R>>,
        roots: Arc<RwLock<RootCollection>>,
    ) -> Self {
        Self {
            memory_service,
            roots,
        }
    }

    /// Backwards-compatible constructor that creates empty roots
    pub fn new(memory_service: Arc<MemoryService<R>>) -> Self {
        Self::with_roots(
            memory_service,
            Arc::new(RwLock::new(RootCollection::default())),
        )
    }
}
