use mm_git::{GitRepository, GitService};
use mm_memory::{MemoryRepository, MemoryService};
use std::sync::{Arc, RwLock};

use crate::RootCollection;

/// Ports struct containing all required services for operations
///
/// This struct serves as a dependency injection container for all operations.
/// Each operation function will receive this struct to access the services it needs.
pub struct Ports<MR, GR>
where
    MR: MemoryRepository + Send + Sync + 'static,
    GR: GitRepository + Send + Sync + 'static,
{
    /// Memory service for entity operations
    pub memory_service: Arc<MemoryService<MR>>,
    /// Git service for repository operations
    pub git_service: Arc<GitService<GR>>,
    /// Collection of client-provided roots
    pub roots: Arc<RwLock<RootCollection>>,
}

impl<MR, GR> Ports<MR, GR>
where
    MR: MemoryRepository + Send + Sync + 'static,
    GR: GitRepository + Send + Sync + 'static,
{
    /// Create a new `Ports` instance with the given services and empty roots
    pub fn new(memory_service: Arc<MemoryService<MR>>, git_service: Arc<GitService<GR>>) -> Self {
        Self {
            memory_service,
            git_service,
            roots: Arc::new(RwLock::new(RootCollection::default())),
        }
    }

    /// Create a new `Ports` instance with the given services and roots
    pub fn with_roots(
        memory_service: Arc<MemoryService<MR>>,
        git_service: Arc<GitService<GR>>,
        roots: Arc<RwLock<RootCollection>>,
    ) -> Self {
        Self {
            memory_service,
            git_service,
            roots,
        }
    }
}

#[cfg(any(test, feature = "mock"))]
impl Ports<mm_memory::MockMemoryRepository, mm_git::repository::MockGitRepository> {
    /// Create a new `Ports` instance using noop mock repositories.
    ///
    /// This helper is useful in tests where only a subset of services are
    /// needed. The returned instance provides mock services that do nothing
    /// unless expectations are explicitly set.
    pub fn new_noop() -> Self {
        let memory_service = Arc::new(MemoryService::new(
            mm_memory::MockMemoryRepository::new(),
            mm_memory::MemoryConfig::default(),
        ));
        let git_service = Arc::new(GitService::new(mm_git::repository::MockGitRepository::new()));
        Self::new(memory_service, git_service)
    }
}
