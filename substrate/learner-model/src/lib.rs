//! `learner-model` — authoritative runtime for learner state in the Feynman learning loop.
//!
//! # Architecture
//!
//! - [`types`]: Core domain types mirroring `docs/learn/schemas/learner-model.schema.json`.
//! - [`store`]: Typed CRDT-backed store for reading and writing learner model documents.
//! - [`survey`]: Cold-start path: seed a `LearnerModel` from a `learn-survey` diagnostic output.
//! - [`fsrs`]: Minimal FSRS-6 stub for spaced-repetition scheduling.
//!
//! # Example
//!
//! ```rust,no_run
//! use learner_model::{survey::seed_from_survey, types::LearnerModelSeed};
//! use chrono::Utc;
//!
//! let seed = LearnerModelSeed {
//!     schema_version: "1.0.0".to_string(),
//!     learner_id: "did:plc:example".to_string(),
//!     subject: "Rust ownership".to_string(),
//!     surveyed_at: Utc::now(),
//!     mastery_priors: vec![],
//!     recursion_floor: vec![],
//!     misconceptions_detected: vec![],
//! };
//!
//! let model = seed_from_survey(&seed);
//! println!("Learner: {}", model.learner_id);
//! ```

pub mod fsrs;
pub mod store;
pub mod survey;
pub mod types;

// Re-export the most-used public API so callers can write `learner_model::LearnerModel`.
pub use fsrs::{next_review, Rating};
pub use store::{LearnerModelStore, StoreError};
pub use survey::seed_from_survey;
pub use types::{
    CardState, ConceptState, FSRSCard, GapRecord, GapSeverity, LearnerModel, LearnerModelSeed,
    MasteryBasis, MasteryPrior, MisconceptionRecord, ObservationRecord, SessionRecord,
};
