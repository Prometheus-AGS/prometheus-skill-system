use kbd_runtime::{
    ActivePath, Actor, Change, Completion, CompletionDimension, LifecycleState, MutationContext,
    Phase, Runtime, RuntimeState, Stage, Task, WorkStatus,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn context(state: &RuntimeState, command_id: impl Into<String>) -> MutationContext {
    MutationContext {
        expected_revision: state.revision,
        command_id: command_id.into(),
    }
}

fn define_phase(
    runtime: &Runtime,
    actor: &Actor,
    state: &mut RuntimeState,
    id: &str,
    slug: &str,
    parent: Option<&str>,
) {
    *state = runtime
        .define_phase(
            actor.clone(),
            context(state, format!("define-{id}")),
            Phase {
                id: id.into(),
                slug: slug.into(),
                title: format!("Phase {slug}"),
                parent_phase_id: parent.map(str::to_owned),
                status: WorkStatus::InProgress,
                stages: BTreeMap::new(),
                changes: BTreeMap::new(),
                legacy_completion_baseline: None,
                legacy_read_only: false,
            },
        )
        .expect("define phase through journal");
}

fn register_change(
    runtime: &Runtime,
    actor: &Actor,
    state: &mut RuntimeState,
    phase: &str,
    id: &str,
    complete: bool,
) {
    *state = runtime
        .register_change(
            actor.clone(),
            context(state, format!("register-{phase}-{id}")),
            phase,
            Change {
                id: id.into(),
                title: format!("Change {id}"),
                sequence: state.phases[phase].changes.len() as u64 + 1,
                status: WorkStatus::Pending,
                implementation_status: WorkStatus::Pending,
                tasks: BTreeMap::new(),
            },
        )
        .expect("register change through journal");
    *state = runtime
        .register_task(
            actor.clone(),
            context(state, format!("register-{phase}-{id}-task")),
            phase,
            id,
            Task {
                id: "1".into(),
                title: "Implement change".into(),
                sequence: 1,
                status: WorkStatus::Pending,
                summary: None,
            },
        )
        .expect("register task through journal");
    if complete {
        for (status, step) in [
            (WorkStatus::InProgress, "start"),
            (WorkStatus::Complete, "complete"),
        ] {
            *state = runtime
                .transition_task(
                    actor.clone(),
                    context(state, format!("{step}-{phase}-{id}")),
                    phase,
                    id,
                    "1",
                    status,
                    None,
                )
                .expect("transition task through journal");
        }
    }
}

fn read_json(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).expect("read projection"))
        .expect("parse projection")
}

fn assert_replay_and_projection_stability(root: &Path, runtime: &Runtime, paths: &[&str]) {
    let journal = fs::read(runtime.events_path()).expect("read authoritative journal");
    let state = runtime.replay_authority().expect("read authoritative state");
    runtime
        .write_compatibility_projections()
        .expect("project committed state");
    let expected: Vec<_> = paths
        .iter()
        .map(|path| fs::read(root.join(path)).expect("snapshot projection"))
        .collect();

    // The production writer uses the last committed event timestamp. Reopening
    // and projecting again must therefore preserve bytes without a clock mock.
    let reopened = Runtime::open(root);
    assert_eq!(reopened.replay_authority().expect("replay journal"), state);
    for _ in 0..2 {
        reopened
            .write_compatibility_projections()
            .expect("repeat projection after restart");
        for (path, bytes) in paths.iter().zip(&expected) {
            assert_eq!(
                fs::read(root.join(path)).expect("read repeated projection"),
                *bytes,
                "projection changed without a new event: {path}"
            );
        }
        assert_eq!(
            fs::read(reopened.events_path()).expect("read journal after projection"),
            journal
        );
        assert_eq!(reopened.replay_authority().expect("replay again"), state);
    }
}

#[test]
fn phase_completion_is_local_while_run_history_survives_new_phases_and_replay() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("phase-scope-integration", "integration-test");
    let mut state = runtime
        .initialize("scope-project", "scope-run", actor.clone())
        .expect("initialize runtime");
    define_phase(&runtime, &actor, &mut state, "prior-id", "prior", None);
    register_change(&runtime, &actor, &mut state, "prior-id", "old", true);

    let dimensions = [
        (CompletionDimension::Evidence, "evidence"),
        (CompletionDimension::Certification, "certification"),
        (CompletionDimension::Publication, "publication"),
    ];
    for (dimension, name) in dimensions {
        state = runtime
            .update_completion(
                actor.clone(),
                context(&state, format!("complete-run-{name}")),
                dimension,
                Completion {
                    completed: 1,
                    total: 1,
                    status: WorkStatus::Complete,
                    summary: Some(format!("Prior phase {name} complete")),
                    blockers: Vec::new(),
                },
            )
            .expect("record run completion through journal");
    }
    let prior_completion = state.completion.clone();
    define_phase(&runtime, &actor, &mut state, "new-id", "new", None);
    state = runtime
        .set_active_path(
            actor.clone(),
            context(&state, "activate-new-empty-phase"),
            ActivePath {
                phase_path: vec!["new-id".into()],
                phase_id: Some("new-id".into()),
                ..ActivePath::default()
            },
            None,
        )
        .expect("activate empty phase");
    assert_eq!(state.completion, prior_completion);
    runtime
        .write_compatibility_projections()
        .expect("project new empty phase");

    let progress_path = ".kbd-orchestrator/phases/new/progress.json";
    let empty = read_json(fixture.path(), progress_path);
    assert_eq!(empty["phaseId"], "new-id");
    assert_eq!(empty["changes"], json!([]));
    assert_eq!(empty["changes_completed"], 0);
    assert_eq!(empty["changes_total"], 0);
    assert_eq!(
        empty["completion"]["implementation"],
        json!({"completed": 0, "total": 0, "status": "PENDING"})
    );
    assert_eq!(
        empty["completionScopes"],
        json!({"completion": "phase", "runCompletion": "run"})
    );
    assert_eq!(empty["runCompletion"]["scope"], "run");
    assert_eq!(empty["runCompletion"]["runId"], "scope-run");
    assert_eq!(empty["runCompletion"]["primaryCounter"], "implementation");
    assert_eq!(
        empty["runCompletion"]["implementation"],
        json!({"completed": 1, "total": 1, "status": "COMPLETE"})
    );
    for (_, name) in dimensions {
        assert_eq!(
            empty["completion"][name],
            json!({"status": "NOT_TRACKED", "summary": null, "blockers": []})
        );
        assert_eq!(
            empty["runCompletion"][name],
            json!({"status": "COMPLETE", "summary": format!("Prior phase {name} complete"), "blockers": []})
        );
    }
    assert_replay_and_projection_stability(
        fixture.path(),
        &runtime,
        &[
            progress_path,
            ".kbd-orchestrator/phases/prior/progress.json",
            ".kbd-orchestrator/current-waypoint.json",
            ".kbd-orchestrator/position-reminder.txt",
        ],
    );

    register_change(&runtime, &actor, &mut state, "new-id", "done", true);
    register_change(&runtime, &actor, &mut state, "new-id", "pending", false);
    define_phase(
        &runtime,
        &actor,
        &mut state,
        "child-id",
        "child",
        Some("new-id"),
    );
    register_change(&runtime, &actor, &mut state, "child-id", "child-done", true);
    runtime
        .write_compatibility_projections()
        .expect("project separate phase counts");
    let local = read_json(fixture.path(), progress_path);
    assert_eq!(local["changes_completed"], 1);
    assert_eq!(local["changes_total"], 2);
    assert_eq!(local["changes"].as_array().expect("change rows").len(), 2);
    assert_eq!(
        local["completion"]["implementation"],
        json!({"completed": 1, "total": 2, "status": "IN_PROGRESS"})
    );
    assert_eq!(
        local["runCompletion"]["implementation"],
        json!({"completed": 3, "total": 4, "status": "IN_PROGRESS"})
    );
    for (dimension, name) in dimensions {
        assert_eq!(state.completion[&dimension], prior_completion[&dimension]);
        assert_eq!(local["completion"][name], empty["completion"][name]);
        assert_eq!(local["runCompletion"][name], empty["runCompletion"][name]);
    }
    assert_replay_and_projection_stability(
        fixture.path(),
        &runtime,
        &[
            progress_path,
            ".kbd-orchestrator/phases/new/tasks.md",
            ".kbd-orchestrator/phases/prior/progress.json",
            ".kbd-orchestrator/phases/new/children/child/progress.json",
            ".kbd-orchestrator/current-waypoint.json",
            ".kbd-orchestrator/position-reminder.txt",
        ],
    );
}

#[test]
fn waypoint_uses_immediate_parent_slug_and_distinguishes_stage_from_lifecycle() {
    let fixture = tempfile::tempdir().expect("create runtime fixture");
    let runtime = Runtime::open(fixture.path());
    let actor = Actor::operator("waypoint-scope-integration", "integration-test");
    let mut state = runtime
        .initialize("waypoint-project", "waypoint-run", actor.clone())
        .expect("initialize runtime");
    define_phase(&runtime, &actor, &mut state, "root-id", "root-slug", None);
    state = runtime
        .set_active_path(
            actor.clone(),
            context(&state, "activate-root"),
            ActivePath {
                phase_path: vec!["root-id".into()],
                phase_id: Some("root-id".into()),
                ..ActivePath::default()
            },
            None,
        )
        .expect("activate top-level phase");
    runtime
        .write_compatibility_projections()
        .expect("project root");
    let waypoint_path = ".kbd-orchestrator/current-waypoint.json";
    let root = read_json(fixture.path(), waypoint_path);
    assert_eq!(root["activePhase"], "root-slug");
    assert!(root["parentPhase"].is_null());
    assert!(root["stageId"].is_null());
    assert!(root["stageStatus"].is_null());

    define_phase(
        &runtime,
        &actor,
        &mut state,
        "middle-id",
        "middle-slug",
        Some("root-id"),
    );
    define_phase(
        &runtime,
        &actor,
        &mut state,
        "leaf-id",
        "leaf-slug",
        Some("middle-id"),
    );
    state = runtime
        .enter_stage(
            actor,
            context(&state, "enter-leaf-assess"),
            "leaf-id",
            Stage {
                id: "assess".into(),
                title: "Assess".into(),
                sequence: 1,
                status: WorkStatus::InProgress,
            },
        )
        .expect("enter stage through journal");
    assert_eq!(state.lifecycle, LifecycleState::Ready);
    runtime
        .write_compatibility_projections()
        .expect("project leaf");
    let leaf = read_json(fixture.path(), waypoint_path);
    assert_eq!(
        leaf["path"],
        json!(["root-slug", "middle-slug", "leaf-slug"])
    );
    assert_eq!(leaf["phaseIds"], json!(["root-id", "middle-id", "leaf-id"]));
    assert_eq!(leaf["activePhaseId"], "leaf-id");
    assert_eq!(leaf["activePhase"], "leaf-slug");
    assert_eq!(leaf["parentPhase"], "middle-slug");
    assert_eq!(leaf["stageId"], "assess");
    assert_eq!(leaf["stageStatus"], "IN_PROGRESS");
    assert_eq!(leaf["status"], "ready");
    assert_eq!(leaf["lifecycle"], "ready");
    let reminder_path = ".kbd-orchestrator/position-reminder.txt";
    let reminder = fs::read_to_string(fixture.path().join(reminder_path)).expect("read reminder");
    assert!(reminder.lines().any(|line| line == "Stage: assess"));
    assert!(reminder
        .lines()
        .any(|line| line == "Stage status: IN_PROGRESS"));
    assert!(reminder.lines().any(|line| line == "Lifecycle: ready"));
    assert_replay_and_projection_stability(
        fixture.path(),
        &runtime,
        &[
            waypoint_path,
            reminder_path,
            ".kbd-orchestrator/phases/root-slug/progress.json",
            ".kbd-orchestrator/phases/root-slug/children/middle-slug/progress.json",
            ".kbd-orchestrator/phases/root-slug/children/middle-slug/children/leaf-slug/progress.json",
        ],
    );
}
