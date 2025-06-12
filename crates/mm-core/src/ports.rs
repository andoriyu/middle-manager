use std::sync::Arc;
use crate::MemoryService;
use crate::neo4rs;

/// Ports struct containing all required services for operations
/// 
/// This struct serves as a dependency injection container for all operations.
/// Each operation function will receive this struct to access the services it needs.
pub struct Ports {
    /// Memory service for entity operations
    pub memory_service: Arc<dyn MemoryService<neo4rs::Error> + Send + Sync>,
}

impl Ports {
    /// Create a new Ports instance
    pub fn new(memory_service: Arc<dyn MemoryService<neo4rs::Error> + Send + Sync>) -> Self {
        Self { memory_service }
    }
}
