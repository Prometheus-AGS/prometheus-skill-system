use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::fsrs::Rating;

/// Top-level learner model document. Stored as a CRDT document keyed by learner_id.
///
/// `concepts` and `gaps` are maps (concept_id → ConceptState, gap_id → GapRecord)
/// matching the JSON Schema definition in docs/learn/schemas/learner-model.schema.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerModel {
    pub schema_version: String,
    pub learner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Map of concept_id (kebab-case surreal-memory entity name) → ConceptState.
    pub concepts: HashMap<String, ConceptState>,
    /// Map of gap_id (UUID v4) → GapRecord.
    pub gaps: HashMap<String, GapRecord>,
    pub sessions: Vec<SessionRecord>,
}

/// Per-concept mastery and scheduling state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptState {
    pub concept_id: String,
    pub label: String,
    /// Current mastery estimate [0,1].
    /// Updated by PFA rule after ≥5 observations; LLM-seeded prior before that.
    pub mastery: f64,
    /// Immutable mastery seed. Legacy snapshots populate this from `mastery`
    /// during migration before any new evidence is folded.
    #[serde(default)]
    pub mastery_prior: Option<f64>,
    /// Immutable evidence keyed by globally unique observation ID.
    #[serde(default, deserialize_with = "deserialize_observations")]
    pub observations: BTreeMap<String, ObservationRecord>,
    /// FSRS-6 scheduling card for spaced-repetition scheduling.
    pub fsrs_card: FSRSCard,
    /// Immutable scheduling seed used to derive `fsrs_card` after every merge.
    #[serde(default)]
    pub fsrs_prior: Option<FSRSCard>,
}

/// A single scored observation of learner performance on a concept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationRecord {
    #[serde(default)]
    pub observation_id: String,
    pub timestamp: DateTime<Utc>,
    /// Performance score [0,1].
    pub score: f64,
    /// Which learn-* skill produced this observation.
    pub source_skill: String,
    /// Vector clock for CRDT LWW merge. Key = device DID, value = logical timestamp.
    pub vector_clock: HashMap<String, u64>,
    /// Present only when this evidence also represents a retention review.
    #[serde(default)]
    pub rating: Option<Rating>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ObservationCollection {
    Keyed(BTreeMap<String, ObservationRecord>),
    Legacy(Vec<ObservationRecord>),
}

fn deserialize_observations<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, ObservationRecord>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let collection = ObservationCollection::deserialize(deserializer)?;
    let mut keyed = BTreeMap::new();
    match collection {
        ObservationCollection::Keyed(records) => keyed = records,
        ObservationCollection::Legacy(records) => {
            for (index, mut record) in records.into_iter().enumerate() {
                if record.observation_id.is_empty() {
                    let canonical = serde_jcs::to_vec(&serde_json::json!({
                        "timestamp": record.timestamp,
                        "score": record.score,
                        "sourceSkill": record.source_skill,
                        "vectorClock": record.vector_clock,
                        "legacyIndex": index,
                    }))
                    .map_err(serde::de::Error::custom)?;
                    record.observation_id = format!("legacy-{}", blake3::hash(&canonical).to_hex());
                }
                keyed.entry(record.observation_id.clone()).or_insert(record);
            }
        }
    }
    Ok(keyed)
}

/// FSRS-6 scheduling card state, persisted per concept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSRSCard {
    /// FSRS stability (days). CRDT merge: take max(local, remote).
    pub stability: f64,
    /// FSRS difficulty [1,10]. CRDT merge: LWW (latest timestamp wins).
    pub difficulty: f64,
    /// Next scheduled review date. CRDT merge: take min(local, remote) — prefer earlier review.
    pub due: DateTime<Utc>,
    /// FSRS card state. CRDT merge: LWW (latest timestamp wins).
    pub state: CardState,
    /// Total review repetitions. CRDT merge: take max(local, remote).
    pub reps: u32,
    /// Total lapses (forgot). CRDT merge: take max(local, remote).
    pub lapses: u32,
    /// Timestamp of last review. Used for LWW merge decisions.
    pub last_review: Option<DateTime<Utc>>,
}

/// FSRS card states matching the JSON Schema enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

/// Gap severity as defined in the JSON Schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    Minor,
    Major,
    Misconception,
}

/// A knowledge gap detected by learn-grade. Append-only; never deleted by CRDT merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRecord {
    pub gap_id: String,
    pub concept_id: String,
    pub description: String,
    pub severity: Option<GapSeverity>,
    pub detected_at: DateTime<Utc>,
    /// Set when a subsequent learn-grade pass finds this gap closed. Null if still open.
    pub resolved_at: Option<DateTime<Utc>>,
    /// Which learn-* skill detected this gap.
    pub source_skill: String,
    /// Source reference from grounding corpus that identifies this gap.
    pub source_evidence: Option<String>,
}

/// A learning session record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub skills_called: Vec<String>,
    pub concepts_touched: Vec<String>,
}

/// Cold-start seed output from learn-survey.
/// Consumed by `seed_from_survey()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerModelSeed {
    pub schema_version: String,
    pub learner_id: String,
    pub subject: String,
    pub surveyed_at: DateTime<Utc>,
    /// LLM-seeded Bayesian priors per concept. Used as initial mastery estimates.
    pub mastery_priors: Vec<MasteryPrior>,
    /// Concept IDs the learner demonstrably owns. feynman-loop never recurses into these.
    #[serde(default)]
    pub recursion_floor: Vec<String>,
    /// Misconceptions detected during the survey.
    #[serde(default)]
    pub misconceptions_detected: Vec<MisconceptionRecord>,
}

/// A single mastery prior from learn-survey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryPrior {
    pub concept_id: String,
    /// Initial mastery estimate [0,1]. Not a performance score — a prior probability.
    pub estimated_mastery_prior: f64,
    /// Confidence in the prior [0,1]. Low confidence = wide prior distribution.
    pub confidence: f64,
    /// How this prior was derived.
    pub basis: MasteryBasis,
}

/// Basis for a mastery prior estimate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MasteryBasis {
    SurveyResponse,
    SelfReportAdjusted,
    DiagnosticItem,
    DefaultPrior,
}

/// A misconception detected during the survey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MisconceptionRecord {
    pub concept_id: String,
    pub wrong_model: String,
    pub source_evidence: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learner_model_roundtrips_json() {
        let now = Utc::now();
        let mut concepts = HashMap::new();
        concepts.insert(
            "ownership".to_string(),
            ConceptState {
                concept_id: "ownership".to_string(),
                label: "Ownership".to_string(),
                mastery: 0.5,
                mastery_prior: Some(0.5),
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

        let model = LearnerModel {
            schema_version: "1.1.0".to_string(),
            learner_id: "did:plc:test".to_string(),
            created_at: now,
            updated_at: now,
            concepts,
            gaps: HashMap::new(),
            sessions: vec![],
        };

        let json = serde_json::to_string(&model).expect("serialize");
        let decoded: LearnerModel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.learner_id, "did:plc:test");
        assert_eq!(decoded.schema_version, "1.1.0");
        assert!(decoded.concepts.contains_key("ownership"));
    }

    #[test]
    fn card_state_serializes_correctly() {
        let state = CardState::Relearning;
        let json = serde_json::to_string(&state).expect("serialize");
        assert_eq!(json, "\"Relearning\"");
    }

    #[test]
    fn gap_severity_serializes_snake_case() {
        let s = GapSeverity::Misconception;
        let json = serde_json::to_string(&s).expect("serialize");
        assert_eq!(json, "\"misconception\"");
    }

    #[test]
    fn mastery_basis_serializes_snake_case() {
        let b = MasteryBasis::SelfReportAdjusted;
        let json = serde_json::to_string(&b).expect("serialize");
        assert_eq!(json, "\"self_report_adjusted\"");
    }
}
