//! `learner-model` binary — JSON-RPC shell interface for skill callers.
//!
//! Reads newline-delimited JSON-RPC commands from stdin; writes JSON responses to stdout.
//! Skills can call this binary without any Rust tooling installed.
//!
//! # Usage
//!
//! ```bash
//! learner-model [data-dir]
//! ```
//!
//! Default data dir: `~/.prometheus/learn/learner-model`
//!
//! # Supported Methods
//!
//! | Method | Params | Description |
//! |--------|--------|-------------|
//! | `load` | `{"learner_id": "..."}` | Load a learner model |
//! | `seed_from_survey` | `{"seed": {...}}` | Seed model from survey output |
//! | `get_concept` | `{"learner_id", "concept_id"}` | Load one concept state |
//! | `add_observation` | `{"learner_id", "concept_id", "score", "source_skill"}` | Record observation |
//! | `review` | `{"learner_id", "concept_id", "score", "rating", "source_skill"}` | Record retention review and advance FSRS |

use chrono::{DateTime, Utc};
use learner_model::{
    fsrs::Rating, store::LearnerModelStore, survey::seed_from_survey, types::LearnerModelSeed,
};
use serde_json::{json, Value};
use std::io::{self, BufRead};
use storage_provider::{LocalDirAdapter, LoroAdapter};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // learner-model [data-dir]  — reads JSON-RPC commands from stdin
    let data_dir = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("~/.prometheus/learn/learner-model");
    let data_dir = shellexpand::tilde(data_dir).to_string();

    let storage = LocalDirAdapter::new(&data_dir);
    let crdt = LoroAdapter;
    let store = LearnerModelStore::new(storage, crdt);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("stdin error: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let result = handle_command(&store, &line).await;
        println!("{}", result);
    }
}

async fn handle_command(
    store: &LearnerModelStore<LocalDirAdapter, LoroAdapter>,
    line: &str,
) -> String {
    let cmd: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => return json!({"error": format!("parse error: {}", e)}).to_string(),
    };

    let method = cmd["method"].as_str().unwrap_or("");
    let params = &cmd["params"];

    let result = match method {
        "load" => {
            let learner_id = match params["learner_id"].as_str() {
                Some(id) => id,
                None => return json!({"error": "missing params.learner_id"}).to_string(),
            };
            match store.load(learner_id).await {
                Ok(m) => serde_json::to_value(&m).unwrap_or(json!(null)),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "seed_from_survey" => {
            match serde_json::from_value::<LearnerModelSeed>(params["seed"].clone()) {
                Ok(seed) => {
                    let model = seed_from_survey(&seed);
                    match store.save(&model).await {
                        Ok(()) => json!({"ok": true, "learner_id": model.learner_id}),
                        Err(e) => json!({"error": e.to_string()}),
                    }
                }
                Err(e) => json!({"error": format!("invalid seed: {}", e)}),
            }
        }

        "get_concept" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            let concept_id = params["concept_id"].as_str().unwrap_or("");
            if learner_id.is_empty() || concept_id.is_empty() {
                return json!({"error": "missing params.learner_id or params.concept_id"})
                    .to_string();
            }
            match store.load(learner_id).await {
                Ok(model) => match model.concepts.get(concept_id) {
                    Some(concept) => serde_json::to_value(concept).unwrap_or(json!(null)),
                    None => json!({"error": format!("concept not found: {}", concept_id)}),
                },
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "add_observation" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            let concept_id = params["concept_id"].as_str().unwrap_or("");
            let score = params["score"].as_f64().unwrap_or(0.0);
            let source_skill = params["source_skill"].as_str().unwrap_or("unknown");

            if learner_id.is_empty() || concept_id.is_empty() {
                return json!({"error": "missing params.learner_id or params.concept_id"})
                    .to_string();
            }

            match store
                .add_observation(learner_id, concept_id, score, source_skill)
                .await
            {
                Ok(()) => json!({"ok": true}),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "review" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            let concept_id = params["concept_id"].as_str().unwrap_or("");
            let score = params["score"].as_f64().unwrap_or(0.0).clamp(0.0, 1.0);
            let source_skill = params["source_skill"].as_str().unwrap_or("learn-retain");
            let rating = match params["rating"]
                .as_str()
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str()
            {
                "again" => Rating::Again,
                "hard" => Rating::Hard,
                "good" => Rating::Good,
                "easy" => Rating::Easy,
                _ => {
                    return json!({"error": "invalid params.rating; expected again|hard|good|easy"})
                        .to_string()
                }
            };
            if learner_id.is_empty() || concept_id.is_empty() {
                return json!({"error": "missing params.learner_id or params.concept_id"})
                    .to_string();
            }
            let reviewed_at = match params["timestamp"].as_str() {
                Some(value) => match DateTime::parse_from_rfc3339(value) {
                    Ok(value) => value.with_timezone(&Utc),
                    Err(_) => {
                        return json!({"error": "invalid params.timestamp; expected RFC3339"})
                            .to_string()
                    }
                },
                None => Utc::now(),
            };
            match store
                .review_concept(
                    learner_id,
                    concept_id,
                    score,
                    rating,
                    source_skill,
                    reviewed_at,
                )
                .await
            {
                Ok(card) => json!({"ok": true, "fsrs_card": card}),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        _ => json!({"error": format!("unknown method: {}", method)}),
    };

    result.to_string()
}
