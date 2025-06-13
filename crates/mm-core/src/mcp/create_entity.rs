use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

use mm_memory_neo4j::MemoryEntity;
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::CoreResult;
use crate::service::MemoryService;

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
    pub async fn execute<E, S>(
        self,
        service: Arc<S>,
    ) -> CoreResult<CreateEntityResponse, E>
    where
        E: StdError + Send + Sync + 'static,
        S: MemoryService<E> + Send + Sync + 'static,
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
    use crate::service::MockMemoryService;
    use mm_memory_neo4j::{ValidationError, neo4rs};
    use crate::error::CoreError;
    
    #[tokio::test]
    async fn test_create_entity_success() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        
        // Set up the mock expectation
        mock.expect_create_entity()
            .returning(|_| Ok(()));
        
        let service = Arc::new(mock);
        
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
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        
        // Set up the mock expectation
        mock.expect_create_entity()
            .returning(|e| Err(CoreError::Validation(ValidationError::NoLabels(e.name.clone()))));
        
        let service = Arc::new(mock);
        
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
