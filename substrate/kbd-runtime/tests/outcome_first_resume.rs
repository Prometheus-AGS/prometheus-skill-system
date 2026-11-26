//! Issue #256: an outcome-first phase keeps blocked work from outranking
//! eligible pending work, and carries the outcome, the last meaningful
//! evidence and the next authorized action through the projections. A phase
//! that did not opt in selects and projects exactly as before.

use kbd_runtime::{
    ActivePath, Actor, Change, MutationContext, Phase, Runtime, RuntimeState, Task, WorkStatus,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const PHASE: &str = "outcome-phase";
const OUTCOME: &str = "An operator can export a signed report from the CLI";

fn context(state: &RuntimeState, command_id: &str) -> MutationContext {
    MutationContext {
        expected_revision: state.revision,
        command_id: command_id.into(),
    }
}

fn phase(outcome_first: bool) -> Phase {
    Phase {
        id: PHASE.into(),
        slug: PHASE.into(),
        title: "Outcome phase".into(),
        parent_phase_id: None,
        status: WorkStatus::InProgress,
        stages: BTreeMap::new(),
        changes: BTreeMap::new(),
        legacy_completion_baseline: None,
        legacy_read_only: false,
        outcome_first,
        outcome: outcome_first.then(|| OUTCOME.to_string()),
    }
}

fn change(id: &str, sequence: u64, depends_on: &[&str]) -> Change {
    Change {
        id: id.into(),
        title: format!("Change {id}"),
        sequence,
        status: WorkStatus::Pending,
        implementation_status: WorkStatus::Pending,
        tasks: BTreeMap::new(),
        depends_on: depends_on.iter().map(|id| (*id).to_string()).collect(),
    }
}

/// Three changes in ledger order: `change-a` (will be blocked), `change-b`
/// (independent) and `change-c` (depends on `change-a`). Each has one task.
fn seeded(runtime: &Runtime, actor: &Actor, outcome_first: bool) -> RuntimeState {
    let mut state = runtime
        .initialize("outcome-project", "outcome-run", actor.clone())
        .expect("initialize runtime");
    state = runtime
        .define_phase(
            actor.clone(),
            context(&state, "define-phase"),
            phase(outcome_first),
        )
        .expect("define phase");
    for (sequence, id, depends_on) in [
        (1, "change-a", &[][..]),
        (2, "change-b", &[][..]),
        (3, "change-c", &["change-a"][..]),
    ] {
        state = runtime
            .register_change(
                actor.clone(),
                context(&state, &format!("register-{id}")),
                PHASE,
                change(id, sequence, depends_on),
            )
            .expect("register change");
        state = runtime
            .register_task(
                actor.clone(),
                context(&state, &format!("register-{id}-task")),
                PHASE,
                id,
                Task {
                    id: "1".into(),
                    title: format!("Build {id}"),
                    sequence: 1,
                    status: WorkStatus::Pending,
                    summary: None,
                },
            )
            .expect("register task");
    }
    runtime
        .set_active_path(
            actor.clone(),
            context(&state, "activate-phase"),
            ActivePath {
                phase_path: vec![PHASE.into()],
                phase_id: Some(PHASE.into()),
                ..ActivePath::default()
            },
            Some("/kbd-apply change-a".into()),
        )
        .expect("activate phase")
}

fn transition(
    runtime: &Runtime,
    actor: &Actor,
    state: &RuntimeState,
    change_id: &str,
    to: WorkStatus,
    command_id: &str,
) -> RuntimeState {
    runtime
        .transition_task(
            actor.clone(),
            context(state, command_id),
            PHASE,
            change_id,
            "1",
            to,
            None,
        )
        .expect("transition task")
}

fn read_json(root: &Path, relative: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join(relative)).expect("read projection"))
        .expect("parse projection")
}

#[test]
fn blocked_change_does_not_outrank_independent_pending_work_in_an_outcome_first_phase() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("outcome-integration", "integration-test");
    let mut state = seeded(&runtime, &actor, true);

    state = transition(
        &runtime,
        &actor,
        &state,
        "change-a",
        WorkStatus::Blocked,
        "block-a",
    );
    assert_eq!(
        state.phases[PHASE].changes["change-a"].implementation_status,
        WorkStatus::Blocked
    );
    // change-b is pending with no dependencies: it is selected ahead of the
    // blocked change-a. change-c depends on change-a and is not eligible.
    assert_eq!(state.active_path.change_id.as_deref(), Some("change-b"));
    assert_eq!(state.active_path.task_id.as_deref(), Some("1"));

    // The same selection survives a restart and replay.
    let reopened = Runtime::open(fixture.path());
    let replayed = reopened.replay().expect("replay after restart");
    assert_eq!(replayed.active_path, state.active_path);
    assert!(replayed.phases[PHASE].outcome_first);
    assert_eq!(replayed.phases[PHASE].outcome.as_deref(), Some(OUTCOME));
    assert_eq!(
        replayed.phases[PHASE].changes["change-c"].depends_on,
        vec!["change-a".to_string()]
    );
    reopened
        .write_compatibility_projections()
        .expect("write projections");

    let waypoint = read_json(fixture.path(), ".kbd-orchestrator/current-waypoint.json");
    assert_eq!(waypoint["nextChange"], "change-b");
    assert_eq!(waypoint["nextTask"], "1");
    assert_eq!(waypoint["outcomeFirst"], true);
    assert_eq!(waypoint["outcome"], OUTCOME);
    assert_eq!(waypoint["nextAuthorizedAction"], "change change-b, task 1");
    assert!(waypoint["lastEvidence"].is_null());
    let blocked = waypoint["blockedChanges"]
        .as_array()
        .expect("blockedChanges array");
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0]["id"], "change-a");
    assert_eq!(blocked[0]["blockers"][0], "Build change-a");
    // Operator prose is carried unchanged and still selects nothing.
    assert_eq!(waypoint["exactNextCommand"], "/kbd-apply change-a");

    let reminder = fs::read_to_string(
        fixture
            .path()
            .join(".kbd-orchestrator/position-reminder.txt"),
    )
    .expect("read reminder");
    assert!(reminder.contains(&format!("Outcome: {OUTCOME}")));
    assert!(reminder.contains("Last evidence: (none yet)"));
    assert!(reminder.contains("Next authorized action: change change-b, task 1"));
    assert!(reminder.contains("Blocked, not selected: change-a"));
    assert!(reminder.contains("Next work: change change-b, task 1"));

    let progress = read_json(
        fixture.path(),
        &format!(".kbd-orchestrator/phases/{PHASE}/progress.json"),
    );
    assert_eq!(progress["outcomeFirst"], true);
    assert_eq!(progress["outcome"], OUTCOME);
    let rows = progress["changes"].as_array().expect("changes array");
    let change_c = rows
        .iter()
        .find(|row| row["id"] == "change-c")
        .expect("change-c row");
    assert_eq!(change_c["depends_on"][0], "change-a");
    let change_b = rows
        .iter()
        .find(|row| row["id"] == "change-b")
        .expect("change-b row");
    assert!(change_b.get("depends_on").is_none());
}

#[test]
fn dependent_work_waits_and_blocked_work_is_the_last_resort() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("outcome-integration", "integration-test");
    let mut state = seeded(&runtime, &actor, true);

    state = transition(
        &runtime,
        &actor,
        &state,
        "change-a",
        WorkStatus::Blocked,
        "block-a",
    );
    state = transition(
        &runtime,
        &actor,
        &state,
        "change-b",
        WorkStatus::InProgress,
        "start-b",
    );
    state = transition(
        &runtime,
        &actor,
        &state,
        "change-b",
        WorkStatus::Complete,
        "complete-b",
    );

    // Only change-a (blocked) and change-c (waiting on change-a) remain.
    // change-c must not be selected; the blocked change is surfaced instead.
    assert_eq!(state.active_path.change_id.as_deref(), Some("change-a"));
    runtime
        .write_compatibility_projections()
        .expect("write projections");
    let waypoint = read_json(fixture.path(), ".kbd-orchestrator/current-waypoint.json");
    assert_eq!(waypoint["nextChange"], "change-a");
    assert_eq!(
        waypoint["nextAuthorizedAction"],
        "resolve the blocker on change change-a; no other change is eligible"
    );
    assert_eq!(waypoint["lastEvidence"]["kind"], "task");
    assert_eq!(waypoint["lastEvidence"]["change"], "change-b");
    // The selected blocked change is not repeated in the "not selected" list.
    assert_eq!(
        waypoint["blockedChanges"]
            .as_array()
            .expect("blockedChanges array")
            .len(),
        0
    );

    // Clearing the blocker and finishing change-a makes change-c eligible.
    state = transition(
        &runtime,
        &actor,
        &state,
        "change-a",
        WorkStatus::InProgress,
        "unblock-a",
    );
    state = transition(
        &runtime,
        &actor,
        &state,
        "change-a",
        WorkStatus::Complete,
        "complete-a",
    );
    assert_eq!(state.active_path.change_id.as_deref(), Some("change-c"));
}

#[test]
fn a_phase_that_did_not_opt_in_selects_and_projects_as_before() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("outcome-integration", "integration-test");
    let mut state = seeded(&runtime, &actor, false);

    state = transition(
        &runtime,
        &actor,
        &state,
        "change-a",
        WorkStatus::Blocked,
        "block-a",
    );
    // Unchanged rule: blocked work is selected before pending work.
    assert_eq!(state.active_path.change_id.as_deref(), Some("change-a"));

    runtime
        .write_compatibility_projections()
        .expect("write projections");
    let waypoint = read_json(fixture.path(), ".kbd-orchestrator/current-waypoint.json");
    assert_eq!(waypoint["nextChange"], "change-a");
    for key in [
        "outcomeFirst",
        "outcome",
        "lastEvidence",
        "nextAuthorizedAction",
        "blockedChanges",
    ] {
        assert!(
            waypoint.get(key).is_none(),
            "waypoint of a phase that did not opt in must not carry {key}"
        );
    }
    let reminder = fs::read_to_string(
        fixture
            .path()
            .join(".kbd-orchestrator/position-reminder.txt"),
    )
    .expect("read reminder");
    assert!(!reminder.contains("Outcome:"));
    assert!(!reminder.contains("Next authorized action:"));
    let progress = read_json(
        fixture.path(),
        &format!(".kbd-orchestrator/phases/{PHASE}/progress.json"),
    );
    assert!(progress.get("outcomeFirst").is_none());
}

#[test]
fn records_without_the_new_fields_keep_their_serialized_form() {
    // Events written before these fields existed must keep their bytes, so
    // their integrity hashes and signatures still verify on replay.
    let plain_phase = serde_json::to_value(phase(false)).expect("serialize phase");
    assert!(plain_phase.get("outcomeFirst").is_none());
    assert!(plain_phase.get("outcome").is_none());
    let plain_change = serde_json::to_value(change("change-a", 1, &[])).expect("serialize change");
    assert!(plain_change.get("dependsOn").is_none());

    // And a record that never had them still loads, with the defaults.
    let loaded: Phase = serde_json::from_value(plain_phase).expect("load legacy phase");
    assert!(!loaded.outcome_first);
    assert!(loaded.outcome.is_none());
    let loaded: Change = serde_json::from_value(plain_change).expect("load legacy change");
    assert!(loaded.depends_on.is_empty());
}

#[test]
fn a_dependency_must_name_a_registered_change() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("outcome-integration", "integration-test");
    let state = seeded(&runtime, &actor, true);

    let unknown = runtime.register_change(
        actor.clone(),
        context(&state, "register-unknown-dependency"),
        PHASE,
        change("change-d", 4, &["change-missing"]),
    );
    assert!(unknown.is_err(), "an unknown dependency must be refused");

    let own = runtime.register_change(
        actor.clone(),
        context(&state, "register-self-dependency"),
        PHASE,
        change("change-e", 5, &["change-e"]),
    );
    assert!(own.is_err(), "a self dependency must be refused");

    // Nothing was recorded by the refused registrations.
    let after = runtime.replay().expect("replay");
    assert_eq!(after.revision, state.revision);
    assert!(!after.phases[PHASE].changes.contains_key("change-d"));
    assert!(!after.phases[PHASE].changes.contains_key("change-e"));
}

#[test]
fn stated_intent_does_not_reorder_adopted_work() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("outcome-integration", "integration-test");
    let state = seeded(&runtime, &actor, true);

    // Naming later work as intent is not a signed amendment: ledger order holds.
    runtime
        .set_active_path(
            actor.clone(),
            context(&state, "state-intent-for-change-b"),
            ActivePath {
                phase_path: vec![PHASE.into()],
                phase_id: Some(PHASE.into()),
                ..ActivePath::default()
            },
            Some("do change-b first".into()),
        )
        .expect("record operator intent");
    runtime
        .write_compatibility_projections()
        .expect("write projections");
    let waypoint = read_json(fixture.path(), ".kbd-orchestrator/current-waypoint.json");
    assert_eq!(waypoint["exactNextCommand"], "do change-b first");
    assert_eq!(waypoint["nextChange"], "change-a");
    assert_eq!(waypoint["nextAuthorizedAction"], "change change-a, task 1");
}
