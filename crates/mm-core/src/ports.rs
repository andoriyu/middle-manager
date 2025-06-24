use mm_git::GitServiceTrait;
use mm_memory::{MemoryRepository, MemoryService};
use std::sync::{Arc, RwLock};

use crate::RootCollection;

/// Ports struct containing all required services for operations
///
/// This struct serves as a dependency injection container for all operations.
/// Each operation function will receive this struct to access the services it needs.
pub struct Ports<R, G>
where
    R: MemoryRepository + Send + Sync + 'static,
    G: GitServiceTrait + Send + Sync + 'static,
{
    /// Memory service for entity operations
    pub memory_service: Arc<MemoryService<R>>,
    /// Git service for repository operations
    pub git_service: Arc<G>,
    /// Collection of client-provided roots
    pub roots: Arc<RwLock<RootCollection>>,
}

impl<R, G> Ports<R, G>
where
    R: MemoryRepository + Send + Sync + 'static,
    G: GitServiceTrait + Send + Sync + 'static,
{
    /// Create a new Ports instance with the given memory and git services and roots
    pub fn with_roots(
        memory_service: Arc<MemoryService<R>>,
        git_service: Arc<G>,
        roots: Arc<RwLock<RootCollection>>,
    ) -> Self {
        Self {
            memory_service,
            git_service,
            roots,
        }
    }

    /// Create a Ports instance with default empty roots
    pub fn new(memory_service: Arc<MemoryService<R>>, git_service: Arc<G>) -> Self {
        Self::with_roots(
            memory_service,
            git_service,
            Arc::new(RwLock::new(RootCollection::default())),
        )
    }
}
