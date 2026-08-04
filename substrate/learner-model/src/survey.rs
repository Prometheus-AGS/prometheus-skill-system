use crate::types::{CardState, ConceptState, FSRSCard, LearnerModel, LearnerModelSeed};
use chrono::Utc;
use std::collections::{BTreeMap, HashMap};

/// Seed a new `LearnerModel` from a learn-survey diagnostic output.
///
/// This is the cold-start path: creates one `ConceptState` per `mastery_prior` entry.
/// Concepts with an estimated mastery > 0.7 are seeded into `CardState::Review`
/// (they have existing knowledge); all others start as `CardState::New`.
///
/// The initial review date (`due`) is set to now so the first review is immediate.
pub fn seed_from_survey(seed: &LearnerModelSeed) -> LearnerModel {
    let now = Utc::now();

    let concepts: HashMap<String, ConceptState> = seed
        .mastery_priors
        .iter()
        .map(|prior| {
            let state = if prior.estimated_mastery_prior > 0.7 {
                CardState::Review
            } else {
                CardState::New
            };

            let concept = ConceptState {
                concept_id: prior.concept_id.clone(),
                label: prior.concept_id.replace('-', " "),
                mastery: prior.estimated_mastery_prior,
                mastery_prior: Some(prior.estimated_mastery_prior),
                observations: BTreeMap::new(),
                fsrs_card: FSRSCard {
                    // New cards start with stability=1 day, difficulty=5 (neutral midpoint)
                    stability: 1.0,
                    difficulty: 5.0,
                    due: now,
                    state,
                    reps: 0,
                    lapses: 0,
                    last_review: None,
                },
                fsrs_prior: None,
            };

            (prior.concept_id.clone(), concept)
        })
        .collect();

    LearnerModel {
        schema_version: "1.1.0".to_string(),
        learner_id: seed.learner_id.clone(),
        created_at: now,
        updated_at: now,
        concepts,
        gaps: HashMap::new(),
        sessions: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MasteryBasis, MasteryPrior};

    fn make_seed(priors: Vec<(&str, f64)>) -> LearnerModelSeed {
        LearnerModelSeed {
            schema_version: "1.0.0".to_string(),
            learner_id: "did:plc:test-survey".to_string(),
            subject: "Rust programming".to_string(),
            surveyed_at: Utc::now(),
            mastery_priors: priors
                .into_iter()
                .map(|(concept_id, mastery)| MasteryPrior {
                    concept_id: concept_id.to_string(),
                    estimated_mastery_prior: mastery,
                    confidence: 0.8,
                    basis: MasteryBasis::SurveyResponse,
                })
                .collect(),
            recursion_floor: vec![],
            misconceptions_detected: vec![],
        }
    }

    #[test]
    fn seeds_one_concept_per_prior() {
        let seed = make_seed(vec![
            ("ownership", 0.3),
            ("lifetimes", 0.6),
            ("traits", 0.9),
        ]);
        let model = seed_from_survey(&seed);

        assert_eq!(model.schema_version, "1.1.0");
        assert_eq!(model.learner_id, "did:plc:test-survey");
        assert_eq!(model.concepts.len(), 3);
        assert!(model.concepts.contains_key("ownership"));
        assert!(model.concepts.contains_key("lifetimes"));
        assert!(model.concepts.contains_key("traits"));
    }

    #[test]
    fn low_mastery_concept_starts_as_new() {
        let seed = make_seed(vec![("ownership", 0.3)]);
        let model = seed_from_survey(&seed);
        assert_eq!(model.concepts["ownership"].fsrs_card.state, CardState::New);
    }

    #[test]
    fn high_mastery_concept_starts_as_review() {
        let seed = make_seed(vec![("traits", 0.85)]);
        let model = seed_from_survey(&seed);
        assert_eq!(model.concepts["traits"].fsrs_card.state, CardState::Review);
    }

    #[test]
    fn boundary_mastery_0_7_starts_as_review() {
        // > 0.7 → Review; exactly 0.7 → New
        let seed = make_seed(vec![("a", 0.70), ("b", 0.71)]);
        let model = seed_from_survey(&seed);
        assert_eq!(model.concepts["a"].fsrs_card.state, CardState::New);
        assert_eq!(model.concepts["b"].fsrs_card.state, CardState::Review);
    }

    #[test]
    fn mastery_prior_transferred_to_concept() {
        let seed = make_seed(vec![("closures", 0.55)]);
        let model = seed_from_survey(&seed);
        let concept = &model.concepts["closures"];
        assert!((concept.mastery - 0.55).abs() < f64::EPSILON);
        assert_eq!(concept.observations.len(), 0);
    }

    #[test]
    fn label_derived_from_concept_id_kebab_case() {
        let seed = make_seed(vec![("async-await", 0.4)]);
        let model = seed_from_survey(&seed);
        assert_eq!(model.concepts["async-await"].label, "async await");
    }

    #[test]
    fn empty_seed_produces_empty_model() {
        let seed = make_seed(vec![]);
        let model = seed_from_survey(&seed);
        assert!(model.concepts.is_empty());
        assert!(model.gaps.is_empty());
        assert!(model.sessions.is_empty());
    }
}
