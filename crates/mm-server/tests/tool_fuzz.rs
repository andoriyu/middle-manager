use arbitrary::Arbitrary;
use mm_core::Ports;
use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
use mm_server::{
    AddObservationsTool, CreateEntityTool, CreateRelationshipTool, GetEntityTool,
    RelationshipInput, RemoveAllObservationsTool, RemoveObservationsTool, SetObservationsTool,
};
use mm_utils::prop::async_arbtest;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn fuzz_add_observations_tool() {
    async_arbtest(|rt, u| {
        let tool = AddObservationsTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let name = tool.name.clone();
        let obs = tool.observations.clone();
        mock.expect_add_observations()
            .withf(move |n, o| n == name && o == obs.as_slice())
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

#[test]
fn fuzz_set_observations_tool() {
    async_arbtest(|rt, u| {
        let tool = SetObservationsTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let name = tool.name.clone();
        let obs = tool.observations.clone();
        mock.expect_set_observations()
            .withf(move |n, o| n == name && o == obs.as_slice())
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

#[test]
fn fuzz_remove_observations_tool() {
    async_arbtest(|rt, u| {
        let tool = RemoveObservationsTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let name = tool.name.clone();
        let obs = tool.observations.clone();
        mock.expect_remove_observations()
            .withf(move |n, o| n == name && o == obs.as_slice())
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

#[test]
fn fuzz_remove_all_observations_tool() {
    async_arbtest(|rt, u| {
        let tool = RemoveAllObservationsTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let name = tool.name.clone();
        mock.expect_remove_all_observations()
            .withf(move |n| n == name)
            .returning(|_| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

#[test]
fn fuzz_get_entity_tool() {
    async_arbtest(|rt, u| {
        let tool = GetEntityTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let name = tool.name.clone();
        let name_with = name.clone();
        mock.expect_find_entity_by_name()
            .withf(move |n| n == name_with)
            .returning(move |_| {
                Ok(Some(MemoryEntity {
                    name: name.clone(),
                    labels: vec![],
                    observations: vec![],
                    properties: HashMap::new(),
                }))
            });
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

#[test]
fn fuzz_create_entity_tool() {
    async_arbtest(|rt, u| {
        let tool = CreateEntityTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let expected = tool.entities.clone();
        mock.expect_create_entities()
            .withf(move |ents| ents == expected.as_slice())
            .returning(|_| Ok(()));
        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_labels: false,
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

#[test]
fn fuzz_create_relationship_tool() {
    async_arbtest(|rt, u| {
        let tool = CreateRelationshipTool::arbitrary(u)?;
        let mut mock = MockMemoryRepository::new();
        let expected: Vec<_> = tool
            .relationships
            .iter()
            .map(|r| {
                (
                    r.from.clone(),
                    r.to.clone(),
                    r.name.clone(),
                    r.properties.clone().unwrap_or_default(),
                )
            })
            .collect();
        let expected_clone = expected.clone();
        mock.expect_create_relationships()
            .withf(move |rels| {
                rels.len() == expected_clone.len()
                    && rels.iter().zip(expected_clone.iter()).all(|(a, e)| {
                        a.from == e.0 && a.to == e.1 && a.name == e.2 && a.properties == e.3
                    })
            })
            .returning(|_| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let result = rt.block_on(tool.call_tool(&ports));
        assert!(result.is_ok());
        Ok(())
    });
}

// Invalid parameter checks

#[tokio::test]
async fn invalid_add_observations_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_add_observations().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = AddObservationsTool {
        name: String::new(),
        observations: vec![],
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_set_observations_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_set_observations().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = SetObservationsTool {
        name: String::new(),
        observations: vec![],
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_remove_observations_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_remove_observations().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = RemoveObservationsTool {
        name: String::new(),
        observations: vec![],
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_remove_all_observations_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_remove_all_observations().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = RemoveAllObservationsTool {
        name: String::new(),
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_get_entity_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_find_entity_by_name().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = GetEntityTool {
        name: String::new(),
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_create_entity_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_create_entities().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = CreateEntityTool {
        entities: vec![MemoryEntity {
            name: String::new(),
            labels: vec!["Memory".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        }],
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn invalid_create_relationship_empty_name() {
    let mut mock = MockMemoryRepository::new();
    mock.expect_create_relationships().never();
    let service = MemoryService::new(mock, MemoryConfig::default());
    let ports = Ports::new(Arc::new(service));
    let tool = CreateRelationshipTool {
        relationships: vec![RelationshipInput {
            from: "a".to_string(),
            to: "b".to_string(),
            name: String::new(),
            properties: None,
        }],
    };
    let result = tool.call_tool(&ports).await;
    assert!(result.is_err());
}
