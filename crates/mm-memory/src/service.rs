use crate::{
    DEFAULT_LABELS, DEFAULT_RELATIONSHIPS, MemoryConfig, MemoryEntity, MemoryRelationship,
    MemoryRepository, MemoryResult, ValidationError, ValidationErrorKind,
};
use mm_utils::is_snake_case;
use tracing::instrument;

/// Service for memory operations
///
/// This service provides a high-level API for interacting with the memory store.
/// It uses a repository to perform the actual operations, and adds additional
/// business logic as needed.
///
/// The service is generic over the repository type `R`, allowing it to work with
/// different repository implementations.
pub struct MemoryService<R>
where
    R: MemoryRepository,
{
    /// The repository used to perform memory operations
    repository: R,

    /// Configuration for the service
    config: MemoryConfig,
}

impl<R> MemoryService<R>
where
    R: MemoryRepository + Sync,
{
    /// Create a new memory service with the given repository
    pub fn new(repository: R, config: MemoryConfig) -> Self {
        Self { repository, config }
    }

    /// Create multiple entities in a batch
    #[instrument(skip(self, entities), fields(entities_count = entities.len()))]
    pub async fn create_entities(
        &self,
        entities: &[MemoryEntity],
    ) -> MemoryResult<Vec<(String, ValidationError)>, R::Error> {
        let mut errors = Vec::new();
        let mut valid = Vec::new();

        for entity in entities {
            let mut tagged = entity.clone();
            if let Some(tag) = &self.config.default_tag {
                if !tagged.labels.contains(tag) {
                    tagged.labels.push(tag.clone());
                }
            }

            let mut errs = Vec::new();
            if tagged.name.is_empty() {
                errs.push(ValidationErrorKind::EmptyEntityName);
            }
            if tagged.labels.is_empty() {
                errs.push(ValidationErrorKind::NoLabels(tagged.name.clone()));
            }
            if self.config.default_labels {
                for label in &tagged.labels {
                    let allowed_default_tag =
                        self.config.default_tag.as_deref() == Some(label.as_str());
                    if !allowed_default_tag
                        && !DEFAULT_LABELS.contains(&label.as_str())
                        && !self.config.additional_labels.contains(label)
                    {
                        errs.push(ValidationErrorKind::UnknownLabel(label.clone()));
                    }
                }
            }

            if errs.is_empty() {
                valid.push(tagged);
            } else {
                errors.push((entity.name.clone(), ValidationError(errs)));
            }
        }

        if !valid.is_empty() {
            self.repository.create_entities(&valid).await?;
        }

        Ok(errors)
    }

    /// Find an entity by name
    #[instrument(skip(self), fields(name))]
    pub async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity>, R::Error> {
        self.repository.find_entity_by_name(name).await
    }

    /// Replace all observations for an entity
    #[instrument(skip(self, observations), fields(name, observations_count = observations.len()))]
    pub async fn set_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), R::Error> {
        self.repository.set_observations(name, observations).await
    }

    /// Add observations to an entity
    #[instrument(skip(self, observations), fields(name, observations_count = observations.len()))]
    pub async fn add_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), R::Error> {
        self.repository.add_observations(name, observations).await
    }

    /// Remove all observations from an entity
    #[instrument(skip(self), fields(name))]
    pub async fn remove_all_observations(&self, name: &str) -> MemoryResult<(), R::Error> {
        self.repository.remove_all_observations(name).await
    }

    /// Remove specific observations from an entity
    #[instrument(skip(self, observations), fields(name, observations_count = observations.len()))]
    pub async fn remove_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), R::Error> {
        self.repository
            .remove_observations(name, observations)
            .await
    }

    /// Create multiple relationships in a batch
    #[instrument(skip(self, relationships), fields(relationships_count = relationships.len()))]
    pub async fn create_relationships(
        &self,
        relationships: &[MemoryRelationship],
    ) -> MemoryResult<Vec<(String, ValidationError)>, R::Error> {
        let mut errors = Vec::new();
        let mut valid = Vec::new();

        for rel in relationships {
            let mut errs = Vec::new();
            if rel.from.is_empty() || rel.to.is_empty() {
                errs.push(ValidationErrorKind::EmptyEntityName);
            }
            if !is_snake_case(&rel.name) {
                errs.push(ValidationErrorKind::InvalidRelationshipFormat(
                    rel.name.clone(),
                ));
            }
            if self.config.default_relationships
                && !DEFAULT_RELATIONSHIPS.contains(&rel.name.as_str())
                && !self.config.additional_relationships.contains(&rel.name)
            {
                errs.push(ValidationErrorKind::UnknownRelationship(rel.name.clone()));
            }

            if errs.is_empty() {
                valid.push(rel.clone());
            } else {
                errors.push((rel.name.clone(), ValidationError(errs)));
            }
        }

        if !valid.is_empty() {
            self.repository.create_relationships(&valid).await?;
        }

        Ok(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockMemoryRepository;
    use crate::ValidationErrorKind;
    use std::collections::{HashMap, HashSet};

    #[tokio::test]
    async fn test_default_tag_added() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && e[0].labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_no_default_tag() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && !e[0].labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_empty_labels_adds_default_tag() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| {
                e.len() == 1
                    && e[0].labels.len() == 1
                    && e[0].labels.contains(&"Memory".to_string())
            })
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let errors = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_empty_labels_without_default_tag_fails() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities().never();

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.iter().any(|(n, e)| {
            n == "test:entity"
                && e.0
                    .contains(&ValidationErrorKind::NoLabels("test:entity".to_string()))
        }));
    }

    #[tokio::test]
    async fn test_default_tag_allowed_with_label_validation() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && e[0].labels == ["Custom".to_string()])
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Custom".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: true,
                additional_labels: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_create_entity_unknown_label() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities().never();

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: true,
                additional_labels: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Unknown".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.iter().any(|(n, e)| {
            n == "test:entity"
                && e.0
                    .contains(&ValidationErrorKind::UnknownLabel("Unknown".to_string()))
        }));
    }

    #[tokio::test]
    async fn test_create_relationship_allowed() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships().returning(|_| Ok(()));
        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: true,
                additional_labels: HashSet::new(),
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "relates_to".to_string(),
            properties: HashMap::new(),
        };

        let errors = service
            .create_relationships(std::slice::from_ref(&rel))
            .await
            .unwrap();
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_create_relationship_unknown() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships().never();
        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: true,
                additional_labels: HashSet::new(),
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "custom_rel".to_string(),
            properties: HashMap::new(),
        };

        let result = service
            .create_relationships(std::slice::from_ref(&rel))
            .await
            .unwrap();
        assert!(result.iter().any(|(n, e)| {
            n == "custom_rel"
                && e.0.contains(&ValidationErrorKind::UnknownRelationship(
                    "custom_rel".to_string(),
                ))
        }));
    }

    #[tokio::test]
    async fn test_create_entity_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .returning(|_| Err(crate::MemoryError::query_error("fail")));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = service.create_entities(std::slice::from_ref(&entity)).await;
        assert!(matches!(result, Err(crate::MemoryError::QueryError { .. })));
    }

    #[tokio::test]
    async fn test_create_relationship_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships()
            .returning(|_| Err(crate::MemoryError::query_error("fail")));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
                default_labels: true,
                additional_labels: HashSet::new(),
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "relates_to".to_string(),
            properties: HashMap::new(),
        };

        let result = service
            .create_relationships(std::slice::from_ref(&rel))
            .await;
        assert!(matches!(result, Err(crate::MemoryError::QueryError { .. })));
    }

    mod prop {
        use super::*;
        use arbitrary::{Arbitrary, Unstructured};
        use mm_utils::prop::{self, NonEmptyName, async_arbtest};

        impl<'a> Arbitrary<'a> for MemoryEntity {
            fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                if u.is_empty() {
                    return Ok(Self {
                        name: String::new(),
                        labels: Vec::new(),
                        observations: Vec::new(),
                        properties: std::collections::HashMap::new(),
                    });
                }
                let name = prop::small_string(u)?;
                let label_count = u.int_in_range::<usize>(0..=3)?;
                let mut labels = Vec::new();
                for _ in 0..label_count {
                    labels.push(prop::small_string(u)?);
                }
                let obs_count = u.int_in_range::<usize>(0..=3)?;
                let mut observations = Vec::new();
                for _ in 0..obs_count {
                    observations.push(prop::small_string(u)?);
                }
                let prop_count = u.int_in_range::<usize>(0..=3)?;
                let mut properties = std::collections::HashMap::new();
                for _ in 0..prop_count {
                    properties.insert(prop::small_string(u)?, prop::small_string(u)?);
                }
                Ok(Self {
                    name,
                    labels,
                    observations,
                    properties,
                })
            }
        }

        impl<'a> Arbitrary<'a> for MemoryRelationship {
            fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                if u.is_empty() {
                    return Ok(Self {
                        from: String::new(),
                        to: String::new(),
                        name: String::new(),
                        properties: std::collections::HashMap::new(),
                    });
                }
                let from = prop::small_string(u)?;
                let to = prop::small_string(u)?;
                let name = prop::small_string(u)?;
                let prop_count = u.int_in_range::<usize>(0..=3)?;
                let mut properties = std::collections::HashMap::new();
                for _ in 0..prop_count {
                    properties.insert(prop::small_string(u)?, prop::small_string(u)?);
                }
                Ok(Self {
                    from,
                    to,
                    name,
                    properties,
                })
            }
        }

        #[test]
        fn prop_create_entities_success() {
            async_arbtest(|rt, u| {
                let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
                let idx = u.int_in_range::<usize>(0..=DEFAULT_LABELS.len() - 1)?;
                let label = DEFAULT_LABELS[idx].to_string();
                let mut entity = MemoryEntity::arbitrary(u)?;
                entity.name = name.clone();
                entity.labels = vec![label];

                let mut mock = MockMemoryRepository::new();
                let name_clone = name.clone();
                mock.expect_create_entities()
                    .withf(move |e| e.len() == 1 && e[0].name == name_clone)
                    .returning(|_| Ok(()));
                let service = MemoryService::new(mock, MemoryConfig::default());
                let result = rt
                    .block_on(service.create_entities(std::slice::from_ref(&entity)))
                    .unwrap();
                assert!(result.is_empty());
                Ok(())
            });
        }

        #[test]
        fn prop_create_entities_empty_name() {
            async_arbtest(|rt, u| {
                let idx = u.int_in_range::<usize>(0..=DEFAULT_LABELS.len() - 1)?;
                let label = DEFAULT_LABELS[idx].to_string();
                let mut entity = MemoryEntity::arbitrary(u)?;
                entity.name = String::new();
                entity.labels = vec![label];

                let mut mock = MockMemoryRepository::new();
                mock.expect_create_entities().never();
                let service = MemoryService::new(mock, MemoryConfig::default());
                let result = rt
                    .block_on(service.create_entities(std::slice::from_ref(&entity)))
                    .unwrap();
                assert!(
                    result.iter().any(|(n, e)| n.is_empty()
                        && e.0.contains(&ValidationErrorKind::EmptyEntityName))
                );
                Ok(())
            });
        }

        #[test]
        fn prop_create_entities_unknown_label() {
            async_arbtest(|rt, u| {
                let NonEmptyName(name) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let mut label = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v.0,
                    Err(_) => return Ok(()),
                };
                if DEFAULT_LABELS.contains(&label.as_str())
                    || label == MemoryConfig::default().default_tag.clone().unwrap()
                {
                    label.push_str("_x");
                }
                let mut entity = match MemoryEntity::arbitrary(u) {
                    Ok(e) => e,
                    Err(_) => return Ok(()),
                };
                entity.name = name.clone();
                entity.labels = vec![label.clone()];

                let mut mock = MockMemoryRepository::new();
                mock.expect_create_entities().never();
                let service = MemoryService::new(mock, MemoryConfig::default());
                let result = rt
                    .block_on(service.create_entities(std::slice::from_ref(&entity)))
                    .unwrap();
                assert!(result.iter().any(|(n, e)| {
                    n == &name
                        && e.0
                            .contains(&ValidationErrorKind::UnknownLabel(label.clone()))
                }));
                Ok(())
            });
        }

        #[test]
        fn prop_create_relationships_success() {
            async_arbtest(|rt, u| {
                let NonEmptyName(from) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let NonEmptyName(to) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let idx = match u.int_in_range::<usize>(0..=DEFAULT_RELATIONSHIPS.len() - 1) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let name = DEFAULT_RELATIONSHIPS[idx].to_string();
                let mut rel = match MemoryRelationship::arbitrary(u) {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };
                rel.from = from.clone();
                rel.to = to.clone();
                rel.name = name.clone();

                let mut mock = MockMemoryRepository::new();
                let from_clone = from.clone();
                let to_clone = to.clone();
                let name_clone = name.clone();
                mock.expect_create_relationships()
                    .withf(move |r| {
                        r.len() == 1
                            && r[0].from == from_clone
                            && r[0].to == to_clone
                            && r[0].name == name_clone
                    })
                    .returning(|_| Ok(()));
                let service = MemoryService::new(mock, MemoryConfig::default());
                let result = rt
                    .block_on(service.create_relationships(std::slice::from_ref(&rel)))
                    .unwrap();
                assert!(result.is_empty());
                Ok(())
            });
        }

        #[test]
        fn prop_create_relationships_invalid_format() {
            async_arbtest(|rt, u| {
                let NonEmptyName(from) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let NonEmptyName(to) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let idx = match u.int_in_range::<usize>(0..=DEFAULT_RELATIONSHIPS.len() - 1) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let mut name = DEFAULT_RELATIONSHIPS[idx].to_string();
                name.push('A');
                let mut rel = match MemoryRelationship::arbitrary(u) {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };
                rel.from = from;
                rel.to = to;
                rel.name = name.clone();

                let mut mock = MockMemoryRepository::new();
                mock.expect_create_relationships().never();
                let service = MemoryService::new(mock, MemoryConfig::default());
                let result = rt
                    .block_on(service.create_relationships(std::slice::from_ref(&rel)))
                    .unwrap();
                assert!(result.iter().any(|(n, e)| {
                    n == &name
                        && e.0
                            .contains(&ValidationErrorKind::InvalidRelationshipFormat(
                                name.clone(),
                            ))
                }));
                Ok(())
            });
        }

        #[test]
        fn prop_create_relationships_unknown_relationship() {
            async_arbtest(|rt, u| {
                let NonEmptyName(from) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let NonEmptyName(to) = match NonEmptyName::arbitrary(u) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let idx = match u.int_in_range::<usize>(0..=DEFAULT_RELATIONSHIPS.len() - 1) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let mut name = DEFAULT_RELATIONSHIPS[idx].to_string();
                name.push_str("_extra");
                let mut rel = match MemoryRelationship::arbitrary(u) {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };
                rel.from = from.clone();
                rel.to = to.clone();
                rel.name = name.clone();

                let mut mock = MockMemoryRepository::new();
                mock.expect_create_relationships().never();
                let service = MemoryService::new(mock, MemoryConfig::default());
                let result = rt
                    .block_on(service.create_relationships(std::slice::from_ref(&rel)))
                    .unwrap();
                assert!(result.iter().any(|(n, e)| {
                    n == &name
                        && e.0
                            .contains(&ValidationErrorKind::UnknownRelationship(name.clone()))
                }));
                Ok(())
            });
        }
    }
}
