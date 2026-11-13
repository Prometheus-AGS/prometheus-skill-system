//! Replay compatibility for issue #219: a signed journal whose PASS closure
//! was written by prometheus 1.13.2 (before evidence admission) replays
//! through the current reducer to the state that binary reported.
//!
//! The fixture is the issue's reproduction: the closure's evidence named
//! another phase and project and carried no source digests or observation
//! fields. The current admission refuses that evidence for a new closure;
//! the recorded one keeps the semantics it was recorded under, and replay
//! reads no files (the fixture has no project tree at all).

use kbd_runtime::{replay_events, Event, WorkStatus};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const PHASE: &str = "closure-phase";

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pre-219-closure-journal")
        .join(name)
}

fn events() -> Vec<Event> {
    fs::read_to_string(fixture("events.jsonl"))
        .expect("read pre-219 journal")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("pre-219 event parses"))
        .collect()
}

#[test]
fn pre_219_closure_replays_to_the_state_the_old_binary_reported() {
    let state =
        replay_events(&events()).expect("pre-219 journal replays under the current reducer");
    let old: Value = serde_json::from_str(
        &fs::read_to_string(fixture("status-prometheus-1.13.2.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(state.project_id, old["projectId"].as_str().unwrap());
    assert_eq!(state.run_id, old["runId"].as_str().unwrap());
    assert_eq!(state.revision, old["revision"].as_u64().unwrap());
    for (id, phase) in &state.phases {
        assert_eq!(
            serde_json::to_value(&phase.status).unwrap(),
            old["phases"][id]["status"],
            "phase {id} status"
        );
    }
    assert_eq!(state.phases[PHASE].status, WorkStatus::Complete);

    let recorded = state.phase_closure.phases[PHASE]
        .terminal
        .as_ref()
        .expect("the recorded closure is preserved");
    assert_eq!(
        recorded.closure.admission_version, None,
        "no admission version is invented for a pre-change closure"
    );
    assert_eq!(recorded.closure.input.evidence_facts, None);
    assert_eq!(
        serde_json::to_value(recorded).unwrap(),
        old["phaseClosure"]["phases"][PHASE]["terminal"],
        "the closure record serializes exactly as the old binary wrote it"
    );
}
