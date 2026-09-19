use kbd_runtime::{ActivePath, Actor, Change, MutationContext, Phase, Runtime, Task, WorkStatus};
use std::collections::BTreeMap;
use std::fs;

fn context(state: &kbd_runtime::RuntimeState, command_id: &str) -> MutationContext {
    MutationContext {
        expected_revision: state.revision,
        command_id: command_id.into(),
    }
}

#[test]
fn replay_and_projections_advance_from_task_state_without_rewriting_operator_intent() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("position-integration", "integration-test");
    let mut state = runtime
        .initialize("position-project", "position-run", actor.clone())
        .expect("initialize runtime");
    state = runtime
        .define_phase(
            actor.clone(),
            context(&state, "define-position-phase"),
            Phase {
                id: "position-phase".into(),
                slug: "position-phase".into(),
                title: "Position phase".into(),
                parent_phase_id: None,
                status: WorkStatus::InProgress,
                stages: BTreeMap::new(),
                changes: BTreeMap::new(),
                legacy_completion_baseline: None,
                legacy_read_only: false,
            },
        )
        .expect("define phase");

    for (sequence, change_id) in [(1, "change-a"), (2, "change-b")] {
        state = runtime
            .register_change(
                actor.clone(),
                context(&state, &format!("register-{change_id}")),
                "position-phase",
                Change {
                    id: change_id.into(),
                    title: format!("Change {change_id}"),
                    sequence,
                    status: WorkStatus::Pending,
                    implementation_status: WorkStatus::Pending,
                    tasks: BTreeMap::new(),
                },
            )
            .expect("register change");
        state = runtime
            .register_task(
                actor.clone(),
                context(&state, &format!("register-{change_id}-task")),
                "position-phase",
                change_id,
                Task {
                    id: "1".into(),
                    title: "Repeated task".into(),
                    sequence: 1,
                    status: WorkStatus::Pending,
                    summary: None,
                },
            )
            .expect("register task");
    }
    state = runtime
        .set_active_path(
            actor.clone(),
            context(&state, "activate-position-phase"),
            ActivePath {
                phase_path: vec!["position-phase".into()],
                phase_id: Some("position-phase".into()),
                ..ActivePath::default()
            },
            Some("/kbd-apply change-a".into()),
        )
        .expect("activate phase");

    for (status, command) in [
        (WorkStatus::InProgress, "start-change-a-task"),
        (WorkStatus::Complete, "complete-change-a-task"),
    ] {
        state = runtime
            .transition_task(
                actor.clone(),
                context(&state, command),
                "position-phase",
                "change-a",
                "1",
                status,
                None,
            )
            .expect("transition first task");
    }

    assert_eq!(state.active_path.change_id.as_deref(), Some("change-b"));
    assert_eq!(state.active_path.task_id.as_deref(), Some("1"));
    assert_eq!(
        state.exact_next_work.as_deref(),
        Some("/kbd-apply change-a")
    );
    assert_eq!(
        state.phases["position-phase"].changes["change-a"].implementation_status,
        WorkStatus::Complete
    );

    state = runtime
        .set_active_path(
            actor,
            context(&state, "reject-stale-terminal-cursor"),
            ActivePath {
                phase_path: vec!["position-phase".into()],
                phase_id: Some("position-phase".into()),
                change_id: Some("change-a".into()),
                task_id: Some("1".into()),
                ..ActivePath::default()
            },
            Some("/kbd-apply change-a".into()),
        )
        .expect("normalize stale terminal cursor");
    assert_eq!(state.active_path.change_id.as_deref(), Some("change-b"));
    assert_eq!(state.active_path.task_id.as_deref(), Some("1"));

    let reopened = Runtime::open(fixture.path());
    let replayed = reopened.replay().expect("replay runtime after restart");
    assert_eq!(replayed.active_path, state.active_path);
    assert_eq!(replayed.exact_next_work, state.exact_next_work);
    reopened
        .write_compatibility_projections()
        .expect("write compatibility projections");

    let waypoint: serde_json::Value = serde_json::from_slice(
        &fs::read(
            fixture
                .path()
                .join(".kbd-orchestrator/current-waypoint.json"),
        )
        .expect("read waypoint"),
    )
    .expect("parse waypoint");
    assert_eq!(waypoint["nextChange"], "change-b");
    assert_eq!(waypoint["nextTask"], "1");
    assert_eq!(waypoint["change"], "change-b");
    assert_eq!(waypoint["currentTask"], "1");
    assert_eq!(waypoint["exactNextCommand"], "/kbd-apply change-a");

    let reminder = fs::read_to_string(
        fixture
            .path()
            .join(".kbd-orchestrator/position-reminder.txt"),
    )
    .expect("read position reminder");
    assert!(reminder.contains("Next work: change change-b, task 1"));
    assert!(reminder.contains("NOT a work selector"));
}
