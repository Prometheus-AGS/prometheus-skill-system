//! Issue #256: replay the five recorded failure patterns against the advisory
//! reader. Each case must produce the stated advice and nothing else: no
//! journal event, no new phase or child, no changed status.

use chrono::Utc;
use kbd_runtime::advisory::bottleneck_advice;
use kbd_runtime::{
    ActivePath, Actor, Blocker, BlockerClass, Change, EventKind, GateKind, GateRun,
    MutationContext, Phase, Runtime, RuntimeState, WorkStatus,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const PHASE: &str = "replay-phase";

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../shared/references/conformance/kbd-bottleneck-advisory/v1/cases")
}

fn context(state: &RuntimeState, command_id: &str) -> MutationContext {
    MutationContext {
        expected_revision: state.revision,
        command_id: command_id.into(),
    }
}

fn class(value: &str) -> BlockerClass {
    serde_json::from_value(Value::String(value.into())).expect("known blocker class")
}

fn seed(runtime: &Runtime, actor: &Actor, case: &Value) -> RuntimeState {
    let mut state = runtime
        .initialize("replay-project", "replay-run", actor.clone())
        .expect("initialize runtime");
    state = runtime
        .define_phase(
            actor.clone(),
            context(&state, "define-phase"),
            Phase {
                id: PHASE.into(),
                slug: PHASE.into(),
                title: "Replay phase".into(),
                parent_phase_id: None,
                status: WorkStatus::InProgress,
                stages: BTreeMap::new(),
                changes: BTreeMap::new(),
                legacy_completion_baseline: None,
                legacy_read_only: false,
                outcome_first: false,
                outcome: None,
            },
        )
        .expect("define phase");
    for sequence in 1..=case["pendingChanges"].as_u64().expect("pendingChanges") {
        let id = format!("change-{sequence}");
        state = runtime
            .register_change(
                actor.clone(),
                context(&state, &format!("register-{id}")),
                PHASE,
                Change {
                    id: id.clone(),
                    title: id.clone(),
                    sequence,
                    status: WorkStatus::Pending,
                    implementation_status: WorkStatus::Pending,
                    tasks: BTreeMap::new(),
                    depends_on: Vec::new(),
                },
            )
            .expect("register change");
    }
    state = runtime
        .set_active_path(
            actor.clone(),
            context(&state, "activate-phase"),
            ActivePath {
                phase_path: vec![PHASE.into()],
                phase_id: Some(PHASE.into()),
                ..ActivePath::default()
            },
            None,
        )
        .expect("activate phase");
    for blocker in case["blockers"].as_array().expect("blockers") {
        let id = blocker["id"].as_str().expect("blocker id");
        state = runtime
            .record_blocker(
                actor.clone(),
                context(&state, &format!("record-{id}")),
                Blocker {
                    id: id.into(),
                    summary: blocker["summary"].as_str().expect("summary").into(),
                    resolved: false,
                    resolution: None,
                    fingerprint: blocker["fingerprint"].as_str().map(str::to_string),
                    class: blocker["class"].as_str().map(class),
                    evidence_ref: blocker["evidenceRef"].as_str().map(str::to_string),
                },
            )
            .expect("record blocker");
        if let Some(resolution) = blocker["cleared"].as_str() {
            state = runtime
                .clear_blocker(
                    actor.clone(),
                    context(&state, &format!("clear-{id}")),
                    id,
                    resolution,
                )
                .expect("clear blocker");
        }
    }
    if case["gateRunning"].as_bool().expect("gateRunning") {
        state = runtime
            .append(
                actor.clone(),
                state.revision,
                EventKind::GateStarted {
                    gate: GateRun {
                        id: "gate-running".into(),
                        kind: GateKind::Integration,
                        scope: "replay-build".into(),
                        phase_id: Some(PHASE.into()),
                        source_revision: state.revision,
                        executable: "cargo".into(),
                        command_sha256: "a".repeat(64),
                        worktree: "/replay".into(),
                        target_dir: None,
                        sccache_available: false,
                        started_at: Utc::now(),
                    },
                },
            )
            .expect("start gate");
    }
    state
}

fn replay(case_file: &str) -> (Value, Value) {
    let case: Value = serde_json::from_str(
        &fs::read_to_string(corpus().join(case_file)).expect("read replay case"),
    )
    .expect("parse replay case");
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("advisory-replay", "integration-test");
    let seeded = seed(&runtime, &actor, &case);
    let events_before = runtime.events().expect("read journal").len();

    // Read the advice the way the CLI does: from a replay of the journal.
    let state = runtime.replay_authority().expect("replay authority");
    let advice = bottleneck_advice(&state);
    assert!(advice.advisory, "{case_file}: advice is advisory");
    assert_eq!(advice.revision, seeded.revision);

    // Reading advice changed nothing: no event, no phase or child, no status.
    let after = Runtime::open(fixture.path())
        .replay_authority()
        .expect("replay after advice");
    assert_eq!(
        runtime.events().expect("read journal").len(),
        events_before,
        "{case_file}: no journal event may be appended"
    );
    assert_eq!(after.revision, seeded.revision, "{case_file}: revision");
    assert_eq!(after.phases, seeded.phases, "{case_file}: phases unchanged");
    assert_eq!(after.phases.len(), 1, "{case_file}: no phase or child created");
    assert_eq!(after.blockers, seeded.blockers, "{case_file}: blockers unchanged");
    assert_eq!(after.decisions, seeded.decisions, "{case_file}: decisions unchanged");
    assert_eq!(after.lifecycle, seeded.lifecycle, "{case_file}: lifecycle unchanged");

    let rendered = serde_json::to_value(&advice).expect("serialize advice");
    let expect = &case["expect"];
    assert_eq!(
        rendered["inFlight"].as_array().expect("inFlight").len() as u64,
        expect["inFlight"].as_u64().expect("expect.inFlight"),
        "{case_file}: running gates"
    );
    assert_eq!(
        rendered["withheldWhileInFlight"], expect["withheld"],
        "{case_file}: withheld recommendations"
    );
    let waiting: Vec<&str> = rendered["waiting"]
        .as_array()
        .expect("waiting")
        .iter()
        .map(|entry| entry["blockerId"].as_str().expect("blockerId"))
        .collect();
    let expected_waiting: Vec<&str> = expect["waiting"]
        .as_array()
        .expect("expect.waiting")
        .iter()
        .map(|entry| entry.as_str().expect("waiting id"))
        .collect();
    assert_eq!(waiting, expected_waiting, "{case_file}: waits");

    let recommendations = rendered["recommendations"].as_array().expect("recommendations");
    let expected = expect["recommendations"].as_array().expect("expect.recommendations");
    assert_eq!(
        recommendations.len(),
        expected.len(),
        "{case_file}: exactly the expected recommendations\n{rendered:#}"
    );
    for (actual, wanted) in recommendations.iter().zip(expected) {
        assert_eq!(actual["kind"], wanted["kind"], "{case_file}: kind");
        assert_eq!(actual["fingerprint"], wanted["fingerprint"], "{case_file}: fingerprint");
        assert_eq!(actual["attempts"], wanted["attempts"], "{case_file}: attempts");
        assert_eq!(
            actual.get("suggestedCommand").is_some(),
            wanted["suggestsCommand"].as_bool().expect("suggestsCommand"),
            "{case_file}: suggested command"
        );
    }
    (case, rendered)
}

#[test]
fn scope_dilution_proposes_a_child_and_runs_nothing() {
    let (_, advice) = replay("scope-dilution.json");
    let command = advice["recommendations"][0]["suggestedCommand"]
        .as_str()
        .expect("a command is suggested");
    assert!(command.contains("kbd-new-child.sh"));
}

#[test]
fn a_repeated_fixture_failure_is_not_reported_as_a_product_failure() {
    let (_, advice) = replay("mixed-product-fixture.json");
    assert!(advice["recommendations"][0]["proposal"]
        .as_str()
        .expect("proposal")
        .contains("not exercised"));
}

#[test]
fn waiting_for_an_approval_is_never_a_failed_attempt() {
    let (_, advice) = replay("approval-wait.json");
    assert!(advice["guidance"].as_str().expect("guidance").contains("not a failed attempt"));
}

#[test]
fn a_repeated_release_setup_failure_recommends_setup_repair_and_ignores_cleared_work() {
    replay("release-setup-failure.json");
}

#[test]
fn a_running_gate_withholds_every_recommendation() {
    let (_, advice) = replay("quiet-running-build.json");
    assert!(advice["guidance"]
        .as_str()
        .expect("guidance")
        .starts_with("in-flight: do not stop"));
}

#[test]
fn new_evidence_on_each_attempt_and_unfingerprinted_blockers_produce_no_advice() {
    let case = serde_json::json!({
        "pendingChanges": 1,
        "gateRunning": false,
        "blockers": [
            { "id": "learn-1", "summary": "Parser rejects the header", "fingerprint": "product:parser", "class": "product", "evidenceRef": "run-1" },
            { "id": "learn-2", "summary": "Parser rejects the footer", "fingerprint": "product:parser", "class": "product", "evidenceRef": "run-2" },
            { "id": "legacy-1", "summary": "Recorded before fingerprints existed" },
            { "id": "legacy-2", "summary": "Recorded before fingerprints existed" }
        ]
    });
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("advisory-replay", "integration-test");
    let state = seed(&runtime, &actor, &case);
    let advice = bottleneck_advice(&state);
    assert!(advice.recommendations.is_empty(), "{advice:#?}");
    assert!(advice.waiting.is_empty());

    // A blocker recorded without the new fields serializes exactly as before.
    let legacy = serde_json::to_value(&state.blockers["legacy-1"]).expect("serialize blocker");
    assert_eq!(
        legacy,
        serde_json::json!({
            "id": "legacy-1",
            "summary": "Recorded before fingerprints existed",
            "resolved": false,
            "resolution": null
        })
    );
}
