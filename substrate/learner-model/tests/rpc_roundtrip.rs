//! Integration test for the `learner-model` binary over stdin and stdout
//! (change-rah-009). The built binary is driven with one JSON-RPC line per
//! request against a temporary data directory; every assertion is on a reply
//! the binary wrote or a document a fresh `load` returned.
//!
//! Sequence: seed from a survey; five observations (mastery moves only from the
//! fifth); add_gap; add_session; set_certified; review Good then review Hard
//! (difficulty differs); get_concept (gaps, sessions, certified_at present);
//! load (gaps and sessions present with their ids); resolve_gap.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn drive(data_dir: &std::path::Path, requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_learner-model"))
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn learner-model");
    {
        let mut stdin = child.stdin.take().expect("stdin");
        for r in requests {
            writeln!(stdin, "{}", r).expect("write request");
        }
        // Closing stdin ends the loop in main(); replies were written in order.
    }
    let stdout = child.stdout.take().expect("stdout");
    let replies: Vec<Value> = BufReader::new(stdout)
        .lines()
        .map(|l| l.expect("read reply"))
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(&l).unwrap_or_else(|e| panic!("reply is not JSON: {e}: {l}")))
        .collect();
    let status = child.wait().expect("wait");
    assert!(status.success(), "learner-model exited {status}");
    assert_eq!(
        replies.len(),
        requests.len(),
        "one reply per request: {replies:#?}"
    );
    replies
}

fn ok(reply: &Value, what: &str) {
    assert!(
        reply.get("error").is_none(),
        "{what} returned an error: {reply}"
    );
}

#[test]
fn rpc_round_trip_over_the_built_binary() {
    let data_dir = std::env::temp_dir().join(format!(
        "learner-model-rpc-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&data_dir).unwrap();
    let learner = "did:plc:rpc-roundtrip";
    let concept = "c1";
    let get = |learner_id: &str, concept_id: &str| json!({"method": "get_concept", "params": {"learner_id": learner_id, "concept_id": concept_id}});
    let observe = |score: f64| json!({"method": "add_observation", "params": {"learner_id": learner, "concept_id": concept, "score": score, "source_skill": "learn-grade"}});

    // ---- phase 1: seed, then four observations leave mastery at the prior ----
    let seed = json!({"method": "seed_from_survey", "params": {"seed": {
        "schema_version": "1.0.0",
        "learner_id": learner,
        "subject": "vector databases",
        "surveyed_at": "2026-09-01T00:00:00Z",
        "mastery_priors": [
            {"concept_id": concept, "estimated_mastery_prior": 0.4, "confidence": 0.5, "basis": "survey_response"},
            {"concept_id": "c2", "estimated_mastery_prior": 0.8, "confidence": 0.9, "basis": "diagnostic_item"}
        ],
        "recursion_floor": ["c2"],
        "misconceptions_detected": []
    }}});
    let mut requests = vec![seed, get(learner, concept)];
    for _ in 0..4 {
        requests.push(observe(1.0));
    }
    requests.push(get(learner, concept));
    requests.push(observe(1.0));
    requests.push(get(learner, concept));
    let replies = drive(&data_dir, &requests);
    ok(&replies[0], "seed_from_survey");
    assert_eq!(replies[0]["learner_id"], learner);
    ok(&replies[1], "get_concept after seed");
    assert_eq!(replies[1]["mastery"], 0.4);
    assert!(
        replies[1]["certified_at"].is_null(),
        "not certified at seed time"
    );
    assert_eq!(replies[1]["gaps"], json!([]));
    assert_eq!(replies[1]["sessions"], json!([]));
    for (i, r) in replies[2..6].iter().enumerate() {
        ok(r, &format!("add_observation #{}", i + 1));
    }
    ok(&replies[6], "get_concept after four observations");
    assert_eq!(
        replies[6]["mastery"], 0.4,
        "mastery unchanged before the fifth observation"
    );
    assert_eq!(
        replies[6]["observations"].as_object().map(|o| o.len()),
        Some(4)
    );
    ok(&replies[7], "fifth add_observation");
    ok(&replies[8], "get_concept after five observations");
    let mastery_after_five = replies[8]["mastery"].as_f64().unwrap();
    assert!(
        mastery_after_five > 0.4,
        "mastery moved only from the fifth observation: {mastery_after_five}"
    );
    assert!(
        (mastery_after_five - (0.4 + 0.3 * (1.0 - 0.4))).abs() < 1e-9,
        "PFA rule applied once: {mastery_after_five}"
    );

    // ---- phase 2: gap, session, certification, two reviews --------------------
    let requests = vec![
        json!({"method": "add_gap", "params": {"learner_id": learner, "concept_id": concept,
            "description": "does not distinguish HNSW recall from exact search recall",
            "severity": "major", "source_skill": "learn-grade",
            "source_evidence": "corpus:vector-db#hnsw-recall", "label": "verified",
            "detected_at": "2026-09-02T10:00:00Z"}}),
        json!({"method": "add_session", "params": {"learner_id": learner,
            "session_id": "sess-practice-001", "session_type": "practice",
            "skills_called": ["learn-practice", "learn-grade"], "concepts_touched": [concept],
            "started_at": "2026-09-02T09:00:00Z", "ended_at": "2026-09-02T09:40:00Z"}}),
        // The same session closed later, without repeating started_at: replaces,
        // never duplicates, and the opening started_at is preserved.
        json!({"method": "add_session", "params": {"learner_id": learner,
            "session_id": "sess-practice-001", "session_type": "practice",
            "skills_called": ["learn-practice", "learn-grade"], "concepts_touched": [concept],
            "ended_at": "2026-09-02T09:55:00Z"}}),
        json!({"method": "add_session", "params": {"learner_id": learner,
            "skills_called": ["feynman-loop"], "concepts_touched": ["c2"], "session_type": "feynman",
            "started_at": "2026-09-02T11:00:00Z"}}),
        json!({"method": "set_certified", "params": {"learner_id": learner, "concept_id": concept,
            "certified_at": "2026-09-03T12:00:00Z"}}),
        json!({"method": "review", "params": {"learner_id": learner, "concept_id": concept,
            "score": 0.9, "rating": "good", "source_skill": "learn-retain",
            "timestamp": "2026-09-04T12:00:00Z"}}),
        json!({"method": "review", "params": {"learner_id": learner, "concept_id": concept,
            "score": 0.6, "rating": "hard", "source_skill": "learn-retain",
            "timestamp": "2026-09-08T12:00:00Z"}}),
        get(learner, concept),
        json!({"method": "load", "params": {"learner_id": learner}}),
        // Error paths answer with error, never exit.
        json!({"method": "add_gap", "params": {"learner_id": learner, "concept_id": "nope",
            "description": "x", "source_skill": "learn-grade"}}),
        json!({"method": "set_certified", "params": {"learner_id": learner, "concept_id": concept,
            "certified_at": "not a timestamp"}}),
        json!({"method": "add_gap", "params": {"learner_id": learner, "concept_id": concept,
            "description": "x", "source_skill": "learn-grade", "label": "guessed"}}),
        // A present-but-wrong-type timestamp is an error, never silently "now".
        json!({"method": "set_certified", "params": {"learner_id": learner, "concept_id": "c2",
            "certified_at": 12345}}),
        json!({"method": "add_session", "params": {"learner_id": learner,
            "skills_called": ["learn-practice"], "concepts_touched": [concept],
            "ended_at": {"at": "2026-09-02T09:55:00Z"}}}),
        // A present non-string label or severity is refused, not read as absent.
        json!({"method": "add_gap", "params": {"learner_id": learner, "concept_id": concept,
            "description": "x", "source_skill": "learn-grade", "label": 123}}),
        json!({"method": "add_gap", "params": {"learner_id": learner, "concept_id": concept,
            "description": "x", "source_skill": "learn-grade", "severity": true}}),
    ];
    let replies = drive(&data_dir, &requests);
    ok(&replies[0], "add_gap");
    let gap_id = replies[0]["gap_id"].as_str().expect("gap_id").to_string();
    ok(&replies[1], "add_session");
    assert_eq!(replies[1]["session_id"], "sess-practice-001");
    ok(&replies[2], "add_session (update)");
    ok(&replies[3], "add_session (generated id)");
    let feynman_session_id = replies[3]["session_id"]
        .as_str()
        .expect("generated session_id")
        .to_string();
    assert!(!feynman_session_id.is_empty());
    ok(&replies[4], "set_certified");
    assert_eq!(replies[4]["certified_at"], "2026-09-03T12:00:00Z");
    ok(&replies[5], "review good");
    ok(&replies[6], "review hard");
    let good_card = &replies[5]["fsrs_card"];
    let hard_card = &replies[6]["fsrs_card"];
    let d_good = good_card["difficulty"].as_f64().unwrap();
    let d_hard = hard_card["difficulty"].as_f64().unwrap();
    assert_ne!(
        d_good, d_hard,
        "difficulty is read and updated on every review: {good_card} vs {hard_card}"
    );
    assert!(d_hard > d_good, "Hard after Good raises difficulty");
    assert_eq!(hard_card["reps"], 2);
    assert!(hard_card["stability"].as_f64().unwrap() > 0.0);

    ok(&replies[7], "get_concept after everything");
    let c = &replies[7];
    assert_eq!(
        c["certified_at"], "2026-09-03T12:00:00Z",
        "certified_at set and returned"
    );
    assert_eq!(c["gaps"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(c["gaps"][0]["gap_id"], gap_id.as_str());
    assert_eq!(c["gaps"][0]["label"], "verified");
    assert_eq!(c["gaps"][0]["severity"], "major");
    assert!(c["gaps"][0]["resolved_at"].is_null());
    assert_eq!(
        c["sessions"].as_array().map(|a| a.len()),
        Some(1),
        "only sessions touching c1, deduplicated"
    );
    assert_eq!(c["sessions"][0]["session_id"], "sess-practice-001");
    assert_eq!(
        c["sessions"][0]["started_at"], "2026-09-02T09:00:00Z",
        "closing without started_at preserved the opening timestamp"
    );
    assert_eq!(
        c["sessions"][0]["ended_at"], "2026-09-02T09:55:00Z",
        "the later report replaced the earlier"
    );
    assert_eq!(c["sessions"][0]["session_type"], "practice");
    assert_eq!(c["fsrs_card"]["difficulty"], hard_card["difficulty"]);
    assert_eq!(
        c["observations"].as_object().map(|o| o.len()),
        Some(7),
        "5 observations + 2 reviews"
    );

    ok(&replies[8], "load");
    let model = &replies[8];
    assert_eq!(model["schema_version"], "1.1.0");
    assert!(
        model["gaps"][gap_id.as_str()].is_object(),
        "gap survives a fresh load, keyed by id"
    );
    let session_ids: Vec<&str> = model["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["session_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        session_ids,
        vec!["sess-practice-001", feynman_session_id.as_str()],
        "sessions deduplicated and ordered by started_at"
    );
    assert_eq!(
        model["concepts"][concept]["certified_at"],
        "2026-09-03T12:00:00Z"
    );
    assert!(model["concepts"]["c2"]["certified_at"].is_null());

    assert!(
        replies[9]["error"].as_str().unwrap_or("").contains("nope"),
        "unknown concept: {}",
        replies[9]
    );
    assert!(
        replies[10]["error"]
            .as_str()
            .unwrap_or("")
            .contains("RFC3339"),
        "bad timestamp: {}",
        replies[10]
    );
    assert!(
        replies[11]["error"]
            .as_str()
            .unwrap_or("")
            .contains("label"),
        "bad label: {}",
        replies[11]
    );
    assert!(
        replies[12]["error"]
            .as_str()
            .unwrap_or("")
            .contains("RFC3339"),
        "numeric certified_at must be an error, not now: {}",
        replies[12]
    );
    assert!(
        replies[13]["error"]
            .as_str()
            .unwrap_or("")
            .contains("RFC3339"),
        "object ended_at must be an error, not now: {}",
        replies[13]
    );
    assert!(
        replies[14]["error"]
            .as_str()
            .unwrap_or("")
            .contains("label"),
        "numeric label must be refused: {}",
        replies[14]
    );
    assert!(
        replies[15]["error"]
            .as_str()
            .unwrap_or("")
            .contains("severity"),
        "boolean severity must be refused: {}",
        replies[15]
    );
    // None of the error calls wrote anything: c2 is still uncertified, the
    // session list and the gap count are unchanged in a fresh process
    // (asserted in phase 3).

    // ---- phase 3: resolve the gap; a fresh process sees it closed ---------------
    let replies = drive(
        &data_dir,
        &[
            json!({"method": "resolve_gap", "params": {"learner_id": learner, "gap_id": gap_id, "resolved_at": "2026-09-09T08:00:00Z"}}),
            get(learner, concept),
            json!({"method": "resolve_gap", "params": {"learner_id": learner, "gap_id": "missing"}}),
            json!({"method": "load", "params": {"learner_id": learner}}),
        ],
    );
    ok(&replies[0], "resolve_gap");
    assert_eq!(replies[1]["gaps"][0]["resolved_at"], "2026-09-09T08:00:00Z");
    assert!(replies[2]["error"]
        .as_str()
        .unwrap_or("")
        .contains("Gap not found"));
    assert!(replies[3]["concepts"]["c2"]["certified_at"].is_null());
    assert_eq!(
        replies[3]["sessions"].as_array().map(|a| a.len()),
        Some(2),
        "rejected add_session wrote nothing"
    );
    assert_eq!(
        replies[3]["gaps"].as_object().map(|g| g.len()),
        Some(1),
        "rejected add_gap calls wrote nothing"
    );

    let _ = std::fs::remove_dir_all(&data_dir);
}
