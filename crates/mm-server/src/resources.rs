use mm_core::Ports;
use mm_core::operations::memory::{GetEntityCommand, get_entity};
use mm_git::GitRepository;
use mm_memory::MemoryRepository;
use rust_mcp_sdk::schema::{
    ListResourceTemplatesResult, ListResourcesResult, ReadResourceResult,
    ReadResourceResultContentsItem, ResourceTemplate, RpcError, TextResourceContents,
};

/// Return the list of resource templates supported by the server.
pub fn list_resource_templates() -> ListResourceTemplatesResult {
    ListResourceTemplatesResult {
        meta: None,
        next_cursor: None,
        resource_templates: vec![ResourceTemplate {
            annotations: None,
            description: Some("Retrieve a memory entity by name".to_string()),
            mime_type: Some("application/json".to_string()),
            name: "Memory Entity".to_string(),
            uri_template: "memory://{name}".to_string(),
        }],
    }
}

/// Return the list of resources. Dynamic memory resources are not enumerated, so this is empty.
pub fn list_resources() -> ListResourcesResult {
    ListResourcesResult {
        meta: None,
        next_cursor: None,
        resources: vec![],
    }
}

/// Read a memory entity from the given URI.
#[tracing::instrument(skip(ports), fields(uri))]
pub async fn read_resource<M, G>(
    ports: &Ports<M, G>,
    uri: &str,
) -> Result<ReadResourceResult, RpcError>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    let Some(name) = uri.strip_prefix("memory://") else {
        return Err(RpcError::invalid_params().with_message("Unsupported URI".to_string()));
    };

    let entity = get_entity(
        ports,
        GetEntityCommand {
            name: name.to_string(),
        },
    )
    .await
    .map_err(|e| RpcError::internal_error().with_message(e.to_string()))?;

    let Some(entity) = entity else {
        return Err(
            RpcError::method_not_found().with_message(format!("Entity '{}' not found", name))
        );
    };

    let text = serde_json::to_string(&entity)
        .map_err(|e| RpcError::internal_error().with_message(e.to_string()))?;

    Ok(ReadResourceResult {
        contents: vec![ReadResourceResultContentsItem::TextResourceContents(
            TextResourceContents {
                mime_type: Some("application/json".to_string()),
                text,
                uri: uri.to_string(),
            },
        )],
        meta: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_read_resource_success() {
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            ..Default::default()
        };

        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let result = read_resource(&ports, "memory://test:entity").await.unwrap();
        if let ReadResourceResultContentsItem::TextResourceContents(contents) = &result.contents[0]
        {
            assert!(contents.text.contains("test:entity"));
        } else {
            panic!("unexpected contents variant");
        }
    }

    #[tokio::test]
    async fn test_read_resource_not_found() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("missing"))
            .returning(|_| Ok(None));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });
        let err = read_resource(&ports, "memory://missing").await.unwrap_err();
        assert_eq!(err.message, "Entity 'missing' not found");
    }

    #[tokio::test]
    async fn test_read_resource_invalid_uri() {
        let mock = MockMemoryRepository::new();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });
        let err = read_resource(&ports, "file://foo").await.unwrap_err();
        assert_eq!(err.message, "Unsupported URI");
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use arbitrary::{Arbitrary, Unstructured};
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mm_utils::prop::{NonEmptyName, async_arbtest};
    use std::sync::Arc;

    #[test]
    fn prop_read_resource_not_found() {
        async_arbtest(|rt, u| {
            let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
            let uri = format!("memory://{}", name);
            let mut mock = MockMemoryRepository::new();
            let name_clone = name.clone();
            mock.expect_find_entity_by_name()
                .withf(move |n| n == name_clone)
                .returning(|_| Ok(None));
            let service = MemoryService::new(mock, MemoryConfig::default());
            let ports = Ports::noop().with(|p| {
                p.memory_service = Arc::new(service);
            });
            let err = rt.block_on(read_resource(&ports, &uri)).unwrap_err();
            assert_eq!(err.message, format!("Entity '{}' not found", name));
            Ok(())
        });
    }

    #[derive(Debug)]
    struct InvalidUri(String);

    impl<'a> Arbitrary<'a> for InvalidUri {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let mut s: String = Arbitrary::arbitrary(u)?;
            if s.starts_with("memory://") {
                s.insert(0, 'x');
            }
            Ok(InvalidUri(s))
        }
    }

    #[test]
    fn prop_read_resource_invalid_prefix() {
        async_arbtest(|rt, u| {
            let InvalidUri(uri) = InvalidUri::arbitrary(u)?;
            let mut mock = MockMemoryRepository::new();
            mock.expect_find_entity_by_name().never();
            let service = MemoryService::new(mock, MemoryConfig::default());
            let ports = Ports::noop().with(|p| {
                p.memory_service = Arc::new(service);
            });
            let err = rt.block_on(read_resource(&ports, &uri)).unwrap_err();
            assert_eq!(err.message, "Unsupported URI");
            Ok(())
        });
    }
}
