use mm_git::{GitRepository, GitService};
use mm_memory::{MemoryRepository, MemoryService};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::RootCollection;

/// Ports struct containing all required services for operations
///
/// This struct serves as a dependency injection container for all operations.
/// Each operation function will receive this struct to access the services it needs.
pub struct Ports<M, G>
where
    M: MemoryRepository + Send + Sync + 'static,
    G: GitRepository + Send + Sync + 'static,
{
    /// Memory service for entity operations
    pub memory_service: Arc<MemoryService<M>>,
    /// Git service for repository operations
    pub git_service: Arc<GitService<G>>,
    /// Collection of client-provided roots
    pub roots: Arc<RwLock<RootCollection>>,
}

impl<M, G> Ports<M, G>
where
    M: MemoryRepository + Send + Sync + 'static,
    G: GitRepository + Send + Sync + 'static,
{
    /// Create a new Ports instance with all services and roots
    pub fn with_all(
        memory_service: Arc<MemoryService<M>>,
        git_service: Arc<GitService<G>>,
        roots: Arc<RwLock<RootCollection>>,
    ) -> Self {
        Self {
            memory_service,
            git_service,
            roots,
        }
    }

    /// Create a new Ports instance with the given memory service, git service and empty roots
    pub fn new(memory_service: Arc<MemoryService<M>>, git_service: Arc<GitService<G>>) -> Self {
        Self::with_all(
            memory_service,
            git_service,
            Arc::new(RwLock::new(RootCollection::default())),
        )
    }
}

#[cfg(any(test, feature = "mock"))]
impl Ports<mm_memory::MockMemoryRepository, mm_git::repository::MockGitRepository> {
    /// Create a no-op Ports instance with default mock repositories
    ///
    /// This method is only available when the `test` or `mock` feature is enabled.
    pub fn noop() -> Ports<mm_memory::MockMemoryRepository, mm_git::repository::MockGitRepository> {
        use mm_git::repository::MockGitRepository;
        use mm_memory::MockMemoryRepository;

        let memory_repo = MockMemoryRepository::new();
        let git_repo = MockGitRepository::new();

        let memory_service = Arc::new(MemoryService::new(
            memory_repo,
            mm_memory::MemoryConfig::default(),
        ));
        let git_service = Arc::new(GitService::new(git_repo));

        Ports {
            memory_service,
            git_service,
            roots: Arc::new(RwLock::new(RootCollection::default())),
        }
    }

    /// can be used to create Ports for tests
    /// Ports::noop().with(|ports| ports.git_service = git_service )
    pub fn with(mut self, configure_fn: impl FnOnce(&mut Self)) -> Self {
        configure_fn(&mut self);
        self
    }
}
