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
//! | `add_gap` | `{"learner_id", "concept_id", "description", "source_skill", "severity"?, "source_evidence"?, "label"?}` | Record a knowledge gap; returns `gap_id` |
//! | `resolve_gap` | `{"learner_id", "gap_id", "resolved_at"?}` | Mark a gap closed |
//! | `add_session` | `{"learner_id", "skills_called", "concepts_touched", "session_id"?, "session_type"?, "started_at"?, "ended_at"?}` | Record a session; returns `session_id` |
//! | `set_certified` | `{"learner_id", "concept_id", "certified_at"?}` | Record a checkpoint credential on the concept |
//!
//! `get_concept` answers with the concept state plus `gaps` (this concept's)
//! and `sessions` (those whose `concepts_touched` include it). See README.md.

use chrono::{DateTime, Utc};
use learner_model::{
    fsrs::Rating,
    store::LearnerModelStore,
    survey::seed_from_survey,
    types::{GapSeverity, LearnerModelSeed, SessionRecord},
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
                    Some(concept) => {
                        // The concept plus everything recorded about it: its
                        // gaps and the sessions that touched it.
                        let mut value = serde_json::to_value(concept).unwrap_or(json!({}));
                        let mut gaps: Vec<_> = model
                            .gaps
                            .values()
                            .filter(|g| g.concept_id == concept_id)
                            .cloned()
                            .collect();
                        gaps.sort_by(|a, b| {
                            (a.detected_at, a.gap_id.as_str())
                                .cmp(&(b.detected_at, b.gap_id.as_str()))
                        });
                        let sessions: Vec<_> = model
                            .sessions
                            .iter()
                            .filter(|s| s.concepts_touched.iter().any(|c| c == concept_id))
                            .cloned()
                            .collect();
                        if let Some(obj) = value.as_object_mut() {
                            obj.insert(
                                "gaps".into(),
                                serde_json::to_value(gaps).unwrap_or(json!([])),
                            );
                            obj.insert(
                                "sessions".into(),
                                serde_json::to_value(sessions).unwrap_or(json!([])),
                            );
                        }
                        value
                    }
                    None => json!({"error": format!("concept not found: {}", concept_id)}),
                },
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "add_gap" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            let concept_id = params["concept_id"].as_str().unwrap_or("");
            let description = params["description"].as_str().unwrap_or("").trim();
            let source_skill = params["source_skill"].as_str().unwrap_or("unknown");
            if learner_id.is_empty() || concept_id.is_empty() {
                return json!({"error": "missing params.learner_id or params.concept_id"})
                    .to_string();
            }
            if description.is_empty() {
                return json!({"error": "missing params.description"}).to_string();
            }
            // Enumerated params: absent, null, or "" mean "not given"; any
            // other value must be one of the vocabulary strings. A present
            // non-string (123, true, {}) is refused, not treated as absent.
            let severity = match optional_enum(&params["severity"]) {
                Ok(None) => None,
                Ok(Some("minor")) => Some(GapSeverity::Minor),
                Ok(Some("major")) => Some(GapSeverity::Major),
                Ok(Some("misconception")) => Some(GapSeverity::Misconception),
                Ok(Some(_)) | Err(()) => {
                    return json!({"error": "invalid params.severity; expected minor|major|misconception"})
                        .to_string()
                }
            };
            let label = match optional_enum(&params["label"]) {
                Ok(None) => None,
                Ok(Some(l @ ("verified" | "inferred"))) => Some(l.to_string()),
                Ok(Some(_)) | Err(()) => {
                    return json!({"error": "invalid params.label; expected verified|inferred"})
                        .to_string()
                }
            };
            let detected_at = match parse_timestamp(&params["detected_at"]) {
                Ok(t) => t,
                Err(e) => return json!({"error": e}).to_string(),
            };
            match store
                .add_gap(
                    learner_id,
                    concept_id,
                    description,
                    severity,
                    source_skill,
                    params["source_evidence"].as_str().map(str::to_string),
                    label,
                    detected_at,
                )
                .await
            {
                Ok(gap_id) => json!({"ok": true, "gap_id": gap_id}),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "resolve_gap" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            let gap_id = params["gap_id"].as_str().unwrap_or("");
            if learner_id.is_empty() || gap_id.is_empty() {
                return json!({"error": "missing params.learner_id or params.gap_id"}).to_string();
            }
            let resolved_at = match parse_timestamp(&params["resolved_at"]) {
                Ok(t) => t,
                Err(e) => return json!({"error": e}).to_string(),
            };
            match store.resolve_gap(learner_id, gap_id, resolved_at).await {
                Ok(()) => json!({"ok": true, "gap_id": gap_id, "resolved_at": resolved_at}),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "add_session" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            if learner_id.is_empty() {
                return json!({"error": "missing params.learner_id"}).to_string();
            }
            let strings = |key: &str| -> Vec<String> {
                params[key]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let skills_called = strings("skills_called");
            let concepts_touched = strings("concepts_touched");
            if skills_called.is_empty() {
                return json!({"error": "missing params.skills_called (non-empty array)"})
                    .to_string();
            }
            // An absent started_at on an update keeps the opening call's value
            // (the store preserves it); on a new session it means now.
            let explicit_start = match parse_optional_timestamp(&params["started_at"]) {
                Ok(t) => t,
                Err(e) => return json!({"error": e}).to_string(),
            };
            let ended_at = match parse_optional_timestamp(&params["ended_at"]) {
                Ok(t) => t,
                Err(e) => return json!({"error": e}).to_string(),
            };
            let session_id = params["session_id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let session = SessionRecord {
                session_id,
                started_at: explicit_start.unwrap_or_else(Utc::now),
                ended_at,
                skills_called,
                concepts_touched,
                session_type: params["session_type"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
            };
            let preserve_start = explicit_start.is_none();
            match store.add_session(learner_id, session, preserve_start).await {
                Ok(id) => json!({"ok": true, "session_id": id}),
                Err(e) => json!({"error": e.to_string()}),
            }
        }

        "set_certified" => {
            let learner_id = params["learner_id"].as_str().unwrap_or("");
            let concept_id = params["concept_id"].as_str().unwrap_or("");
            if learner_id.is_empty() || concept_id.is_empty() {
                return json!({"error": "missing params.learner_id or params.concept_id"})
                    .to_string();
            }
            let certified_at = match parse_timestamp(&params["certified_at"]) {
                Ok(t) => t,
                Err(e) => return json!({"error": e}).to_string(),
            };
            match store
                .set_certified(learner_id, concept_id, certified_at)
                .await
            {
                Ok(()) => {
                    json!({"ok": true, "concept_id": concept_id, "certified_at": certified_at})
                }
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

/// An optional enumerated string parameter. `Ok(None)` for absent, null, or
/// empty; `Ok(Some(s))` for a string; `Err(())` for a present non-string value.
fn optional_enum(value: &Value) -> Result<Option<&str>, ()> {
    match value {
        Value::Null => Ok(None),
        Value::String(s) if s.is_empty() => Ok(None),
        Value::String(s) => Ok(Some(s.as_str())),
        _ => Err(()),
    }
}

/// An optional RFC 3339 timestamp parameter: absent or null means now; any
/// other non-string value is a caller error, never silently "now".
fn parse_timestamp(value: &Value) -> Result<DateTime<Utc>, String> {
    parse_optional_timestamp(value).map(|t| t.unwrap_or_else(Utc::now))
}

/// Like `parse_timestamp` but keeps "absent" distinct from a value.
fn parse_optional_timestamp(value: &Value) -> Result<Option<DateTime<Utc>>, String> {
    match value {
        Value::Null => Ok(None),
        Value::String(s) => DateTime::parse_from_rfc3339(s)
            .map(|t| Some(t.with_timezone(&Utc)))
            .map_err(|_| format!("invalid timestamp {s:?}; expected RFC3339")),
        other => Err(format!(
            "invalid timestamp {other}; expected an RFC3339 string or null"
        )),
    }
}
