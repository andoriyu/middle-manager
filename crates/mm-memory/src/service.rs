#[cfg(test)]
use crate::ObservationsUpdate;
use crate::value::MemoryValue;
use crate::{
    DEFAULT_LABELS, DEFAULT_RELATIONSHIPS, EntityUpdate, LabelMatchMode, MemoryConfig,
    MemoryEntity, MemoryRelationship, MemoryRepository, MemoryResult, RelationshipDirection,
    RelationshipUpdate, ValidationError, ValidationErrorKind,
};
use mm_utils::is_snake_case;
use schemars::JsonSchema;
use std::collections::HashMap;
use tracing::instrument;

/// Minimum allowed traversal depth for related entity queries
const MIN_TRAVERSAL_DEPTH: u32 = 1;

/// Maximum allowed traversal depth for related entity queries
const MAX_TRAVERSAL_DEPTH: u32 = 5;

fn to_default_entity<P>(entity: MemoryEntity<P>) -> MemoryEntity
where
    P: JsonSchema
        + Into<HashMap<String, MemoryValue>>
        + From<HashMap<String, MemoryValue>>
        + Clone
        + std::fmt::Debug
        + Default,
{
    MemoryEntity {
        name: entity.name,
        labels: entity.labels,
        observations: entity.observations,
        properties: entity.properties.into(),
        relationships: entity
            .relationships
            .into_iter()
            .map(to_default_relationship)
            .collect(),
    }
}

fn to_default_relationship<P>(rel: MemoryRelationship<P>) -> MemoryRelationship
where
    P: JsonSchema
        + Into<HashMap<String, MemoryValue>>
        + From<HashMap<String, MemoryValue>>
        + Clone
        + std::fmt::Debug
        + Default,
{
    MemoryRelationship {
        from: rel.from,
        to: rel.to,
        name: rel.name,
        properties: rel.properties.into(),
    }
}

fn from_default_entity<P>(entity: MemoryEntity) -> MemoryEntity<P>
where
    P: JsonSchema
        + Into<HashMap<String, MemoryValue>>
        + From<HashMap<String, MemoryValue>>
        + Clone
        + std::fmt::Debug
        + Default,
{
    MemoryEntity {
        name: entity.name,
        labels: entity.labels,
        observations: entity.observations,
        properties: P::from(entity.properties),
        relationships: entity
            .relationships
            .into_iter()
            .map(from_default_relationship::<P>)
            .collect(),
    }
}

fn from_default_relationship<P>(rel: MemoryRelationship) -> MemoryRelationship<P>
where
    P: JsonSchema
        + Into<HashMap<String, MemoryValue>>
        + From<HashMap<String, MemoryValue>>
        + Clone
        + std::fmt::Debug
        + Default,
{
    MemoryRelationship {
        from: rel.from,
        to: rel.to,
        name: rel.name,
        properties: P::from(rel.properties),
    }
}

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
    pub async fn create_entities_typed<P>(
        &self,
        entities: &[MemoryEntity<P>],
    ) -> MemoryResult<Vec<(String, ValidationError)>, R::Error>
    where
        P: JsonSchema
            + Into<HashMap<String, MemoryValue>>
            + From<HashMap<String, MemoryValue>>
            + Clone
            + std::fmt::Debug
            + Default,
    {
        let mut errors = Vec::default();
        let mut valid = Vec::default();

        for entity in entities {
            let mut tagged = entity.clone();
            if let Some(label) = &self.config.default_label {
                if !tagged.labels.contains(label) {
                    tagged.labels.push(label.clone());
                }
            }

            let mut errs = Vec::default();
            if tagged.name.is_empty() {
                errs.push(ValidationErrorKind::EmptyEntityName);
            }
            if tagged.labels.is_empty() {
                errs.push(ValidationErrorKind::NoLabels(tagged.name.clone()));
            }
            if self.config.default_labels {
                for label in &tagged.labels {
                    let allowed_default_label =
                        self.config.default_label.as_deref() == Some(label.as_str());
                    if !allowed_default_label
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
            let mapped: Vec<MemoryEntity> = valid.into_iter().map(to_default_entity).collect();
            self.repository.create_entities(&mapped).await?;
        }

        Ok(errors)
    }

    /// Create multiple entities using the default HashMap property type
    #[instrument(skip(self, entities), fields(entities_count = entities.len()))]
    pub async fn create_entities(
        &self,
        entities: &[MemoryEntity],
    ) -> MemoryResult<Vec<(String, ValidationError)>, R::Error> {
        self.create_entities_typed::<HashMap<String, MemoryValue>>(entities)
            .await
    }

    /// Find an entity by name
    #[instrument(skip(self), fields(name))]
    pub async fn find_entity_by_name_typed<P>(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity<P>>, R::Error>
    where
        P: JsonSchema
            + From<HashMap<String, MemoryValue>>
            + Into<HashMap<String, MemoryValue>>
            + Clone
            + std::fmt::Debug
            + Default,
    {
        let result = self.repository.find_entity_by_name(name).await?;
        Ok(result.map(from_default_entity::<P>))
    }

    /// Find an entity by name using the default HashMap property type
    #[instrument(skip(self), fields(name))]
    pub async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity>, R::Error> {
        self.find_entity_by_name_typed::<HashMap<String, MemoryValue>>(name)
            .await
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
    pub async fn create_relationships_typed<P>(
        &self,
        relationships: &[MemoryRelationship<P>],
    ) -> MemoryResult<Vec<(String, ValidationError)>, R::Error>
    where
        P: JsonSchema
            + Into<HashMap<String, MemoryValue>>
            + From<HashMap<String, MemoryValue>>
            + Clone
            + std::fmt::Debug
            + Default,
    {
        let mut errors = Vec::default();
        let mut valid = Vec::default();

        for rel in relationships {
            let mut errs = Vec::default();
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
            let mapped: Vec<MemoryRelationship> =
                valid.into_iter().map(to_default_relationship).collect();
            self.repository.create_relationships(&mapped).await?;
        }

        Ok(errors)
    }

    /// Create relationships using the default HashMap property type
    #[instrument(skip(self, relationships), fields(relationships_count = relationships.len()))]
    pub async fn create_relationships(
        &self,
        relationships: &[MemoryRelationship],
    ) -> MemoryResult<Vec<(String, ValidationError)>, R::Error> {
        self.create_relationships_typed::<HashMap<String, MemoryValue>>(relationships)
            .await
    }

    /// Find entities related to the given entity
    #[instrument(skip(self), fields(name, depth))]
    pub async fn find_related_entities_typed<P>(
        &self,
        name: &str,
        relationship_type: Option<String>,
        direction: Option<RelationshipDirection>,
        depth: u32,
    ) -> MemoryResult<Vec<MemoryEntity<P>>, R::Error>
    where
        P: JsonSchema
            + From<HashMap<String, MemoryValue>>
            + Into<HashMap<String, MemoryValue>>
            + Clone
            + std::fmt::Debug
            + Default,
    {
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }
        if !(MIN_TRAVERSAL_DEPTH..=MAX_TRAVERSAL_DEPTH).contains(&depth) {
            return Err(ValidationError::from(ValidationErrorKind::InvalidDepth(depth)).into());
        }

        let raw = self
            .repository
            .find_related_entities(name, relationship_type.clone(), direction, depth)
            .await?;

        let mapped = raw.into_iter().map(from_default_entity::<P>).collect();

        Ok(mapped)
    }

    /// Find related entities using the default HashMap property type
    #[instrument(skip(self), fields(name, depth))]
    pub async fn find_related_entities(
        &self,
        name: &str,
        relationship_type: Option<String>,
        direction: Option<RelationshipDirection>,
        depth: u32,
    ) -> MemoryResult<Vec<MemoryEntity>, R::Error> {
        self.find_related_entities_typed::<HashMap<String, MemoryValue>>(
            name,
            relationship_type,
            direction,
            depth,
        )
        .await
    }

    /// Find entities matching the given labels
    #[instrument(skip(self, labels), fields(labels_count = labels.len()))]
    pub async fn find_entities_by_labels_typed<P>(
        &self,
        labels: &[String],
        match_mode: LabelMatchMode,
        required_label: Option<String>,
    ) -> MemoryResult<Vec<MemoryEntity<P>>, R::Error>
    where
        P: JsonSchema
            + From<HashMap<String, MemoryValue>>
            + Into<HashMap<String, MemoryValue>>
            + Clone
            + std::fmt::Debug
            + Default,
    {
        let effective_required = required_label.or_else(|| self.config.default_label.clone());
        let raw = self
            .repository
            .find_entities_by_labels(labels, match_mode, effective_required)
            .await?;

        let mapped = raw.into_iter().map(from_default_entity::<P>).collect();

        Ok(mapped)
    }

    /// Find entities by labels using the default HashMap property type
    #[instrument(skip(self, labels), fields(labels_count = labels.len()))]
    pub async fn find_entities_by_labels(
        &self,
        labels: &[String],
        match_mode: LabelMatchMode,
        required_label: Option<String>,
    ) -> MemoryResult<Vec<MemoryEntity>, R::Error> {
        self.find_entities_by_labels_typed::<HashMap<String, MemoryValue>>(
            labels,
            match_mode,
            required_label,
        )
        .await
    }

    /// Update aspects of an entity
    #[instrument(skip(self, update), fields(name))]
    pub async fn update_entity(
        &self,
        name: &str,
        update: &EntityUpdate,
    ) -> MemoryResult<(), R::Error> {
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        if let Some(obs) = &update.observations {
            let count =
                obs.add.is_some() as u8 + obs.remove.is_some() as u8 + obs.set.is_some() as u8;
            if count > 1 {
                return Err(
                    ValidationError::from(ValidationErrorKind::ConflictingOperations(
                        "observations",
                    ))
                    .into(),
                );
            }
        }
        if let Some(props) = &update.properties {
            let count = props.add.is_some() as u8
                + props.remove.is_some() as u8
                + props.set.is_some() as u8;
            if count > 1 {
                return Err(
                    ValidationError::from(ValidationErrorKind::ConflictingOperations("properties"))
                        .into(),
                );
            }
        }
        if let Some(labels) = &update.labels {
            let count = labels.add.is_some() as u8 + labels.remove.is_some() as u8;
            if count > 1 {
                return Err(
                    ValidationError::from(ValidationErrorKind::ConflictingOperations("labels"))
                        .into(),
                );
            }
        }

        self.repository.update_entity(name, update).await
    }

    /// Update a relationship's properties
    #[instrument(skip(self, update), fields(from, to, name))]
    pub async fn update_relationship(
        &self,
        from: &str,
        to: &str,
        name: &str,
        update: &RelationshipUpdate,
    ) -> MemoryResult<(), R::Error> {
        if from.is_empty() || to.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }
        if let Some(props) = &update.properties {
            let count = props.add.is_some() as u8
                + props.remove.is_some() as u8
                + props.set.is_some() as u8;
            if count > 1 {
                return Err(
                    ValidationError::from(ValidationErrorKind::ConflictingOperations("properties"))
                        .into(),
                );
            }
        }

        self.repository
            .update_relationship(from, to, name, update)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockMemoryRepository;
    use crate::ValidationErrorKind;
    use mockall::predicate::*;
    use std::collections::{HashMap, HashSet};

    #[tokio::test]
    async fn test_default_label_added() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && e[0].labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_label: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: false,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            ..Default::default()
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_no_default_label() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && !e[0].labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_label: None,
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: false,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            ..Default::default()
        };

        let result = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_empty_labels_adds_default_label() {
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
                default_label: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: false,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            ..Default::default()
        };

        let errors = service
            .create_entities(std::slice::from_ref(&entity))
            .await
            .unwrap();
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_empty_labels_without_default_label_fails() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities().never();

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_label: None,
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: false,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            ..Default::default()
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
    async fn test_default_label_allowed_with_label_validation() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && e[0].labels == ["Custom".to_string()])
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_label: Some("Custom".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            ..Default::default()
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
                default_label: None,
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Unknown".to_string()],
            ..Default::default()
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
                default_label: None,
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "relates_to".to_string(),
            properties: HashMap::default(),
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
                default_label: None,
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "custom_rel".to_string(),
            properties: HashMap::default(),
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
                default_label: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: false,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            ..Default::default()
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
                default_label: None,
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "relates_to".to_string(),
            properties: HashMap::default(),
        };

        let result = service
            .create_relationships(std::slice::from_ref(&rel))
            .await;
        assert!(matches!(result, Err(crate::MemoryError::QueryError { .. })));
    }

    #[tokio::test]
    async fn test_find_entity_with_relationships() {
        let relationship = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "relates_to".to_string(),
            properties: HashMap::default(),
        };
        let entity = MemoryEntity {
            name: "a".to_string(),
            labels: vec!["Test".to_string()],
            relationships: vec![relationship.clone()],
            ..Default::default()
        };

        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("a"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());

        let found = service.find_entity_by_name("a").await.unwrap().unwrap();
        assert_eq!(found.relationships, vec![relationship]);
    }

    #[tokio::test]
    async fn test_find_related_entities_validation() {
        let mock = MockMemoryRepository::new();
        let service = MemoryService::new(mock, MemoryConfig::default());

        let err = service
            .find_related_entities("", None, None, 1)
            .await
            .unwrap_err();
        assert!(matches!(err, crate::MemoryError::ValidationError(_)));

        let err = service
            .find_related_entities("a", None, None, 6)
            .await
            .unwrap_err();
        assert!(matches!(err, crate::MemoryError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_find_related_entities_calls_repo() {
        let mut mock = MockMemoryRepository::new();
        let expected: Vec<MemoryEntity> = vec![MemoryEntity {
            name: "b".to_string(),
            ..Default::default()
        }];
        mock.expect_find_related_entities()
            .with(
                eq("a"),
                eq(Some("relates_to".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(2u32),
            )
            .return_once(move |_, _, _, _| Ok(expected.clone()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let result = service
            .find_related_entities(
                "a",
                Some("relates_to".to_string()),
                Some(RelationshipDirection::Outgoing),
                2,
            )
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "b");
    }

    #[tokio::test]
    async fn test_find_entities_by_labels_default_required() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entities_by_labels()
            .withf(|labels, mode, req| {
                labels == ["Example".to_string()]
                    && *mode == LabelMatchMode::Any
                    && req.as_deref() == Some("Default")
            })
            .return_once(|_, _, _| Ok(Vec::new()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_label: Some("Default".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let _ = service
            .find_entities_by_labels(&["Example".to_string()], LabelMatchMode::Any, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_find_entities_by_labels_required_override() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entities_by_labels()
            .withf(|labels, mode, req| {
                labels.is_empty()
                    && *mode == LabelMatchMode::All
                    && req.as_deref() == Some("Explicit")
            })
            .return_once(|_, _, _| Ok(Vec::new()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_label: Some("Default".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::default(),
                default_labels: true,
                additional_labels: HashSet::default(),
                default_project: None,
            },
        );

        let _ = service
            .find_entities_by_labels(&[], LabelMatchMode::All, Some("Explicit".to_string()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_update_entity_conflict() {
        let mock = MockMemoryRepository::new();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let update = EntityUpdate {
            observations: Some(ObservationsUpdate {
                add: Some(vec!["a".to_string()]),
                remove: Some(vec!["b".to_string()]),
                set: None,
            }),
            properties: None,
            labels: None,
        };
        let err = service.update_entity("e", &update).await.unwrap_err();
        assert!(matches!(err, crate::MemoryError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_update_entity_calls_repo() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity()
            .withf(|name, _| name == "e")
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let update = EntityUpdate::default();
        let result = service.update_entity("e", &update).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_relationship_calls_repo() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_relationship()
            .withf(|from, to, name, _| from == "a" && to == "b" && name == "rel")
            .returning(|_, _, _, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let update = RelationshipUpdate::default();
        let result = service.update_relationship("a", "b", "rel", &update).await;
        assert!(result.is_ok());
    }

    mod prop_tests {
        use super::*;
        use crate::test_helpers::{prop_random_entity, prop_random_relationship};
        use arbitrary::Arbitrary;
        use mm_utils::prop::{NonEmptyName, async_arbtest};

        #[test]
        fn prop_create_entities_success() {
            async_arbtest(|rt, u| {
                let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
                let idx = u.choose_index(DEFAULT_LABELS.len())?;
                let label = DEFAULT_LABELS[idx].to_string();
                let mut entity = prop_random_entity(u, Some(label))?;
                entity.name.clone_from(&name);

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
                let idx = u.choose_index(DEFAULT_LABELS.len())?;
                let label = DEFAULT_LABELS[idx].to_string();
                let mut entity = prop_random_entity(u, Some(label))?;
                entity.name = String::default();

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
                    || label == MemoryConfig::default().default_label.clone().unwrap()
                {
                    label.push_str("_x");
                }
                let mut entity = match prop_random_entity(u, Some(label.clone())) {
                    Ok(e) => e,
                    Err(_) => return Ok(()),
                };
                entity.name.clone_from(&name);

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
                let idx = match u.choose_index(DEFAULT_RELATIONSHIPS.len()) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let name = DEFAULT_RELATIONSHIPS[idx].to_string();
                let mut rel = match prop_random_relationship(u, Some(name.clone())) {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };
                rel.from.clone_from(&from);
                rel.to.clone_from(&to);

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
                let idx = match u.choose_index(DEFAULT_RELATIONSHIPS.len()) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let mut name = DEFAULT_RELATIONSHIPS[idx].to_string();
                name.push('A');
                let mut rel = match prop_random_relationship(u, Some(name.clone())) {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };
                rel.from = from;
                rel.to = to;

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
                let idx = match u.choose_index(DEFAULT_RELATIONSHIPS.len()) {
                    Ok(v) => v,
                    Err(_) => return Ok(()),
                };
                let mut name = DEFAULT_RELATIONSHIPS[idx].to_string();
                name.push_str("_extra");
                let mut rel = match prop_random_relationship(u, Some(name.clone())) {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };
                rel.from.clone_from(&from);
                rel.to.clone_from(&to);

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
