use crate::{
    fsrs::{next_review, Rating},
    types::{ConceptState, FSRSCard, LearnerModel, ObservationRecord},
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use storage_provider::{CrdtEngine, StorageError, StorageProvider};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Model not found for learner: {0}")]
    NotFound(String),
    #[error("Concept not found for learner {learner_id}: {concept_id}")]
    ConceptNotFound {
        learner_id: String,
        concept_id: String,
    },
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
                let mut model: LearnerModel = serde_json::from_value(json)?;
                let migrated = normalize_model(&mut model);
                if migrated {
                    self.archive_original(learner_id, &b).await?;
                    self.write_fresh_normalized(&model).await?;
                }
                Ok(model)
            }
        }
    }

    /// Persist a `LearnerModel` to storage via CRDT, merging with any existing document.
    pub async fn save(&self, model: &LearnerModel) -> Result<(), StoreError> {
        let mut model = model.clone();
        let migrated = normalize_model(&mut model);
        let key = Self::key(&model.learner_id);
        if migrated {
            if let Some(original) = self.storage.read(&key).await? {
                self.archive_original(&model.learner_id, &original).await?;
            }
            self.write_fresh_normalized(&model).await
        } else {
            self.save_normalized(&model).await
        }
    }

    async fn save_normalized(&self, model: &LearnerModel) -> Result<(), StoreError> {
        let key = Self::key(&model.learner_id);
        let patch = serde_json::to_value(model)?;
        let existing = self.storage.read(&key).await?;
        let doc = existing.unwrap_or_else(|| self.crdt.new_doc());
        let (new_doc, _delta) = self.crdt.apply_json(&doc, patch)?;
        self.storage.write(&key, new_doc).await?;
        Ok(())
    }

    async fn write_fresh_normalized(&self, model: &LearnerModel) -> Result<(), StoreError> {
        let key = Self::key(&model.learner_id);
        let patch = serde_json::to_value(model)?;
        let (doc, _) = self.crdt.apply_json(&self.crdt.new_doc(), patch)?;
        self.storage.write(&key, doc).await?;
        Ok(())
    }

    async fn archive_original(&self, learner_id: &str, original: &[u8]) -> Result<(), StoreError> {
        let digest = blake3::hash(original).to_hex();
        let backup = format!("learner/{learner_id}/migrations/pre-1.1-{digest}.loro");
        if self.storage.read(&backup).await?.is_none() {
            self.storage.write(&backup, original.to_vec()).await?;
        }
        Ok(())
    }

    /// Merge a remote CRDT delta into the local model.
    ///
    /// Used for multi-device sync. The caller provides raw CRDT delta bytes
    /// from a Loro update or snapshot.
    pub async fn merge_delta(
        &self,
        learner_id: &str,
        remote_delta: &[u8],
    ) -> Result<(), StoreError> {
        let key = Self::key(learner_id);
        let local = self.storage.read(&key).await?;
        let local = local.unwrap_or_else(|| self.crdt.new_doc());
        let merged = self.crdt.merge(&local, remote_delta)?;
        let json = self.crdt.to_json(&merged)?;
        let mut model: LearnerModel = serde_json::from_value(json)?;
        normalize_model(&mut model);
        let patch = serde_json::to_value(&model)?;
        let (normalized, _) = self.crdt.apply_json(&merged, patch)?;
        self.storage.write(&key, normalized).await?;
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

        let concept =
            model
                .concepts
                .get_mut(concept_id)
                .ok_or_else(|| StoreError::ConceptNotFound {
                    learner_id: learner_id.to_string(),
                    concept_id: concept_id.to_string(),
                })?;
        let observation_id = Uuid::new_v4().to_string();
        concept.observations.insert(
            observation_id.clone(),
            ObservationRecord {
                observation_id,
                timestamp: now,
                score,
                source_skill: source_skill.to_string(),
                vector_clock: HashMap::new(),
                rating: None,
            },
        );
        fold_concept(concept);

        model.updated_at = now;
        self.save(&model).await
    }

    /// Record a scored retention review and advance the concept's FSRS card.
    pub async fn review_concept(
        &self,
        learner_id: &str,
        concept_id: &str,
        score: f64,
        rating: Rating,
        source_skill: &str,
        reviewed_at: DateTime<Utc>,
    ) -> Result<FSRSCard, StoreError> {
        let mut model = self.load(learner_id).await?;
        let concept =
            model
                .concepts
                .get_mut(concept_id)
                .ok_or_else(|| StoreError::ConceptNotFound {
                    learner_id: learner_id.to_string(),
                    concept_id: concept_id.to_string(),
                })?;

        let observation_id = Uuid::new_v4().to_string();
        concept.observations.insert(
            observation_id.clone(),
            ObservationRecord {
                observation_id,
                timestamp: reviewed_at,
                score,
                source_skill: source_skill.to_string(),
                vector_clock: HashMap::new(),
                rating: Some(rating),
            },
        );
        fold_concept(concept);
        let updated_card = concept.fsrs_card.clone();
        model.updated_at = reviewed_at;
        self.save(&model).await?;
        Ok(updated_card)
    }
}

/// Recompute all derived concept state from immutable, uniquely keyed evidence.
/// Ordering is canonical, so local writes and remote imports converge.
pub fn fold_concept(concept: &mut ConceptState) {
    let prior = *concept.mastery_prior.get_or_insert(concept.mastery);
    let fsrs_prior = concept
        .fsrs_prior
        .get_or_insert_with(|| concept.fsrs_card.clone())
        .clone();
    let mut evidence = concept.observations.values().collect::<Vec<_>>();
    evidence.sort_by(|left, right| {
        (left.timestamp, left.observation_id.as_str())
            .cmp(&(right.timestamp, right.observation_id.as_str()))
    });

    let mut mastery = prior;
    let mut card = fsrs_prior.clone();
    let mut earliest_review_due = None;
    for (index, observation) in evidence.iter().enumerate() {
        if index + 1 >= 5 {
            mastery = (mastery + 0.3 * (observation.score - mastery)).clamp(0.0, 1.0);
        }
        if let Some(rating) = observation.rating {
            card = next_review(&card, rating, observation.timestamp);
            earliest_review_due =
                Some(earliest_review_due.map_or(card.due, |due: DateTime<Utc>| due.min(card.due)));
        }
    }
    card.due = earliest_review_due.unwrap_or(fsrs_prior.due);
    card.reps = card.reps.max(fsrs_prior.reps);
    card.lapses = card.lapses.max(fsrs_prior.lapses);
    concept.mastery = mastery;
    concept.fsrs_card = card;
}

fn normalize_model(model: &mut LearnerModel) -> bool {
    let legacy = model.schema_version != "1.1.0"
        || model
            .concepts
            .values()
            .any(|concept| concept.mastery_prior.is_none() || concept.fsrs_prior.is_none());
    for concept in model.concepts.values_mut() {
        for (id, observation) in &mut concept.observations {
            if observation.observation_id.is_empty() {
                observation.observation_id = id.clone();
            }
        }
        fold_concept(concept);
    }
    model.schema_version = "1.1.0".into();
    legacy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CardState, ConceptState};

    // Minimal in-memory implementations for unit tests.

    use async_trait::async_trait;
    use std::collections::{BTreeMap, HashMap as StdMap};
    use std::sync::Mutex;
    use storage_provider::{CrdtEngine, StorageError, StorageProvider};

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
            Ok(guard
                .keys()
                .filter(|k| k.starts_with(prefix))
                .cloned()
                .collect())
        }
        fn backend_name(&self) -> &'static str {
            "mem"
        }
    }

    /// CRDT engine stub: stores raw JSON bytes; no real CRDT merging.
    struct JsonEngine;

    impl CrdtEngine for JsonEngine {
        fn new_doc(&self) -> Vec<u8> {
            b"{}".to_vec()
        }
        fn merge(&self, _local: &[u8], remote: &[u8]) -> StorageResult<Vec<u8>> {
            Ok(remote.to_vec())
        }
        fn apply_json(
            &self,
            _doc: &[u8],
            patch: serde_json::Value,
        ) -> StorageResult<(Vec<u8>, Vec<u8>)> {
            let bytes = serde_json::to_vec(&patch)?;
            Ok((bytes.clone(), bytes))
        }
        fn to_json(&self, doc: &[u8]) -> StorageResult<serde_json::Value> {
            Ok(serde_json::from_slice(doc)?)
        }
        fn engine_name(&self) -> &'static str {
            "json-stub"
        }
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
                mastery_prior: Some(0.4),
                observations: BTreeMap::new(),
                fsrs_card: FSRSCard {
                    stability: 1.0,
                    difficulty: 5.0,
                    due: now,
                    state: CardState::New,
                    reps: 0,
                    lapses: 0,
                    last_review: None,
                },
                fsrs_prior: None,
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
        assert_eq!(loaded.schema_version, "1.1.0");
        assert!(loaded.concepts.contains_key("closures"));
    }

    #[tokio::test]
    async fn load_returns_not_found_for_missing_learner() {
        let store = make_store();
        let err = store.load("did:plc:nobody").await.unwrap_err();
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn legacy_snapshot_migrates_without_deleting_original() {
        let store = make_store();
        let learner_id = "did:plc:legacy";
        let now = Utc::now();
        let original = serde_json::to_vec(&serde_json::json!({
            "schema_version": "1.0.0",
            "learner_id": learner_id,
            "created_at": now,
            "updated_at": now,
            "concepts": {
                "closures": {
                    "concept_id": "closures",
                    "label": "Closures",
                    "mastery": 0.4,
                    "observations": [{
                        "timestamp": now,
                        "score": 0.9,
                        "source_skill": "learn-grade",
                        "vector_clock": {}
                    }],
                    "fsrs_card": {
                        "stability": 1.0,
                        "difficulty": 5.0,
                        "due": now,
                        "state": "New",
                        "reps": 0,
                        "lapses": 0,
                        "last_review": null
                    }
                }
            },
            "gaps": {},
            "sessions": []
        }))
        .unwrap();
        store
            .storage
            .write(
                &LearnerModelStore::<MemStorage, JsonEngine>::key(learner_id),
                original.clone(),
            )
            .await
            .unwrap();

        let migrated = store.load(learner_id).await.unwrap();

        assert_eq!(migrated.schema_version, "1.1.0");
        assert_eq!(migrated.concepts["closures"].observations.len(), 1);
        let backups = store
            .storage
            .list_keys(&format!("learner/{learner_id}/migrations/"))
            .await
            .unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            store.storage.read(&backups[0]).await.unwrap(),
            Some(original)
        );
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

    #[tokio::test]
    async fn missing_concept_returns_error() {
        let store = make_store();
        store
            .save(&seed_model("did:plc:missing-concept"))
            .await
            .expect("save");
        let err = store
            .add_observation("did:plc:missing-concept", "not-present", 0.8, "learn-grade")
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::ConceptNotFound { .. }));
    }

    #[tokio::test]
    async fn review_records_observation_and_advances_fsrs() {
        let store = make_store();
        let learner_id = "did:plc:reviewer";
        store.save(&seed_model(learner_id)).await.expect("save");
        let reviewed_at = Utc::now();

        let card = store
            .review_concept(
                learner_id,
                "closures",
                0.85,
                Rating::Easy,
                "learn-retain",
                reviewed_at,
            )
            .await
            .expect("review");

        assert_eq!(card.reps, 1);
        assert_eq!(card.last_review, Some(reviewed_at));
        assert!(card.due > reviewed_at);
        let updated = store.load(learner_id).await.expect("load");
        assert_eq!(updated.concepts["closures"].observations.len(), 1);
        assert_eq!(updated.concepts["closures"].fsrs_card.reps, 1);
    }

    #[test]
    fn deterministic_evidence_fold_is_order_independent() {
        let at = Utc::now();
        let mut left = seed_model("did:plc:left")
            .concepts
            .remove("closures")
            .unwrap();
        left.observations = [
            (
                "evidence-b".to_string(),
                ObservationRecord {
                    observation_id: "evidence-b".into(),
                    timestamp: at + chrono::Duration::seconds(1),
                    score: 0.2,
                    source_skill: "learn-grade".into(),
                    vector_clock: HashMap::new(),
                    rating: None,
                },
            ),
            (
                "evidence-a".to_string(),
                ObservationRecord {
                    observation_id: "evidence-a".into(),
                    timestamp: at,
                    score: 1.0,
                    source_skill: "learn-grade".into(),
                    vector_clock: HashMap::new(),
                    rating: None,
                },
            ),
        ]
        .into_iter()
        .collect();
        let mut right = left.clone();
        right.observations = left
            .observations
            .iter()
            .rev()
            .map(|(id, evidence)| (id.clone(), evidence.clone()))
            .collect();

        fold_concept(&mut left);
        fold_concept(&mut right);

        assert_eq!(left.mastery, right.mastery);
        assert_eq!(left.fsrs_card.reps, right.fsrs_card.reps);
        assert_eq!(left.fsrs_card.due, right.fsrs_card.due);
    }
}
