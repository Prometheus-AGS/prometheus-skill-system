//! Replay compatibility for issue #192: a signed journal written by
//! prometheus 1.13.0 (before typed phase closure) replays through the current
//! event reducer to the same state the old binary reported, and serializes
//! without the new closure registry. Fixture captured by the pre-fix
//! reproduction (`repro-192.receipt.json`): a FAIL-refused phase that 1.13.0
//! still completed and continued.

use kbd_runtime::{replay_events, Event, WorkStatus};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pre-192-journal")
        .join(name)
}

fn events() -> Vec<Event> {
    fs::read_to_string(fixture("events.jsonl"))
        .expect("read pre-192 journal")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("pre-192 event parses"))
        .collect()
}

#[test]
fn pre_192_journal_replays_to_the_state_the_old_binary_reported() {
    let state =
        replay_events(&events()).expect("pre-192 journal replays under the current reducer");
    let old: Value = serde_json::from_str(
        &fs::read_to_string(fixture("status-prometheus-1.13.0.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(state.project_id, old["projectId"].as_str().unwrap());
    assert_eq!(state.run_id, old["runId"].as_str().unwrap());
    assert_eq!(state.revision, old["revision"].as_u64().unwrap());
    assert_eq!(
        state.active_path.phase_id.as_deref(),
        old["activePath"]["phaseId"].as_str()
    );
    let old_phases = old["phases"].as_object().unwrap();
    assert_eq!(state.phases.len(), old_phases.len());
    for (id, phase) in &state.phases {
        assert_eq!(
            serde_json::to_value(&phase.status).unwrap(),
            old_phases[id]["status"],
            "phase {id} status"
        );
    }
    let old_decisions = old["decisions"].as_object().unwrap();
    assert_eq!(state.decisions.len(), old_decisions.len());
    for (id, decision) in &state.decisions {
        assert_eq!(
            decision.summary,
            old_decisions[id]["summary"].as_str().unwrap(),
            "decision {id}"
        );
    }

    // History is preserved as recorded, including the legacy completion that
    // motivated #192; nothing is re-adjudicated on replay.
    assert_eq!(state.phases["refused-phase"].status, WorkStatus::Complete);
    assert_eq!(
        state.phases["successor-phase"].status,
        WorkStatus::InProgress
    );
    assert!(
        state.phase_closure.is_empty(),
        "no closure state is invented for legacy phases"
    );

    let serialized = serde_json::to_value(&state).unwrap();
    assert!(
        serialized.get("phaseClosure").is_none(),
        "an empty closure registry must not change the serialized state shape"
    );
}
