use std::sync::Arc;

use crate::MemoryEntity;
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use mm_memory::{MemoryRepository, MemoryService};

/// Request parameters for the get_entity MCP tool
#[mcp_tool(name = "get_entity", description = "Get an entity from the memory graph by name")]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetEntityTool {
    /// Name of the entity to retrieve
    pub name: String,
}

/// Response for the get_entity MCP tool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEntityResponse {
    /// The retrieved entity
    pub entity: MemoryEntity,
}

impl GetEntityTool {
    /// Execute the get_entity tool
    pub async fn execute<R>(
        self,
        service: Arc<MemoryService<R>>,
    ) -> CoreResult<GetEntityResponse, R::Error>
    where
        R: MemoryRepository + Send + Sync + 'static,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let entity = service.find_entity_by_name(&self.name).await?;
        
        match entity {
            Some(entity) => Ok(GetEntityResponse { entity }),
            None => Err(CoreError::NotFound(format!("Entity '{}' not found", self.name))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MockMemoryRepository, MemoryConfig, MemoryService};
    use mockall::predicate::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct TestError;
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error")
        }
    }
    impl StdError for TestError {}
    
    #[tokio::test]
    async fn test_get_entity_found() {
        let mut mock_repo = MockMemoryRepository::new();
        
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec!["Test observation".to_string()],
            properties: HashMap::new(),
        };
        
        // Set up the mock expectation
        mock_repo
            .expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = Arc::new(MemoryService::new(mock_repo, MemoryConfig::default()));
        
        let tool = GetEntityTool {
            name: "test:entity".to_string(),
        };
        
        let result = tool.execute(service).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.entity.name, "test:entity");
    }
    
    #[tokio::test]
    async fn test_get_entity_not_found() {
        let mut mock_repo = MockMemoryRepository::new();
        
        // Set up the mock expectation
        mock_repo
            .expect_find_entity_by_name()
            .with(eq("nonexistent:entity"))
            .returning(|_| Ok(None));

        let service = Arc::new(MemoryService::new(mock_repo, MemoryConfig::default()));
        
        let tool = GetEntityTool {
            name: "nonexistent:entity".to_string(),
        };
        
        let result = tool.execute(service).await;
        
        assert!(result.is_err());
        match result {
            Err(CoreError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }
}
