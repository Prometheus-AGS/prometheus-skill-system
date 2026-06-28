use crate::types::{LearnerModel, ObservationRecord};
use storage_provider::{CrdtEngine, StorageError, StorageProvider};
use chrono::Utc;
use serde_json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Model not found for learner: {0}")]
    NotFound(String),
}

/// Typed facade over `StorageProvider` + `CrdtEngine` for learner model documents.
pub struct LearnerModelStore<S: StorageProvider, C: CrdtEngine> {
    storage: S,
    crdt: C,
}

impl<S: StorageProvider, C: CrdtEngine> LearnerModelStore<S, C> {
    pub fn new(storage: S, crdt: C) -> Self {
        Self { storage, crdt }
    }

    /// Canonical storage key for a learner's model document.
    fn key(learner_id: &str) -> String {
        format!("learner/{}/model.crdt", learner_id)
    }

    /// Load a `LearnerModel` from storage. Returns `StoreError::NotFound` if absent.
    pub async fn load(&self, learner_id: &str) -> Result<LearnerModel, StoreError> {
        let key = Self::key(learner_id);
        let bytes = self.storage.read(&key).await?;
        match bytes {
            None => Err(StoreError::NotFound(learner_id.to_string())),
            Some(b) => {
                let json = self.crdt.to_json(&b)?;
                Ok(serde_json::from_value(json)?)
            }
        }
    }

    /// Persist a `LearnerModel` to storage via CRDT, merging with any existing document.
    pub async fn save(&self, model: &LearnerModel) -> Result<(), StoreError> {
        let key = Self::key(&model.learner_id);
        let patch = serde_json::to_value(model)?;
        let existing = self.storage.read(&key).await?;
        let doc = existing.unwrap_or_else(|| self.crdt.new_doc());
        let (new_doc, _delta) = self.crdt.apply_json(&doc, patch)?;
        self.storage.write(&key, new_doc).await?;
        Ok(())
    }

    /// Merge a remote CRDT delta into the local model.
    ///
    /// Used for multi-device sync. The caller provides raw CRDT delta bytes
    /// (e.g. from an automerge sync message).
    pub async fn merge_delta(
        &self,
        learner_id: &str,
        remote_delta: &[u8],
    ) -> Result<(), StoreError> {
        let key = Self::key(learner_id);
        let local = self.storage.read(&key).await?;
        let local = local.unwrap_or_else(|| self.crdt.new_doc());
        let merged = self.crdt.merge(&local, remote_delta)?;
        self.storage.write(&key, merged).await?;
        Ok(())
    }

    /// Record a new observation for a concept and update mastery via the PFA rule.
    ///
    /// PFA update rule: `mastery_new = mastery_old + 0.3 * (score - mastery_old)`
    ///
    /// The rule is only applied when the concept has **≥5 observations** after appending,
    /// matching the schema constraint: "LLM-seeded prior before that."
    pub async fn add_observation(
        &self,
        learner_id: &str,
        concept_id: &str,
        score: f64,
        source_skill: &str,
    ) -> Result<(), StoreError> {
        let mut model = self.load(learner_id).await?;
        let now = Utc::now();

        if let Some(concept) = model.concepts.get_mut(concept_id) {
            concept.observations.push(ObservationRecord {
                timestamp: now,
                score,
                source_skill: source_skill.to_string(),
                vector_clock: HashMap::new(),
            });

            // Apply PFA rule only once concept has accumulated enough signal.
            if concept.observations.len() >= 5 {
                let mastery_old = concept.mastery;
                concept.mastery = (mastery_old + 0.3 * (score - mastery_old)).clamp(0.0, 1.0);
            }
        }

        model.updated_at = now;
        self.save(&model).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal in-memory implementations for unit tests.

    use storage_provider::{CrdtEngine, StorageProvider, StorageError};
    use async_trait::async_trait;
    use std::collections::HashMap as StdMap;
    use std::sync::Mutex;

    type StorageResult<T> = std::result::Result<T, StorageError>;

    struct MemStorage(Mutex<StdMap<String, Vec<u8>>>);

    impl MemStorage {
        fn new() -> Self {
            Self(Mutex::new(StdMap::new()))
        }
    }

    #[async_trait]
    impl StorageProvider for MemStorage {
        async fn read(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }
        async fn write(&self, key: &str, value: Vec<u8>) -> StorageResult<()> {
            self.0.lock().unwrap().insert(key.to_string(), value);
            Ok(())
        }
        async fn delete(&self, key: &str) -> StorageResult<()> {
            self.0.lock().unwrap().remove(key);
            Ok(())
        }
        async fn list_keys(&self, prefix: &str) -> StorageResult<Vec<String>> {
            let guard = self.0.lock().unwrap();
            Ok(guard.keys().filter(|k| k.starts_with(prefix)).cloned().collect())
        }
        fn backend_name(&self) -> &'static str { "mem" }
    }

    /// CRDT engine stub: stores raw JSON bytes; no real CRDT merging.
    struct JsonEngine;

    impl CrdtEngine for JsonEngine {
        fn new_doc(&self) -> Vec<u8> { b"{}".to_vec() }
        fn merge(&self, _local: &[u8], remote: &[u8]) -> StorageResult<Vec<u8>> {
            Ok(remote.to_vec())
        }
        fn apply_json(&self, _doc: &[u8], patch: serde_json::Value) -> StorageResult<(Vec<u8>, Vec<u8>)> {
            let bytes = serde_json::to_vec(&patch)?;
            Ok((bytes.clone(), bytes))
        }
        fn to_json(&self, doc: &[u8]) -> StorageResult<serde_json::Value> {
            Ok(serde_json::from_slice(doc)?)
        }
        fn engine_name(&self) -> &'static str { "json-stub" }
    }

    fn make_store() -> LearnerModelStore<MemStorage, JsonEngine> {
        LearnerModelStore::new(MemStorage::new(), JsonEngine)
    }

    fn seed_model(learner_id: &str) -> LearnerModel {
        let now = Utc::now();
        let mut concepts = StdMap::new();
        concepts.insert(
            "closures".to_string(),
            ConceptState {
                concept_id: "closures".to_string(),
                label: "Closures".to_string(),
                mastery: 0.4,
                observations: vec![],
                fsrs_card: FSRSCard {
                    stability: 1.0,
                    difficulty: 5.0,
                    due: now,
                    state: CardState::New,
                    reps: 0,
                    lapses: 0,
                    last_review: None,
                },
            },
        );
        LearnerModel {
            schema_version: "1.0.0".to_string(),
            learner_id: learner_id.to_string(),
            created_at: now,
            updated_at: now,
            concepts,
            gaps: StdMap::new(),
            sessions: vec![],
        }
    }

    #[tokio::test]
    async fn save_and_load_roundtrip() {
        let store = make_store();
        let model = seed_model("did:plc:abc");
        store.save(&model).await.expect("save");
        let loaded = store.load("did:plc:abc").await.expect("load");
        assert_eq!(loaded.learner_id, "did:plc:abc");
        assert_eq!(loaded.schema_version, "1.0.0");
        assert!(loaded.concepts.contains_key("closures"));
    }

    #[tokio::test]
    async fn load_returns_not_found_for_missing_learner() {
        let store = make_store();
        let err = store.load("did:plc:nobody").await.unwrap_err();
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn add_observation_below_threshold_does_not_update_mastery() {
        let store = make_store();
        let model = seed_model("did:plc:learner1");
        let initial_mastery = model.concepts["closures"].mastery;
        store.save(&model).await.expect("save");

        // Add fewer than 5 observations
        for _ in 0..4 {
            store
                .add_observation("did:plc:learner1", "closures", 0.9, "learn-grade")
                .await
                .expect("observe");
        }

        let updated = store.load("did:plc:learner1").await.expect("load");
        // Mastery unchanged — PFA rule not yet triggered
        assert_eq!(updated.concepts["closures"].mastery, initial_mastery);
        assert_eq!(updated.concepts["closures"].observations.len(), 4);
    }

    #[tokio::test]
    async fn add_observation_at_threshold_applies_pfa_rule() {
        let store = make_store();
        let model = seed_model("did:plc:learner2");
        store.save(&model).await.expect("save");

        // Add 5 observations with score 1.0
        for _ in 0..5 {
            store
                .add_observation("did:plc:learner2", "closures", 1.0, "learn-grade")
                .await
                .expect("observe");
        }

        let updated = store.load("did:plc:learner2").await.expect("load");
        let concept = &updated.concepts["closures"];
        // After 5 observations all at 1.0, mastery should have increased above 0.4
        assert!(concept.mastery > 0.4, "mastery={}", concept.mastery);
        assert!(concept.mastery <= 1.0);
        assert_eq!(concept.observations.len(), 5);
    }

    #[tokio::test]
    async fn mastery_clamps_to_1_0() {
        let store = make_store();
        let mut model = seed_model("did:plc:learner3");
        // Start with very high mastery
        model.concepts.get_mut("closures").unwrap().mastery = 0.99;
        store.save(&model).await.expect("save");

        for _ in 0..5 {
            store
                .add_observation("did:plc:learner3", "closures", 1.0, "learn-grade")
                .await
                .expect("observe");
        }

        let updated = store.load("did:plc:learner3").await.expect("load");
        assert!(updated.concepts["closures"].mastery <= 1.0);
    }
}
