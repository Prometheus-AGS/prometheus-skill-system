use chrono::Utc;
use kbd_runtime::{
    Actor, ActorKind, CausalFrontier, DeviceRecord, DeviceSigner, DeviceStatus, Event, EventKind,
    MigrationProvenance, Phase, Runtime, WorkStatus, EVENT_SCHEMA_VERSION,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    project: PathBuf,
    data: PathBuf,
    operator_key: PathBuf,
    operator: DeviceSigner,
    runtime: Runtime,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        let data = temp.path().join("data");
        fs::create_dir_all(project.join(".kbd-orchestrator")).unwrap();
        let key_runtime = Runtime::open(temp.path().join("operator-key"));
        let operator = key_runtime.device_signer().unwrap();
        let operator_key = key_runtime.runtime_root().join("device-key.json");
        let runtime = Runtime::open_canonical_at(&project, &data).unwrap();
        Self {
            _temp: temp,
            project,
            data,
            operator_key,
            operator,
            runtime,
        }
    }

    fn command(&self, args: &[&str]) -> Output {
        let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
        Command::new(binary)
            .current_dir(&self.project)
            .env("PROMETHEUS_DATA_DIR", &self.data)
            .env("PROMETHEUS_DEVICE_KEY_FILE", &self.operator_key)
            .env("PROMETHEUS_CONTROL_ENDPOINT", "http://127.0.0.1:1")
            .env("PROMETHEUS_HARNESS", "kbd-conflict-integration")
            .args(["kbd", "--path", self.project.to_str().unwrap()])
            .args(args)
            .output()
            .unwrap()
    }
}

fn signed_event(
    project_id: &str,
    replica_id: &str,
    event_id: &str,
    frontier: CausalFrontier,
    actor: Actor,
    kind: EventKind,
    signer: &DeviceSigner,
) -> Event {
    let mut event = Event {
        schema_version: EVENT_SCHEMA_VERSION.into(),
        project_id: project_id.into(),
        replica_id: replica_id.into(),
        run_id: "run-a".into(),
        event_id: event_id.into(),
        command_id: Some(format!("command-{event_id}")),
        revision: frontier.derived_revision().saturating_add(1),
        expected_revision: frontier.derived_revision(),
        lamport: frontier.next_lamport(replica_id),
        frontier,
        causal_parent: None,
        actor_id: actor.id.clone(),
        actor,
        timestamp: Utc::now(),
        kind,
        previous_hash: None,
        migration_provenance: None::<MigrationProvenance>,
        integrity_hash: String::new(),
        signer_key_id: None,
        signer_public_key: None,
        signature: None,
    };
    let bytes = event
        .prepare_host_signature(signer.key_id(), signer.public_key())
        .unwrap();
    event
        .attach_host_signature(signer.sign_base64(&bytes))
        .unwrap();
    event
}

fn phase(title: &str) -> Phase {
    Phase {
        id: "phase-1".into(),
        slug: "phase-1".into(),
        title: title.into(),
        parent_phase_id: None,
        status: WorkStatus::Pending,
        stages: BTreeMap::new(),
        changes: BTreeMap::new(),
        legacy_completion_baseline: None,
        legacy_read_only: false,
    }
}

fn advance(mut frontier: CausalFrontier, event: &Event) -> CausalFrontier {
    frontier.advance(event.replica_id.clone(), event.lamport);
    frontier
}

fn harness(id: &str) -> Actor {
    Actor {
        kind: ActorKind::Harness,
        id: id.into(),
        device: format!("device-{id}"),
        harness: "integration".into(),
        session: format!("session-{id}"),
    }
}

fn require_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_resolution_changes_both_conflict_metadata_and_projected_state_after_restart() {
    let fixture = Fixture::new();
    let operator = fixture.operator.clone();
    let branch_b = DeviceSigner::generate();
    let project_id = fixture
        .runtime
        .project_manifest(false)
        .unwrap()
        .unwrap()
        .project_id;

    let genesis = signed_event(
        &project_id,
        "origin",
        "genesis",
        CausalFrontier::empty(),
        Actor::operator("operator", "integration"),
        EventKind::RunInitialized {
            initial_state: kbd_runtime::LifecycleState::Ready,
            exact_next_work: None,
            plan_revision: 1,
            previous_run_id: None,
            reason: None,
        },
        &operator,
    );
    let after_genesis = advance(CausalFrontier::empty(), &genesis);
    let enrollment = signed_event(
        &project_id,
        "origin",
        "enroll-b",
        after_genesis,
        Actor::operator("operator", "integration"),
        EventKind::DeviceEnrolled {
            device: DeviceRecord {
                device_id: "device-b".into(),
                key_id: branch_b.key_id().into(),
                public_key: branch_b.public_key().into(),
                status: DeviceStatus::Active,
                enrolled_at_revision: 2,
                revoked_at_revision: None,
            },
        },
        &operator,
    );
    let branch_frontier = advance(advance(CausalFrontier::empty(), &genesis), &enrollment);
    let event_a = signed_event(
        &project_id,
        "replica-a",
        "event-a",
        branch_frontier.clone(),
        harness("a"),
        EventKind::PhaseDefined {
            phase: phase("candidate A"),
        },
        &operator,
    );
    let event_b = signed_event(
        &project_id,
        "replica-b",
        "event-b",
        branch_frontier,
        harness("b"),
        EventKind::PhaseDefined {
            phase: phase("candidate B"),
        },
        &branch_b,
    );
    fixture
        .runtime
        .project_document()
        .unwrap()
        .ingest_events(&[genesis, enrollment, event_a, event_b])
        .unwrap();

    let initial = fixture.runtime.replay_authority().unwrap();
    assert_eq!(initial.phases["phase-1"].title, "candidate B");
    let conflict = initial.conflicts.values().next().unwrap().clone();

    let listed = fixture.command(&["conflicts", "--json"]);
    require_success(&listed);
    let listed: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed[&conflict.id]["winnerEventId"], "event-b");

    require_success(&fixture.command(&[
        "resolve",
        &conflict.id,
        "--winner",
        "event-a",
        "--reason",
        "operator selected candidate A",
    ]));

    let reopened =
        Runtime::open_registered_at(&fixture.project, &fixture.data, &project_id).unwrap();
    let resolved = reopened.replay_authority().unwrap();
    assert_eq!(resolved.phases["phase-1"].title, "candidate A");
    assert_eq!(resolved.conflicts[&conflict.id].winner_event_id, "event-a");
    assert!(resolved.conflicts[&conflict.id]
        .resolved_by_event_id
        .is_some());
    require_success(&fixture.command(&["status", "--json"]));
    require_success(&fixture.command(&["conflicts", "--json"]));
}
