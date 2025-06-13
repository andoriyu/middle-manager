use std::error::Error as StdError;
use std::sync::Arc;

use mm_memory_neo4j::MemoryEntity;
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::service::MemoryService;

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
    pub async fn execute<E, S>(
        self,
        service: Arc<S>,
    ) -> CoreResult<GetEntityResponse, E>
    where
        E: StdError + Send + Sync + 'static,
        S: MemoryService<E> + Send + Sync + 'static,
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
    use crate::service::MockMemoryService;
    use mm_memory_neo4j::neo4rs;
    use mockall::predicate::*;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_get_entity_found() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec!["Test observation".to_string()],
            properties: HashMap::new(),
        };
        
        // Set up the mock expectation
        mock.expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));
        
        let service = Arc::new(mock);
        
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
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        
        // Set up the mock expectation
        mock.expect_find_entity_by_name()
            .with(eq("nonexistent:entity"))
            .returning(|_| Ok(None));
        
        let service = Arc::new(mock);
        
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
