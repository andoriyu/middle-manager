use std::collections::HashMap;
use std::sync::Arc;

use crate::MemoryEntity;
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::CoreResult;
use mm_memory::{MemoryRepository, MemoryService};

/// Request parameters for the create_entity MCP tool
#[mcp_tool(name = "create_entity", description = "Create a new entity in the memory graph")]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateEntityTool {
    /// Name of the entity to create
    pub name: String,
    
    /// Labels for the entity (e.g., ["Memory", "Project"])
    pub labels: Vec<String>,
    
    /// Observations about the entity
    pub observations: Vec<String>,
    
    /// Additional properties for the entity
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// Response for the create_entity MCP tool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateEntityResponse {
    /// Name of the created entity
    pub name: String,
    
    /// Status message
    pub message: String,
}

impl CreateEntityTool {
    /// Execute the create_entity tool
    pub async fn execute<R>(
        self,
        service: Arc<MemoryService<R>>,
    ) -> CoreResult<CreateEntityResponse, R::Error>
    where
        R: MemoryRepository + Send + Sync + 'static,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let entity = MemoryEntity {
            name: self.name.clone(),
            labels: self.labels,
            observations: self.observations,
            properties: self.properties,
        };
        
        service.create_entity(&entity).await?;
        
        Ok(CreateEntityResponse {
            name: self.name,
            message: "Entity created successfully".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MockMemoryRepository, MemoryConfig, MemoryService, ValidationError};
    use crate::error::CoreError;

    #[derive(Debug)]
    struct TestError;
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error")
        }
    }
    impl StdError for TestError {}
    
    #[tokio::test]
    async fn test_create_entity_success() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_create_entity().returning(|_| Ok(()));

        let service = Arc::new(MemoryService::new(mock_repo, MemoryConfig::default()));
        
        let tool = CreateEntityTool {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec!["Test observation".to_string()],
            properties: HashMap::new(),
        };
        
        let result = tool.execute(service).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.name, "test:entity");
        assert_eq!(response.message, "Entity created successfully");
    }
    
    #[tokio::test]
    async fn test_create_entity_validation_error() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_create_entity().returning(|e| {
            Err(CoreError::Validation(ValidationError::NoLabels(e.name.clone())))
        });

        let service = Arc::new(MemoryService::new(mock_repo, MemoryConfig::default()));
        
        let tool = CreateEntityTool {
            name: "test:entity".to_string(),
            labels: vec![], // Service will return a validation error
            observations: vec!["Test observation".to_string()],
            properties: HashMap::new(),
        };
        
        let result = tool.execute(service).await;
        
        assert!(result.is_err());
        match result {
            Err(CoreError::Validation(ValidationError::NoLabels(_))) => (),
            _ => panic!("Expected ValidationError::NoLabels"),
        }
    }
}
