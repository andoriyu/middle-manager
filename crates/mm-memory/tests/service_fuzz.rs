use arbitrary::Arbitrary;
use arbtest::arbtest;
use mm_memory::{
    MemoryConfig, MemoryEntity, MemoryRelationship, MemoryService, MockMemoryRepository,
    ValidationErrorKind,
};
use mm_utils::is_snake_case;
use std::collections::HashSet;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn fuzz_create_entities() {
    arbtest(|u| {
        let count = u.int_in_range::<usize>(0..=5)?;
        let mut entities = Vec::new();
        for _ in 0..count {
            entities.push(MemoryEntity::arbitrary(u)?);
        }

        let valid: Vec<_> = entities
            .iter()
            .cloned()
            .filter(|e| !e.labels.is_empty())
            .collect();

        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(move |e| e.iter().all(|en| !en.labels.is_empty()) && e.len() == valid.len())
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: false,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );

        let errors = runtime()
            .block_on(async { service.create_entities(&entities).await })
            .unwrap();

        for entity in &entities {
            let err = errors.iter().find(|(n, _)| n == &entity.name);
            if entity.labels.is_empty() {
                let (_, ve) = err.expect("missing error for invalid entity");
                assert!(ve.0.contains(&ValidationErrorKind::NoLabels(entity.name.clone())));
            } else {
                assert!(err.is_none());
            }
        }
        Ok(())
    });
}

#[test]
fn fuzz_create_relationships() {
    arbtest(|u| {
        let count = u.int_in_range::<usize>(0..=5)?;
        let mut rels = Vec::new();
        for _ in 0..count {
            rels.push(MemoryRelationship::arbitrary(u)?);
        }

        let valid: Vec<_> = rels
            .iter()
            .cloned()
            .filter(|r| is_snake_case(&r.name))
            .collect();

        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships()
            .withf(move |rs| rs.iter().all(|r| is_snake_case(&r.name)) && rs.len() == valid.len())
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: false,
                additional_relationships: HashSet::new(),
                default_labels: false,
                additional_labels: HashSet::new(),
            },
        );

        let errors = runtime()
            .block_on(async { service.create_relationships(&rels).await })
            .unwrap();

        for rel in &rels {
            let err = errors.iter().find(|(n, _)| n == &rel.name);
            if !is_snake_case(&rel.name) {
                let (_, ve) = err.expect("missing error for invalid relationship");
                assert!(
                    ve.0.contains(&ValidationErrorKind::InvalidRelationshipFormat(
                        rel.name.clone()
                    ))
                );
            } else {
                assert!(err.is_none());
            }
        }
        Ok(())
    });
}
