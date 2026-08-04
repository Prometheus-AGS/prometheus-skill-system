use super::*;
use std::env;
use std::fs;
use std::process::Command as ProcessCommand;
use std::thread;
use std::time::{Duration, Instant};

fn required_path(name: &str) -> PathBuf {
    env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{name} must be set for the ignored certification proof"))
}

fn certification_runtime() -> Runtime {
    let project = required_path("KBD_CERT_PROJECT_ROOT");
    let data = required_path("KBD_CERT_DATA_ROOT");
    match read_project_manifest(&project).unwrap() {
        Some(manifest) => Runtime::open_registered_at(&project, &data, &manifest.project_id)
            .or_else(|_| Runtime::open_canonical_at(&project, &data))
            .unwrap(),
        None => Runtime::open_canonical_at(&project, &data).unwrap(),
    }
}

fn certification_actor() -> Actor {
    Actor {
        kind: ActorKind::Operator,
        id: "startup-isolation-certifier".into(),
        device: "local-certification-device".into(),
        harness: "kbd-runtime-live-proof".into(),
        session: "startup-isolation".into(),
    }
}

#[test]
#[ignore = "operator proof requiring KBD_CERT_PROJECT_ROOT and KBD_CERT_DATA_ROOT"]
fn prepare_disposable_project_and_signed_lifecycle_command() {
    let output = required_path("KBD_CERT_SIGNED_COMMAND_OUTPUT");
    assert!(
        !output.exists(),
        "refusing to replace existing proof output"
    );
    let runtime = certification_runtime();
    let manifest = runtime.project_manifest(false).unwrap().unwrap();
    let state = runtime.replay_authority().unwrap();
    let state = if state.revision == 0 {
        runtime
            .initialize(
                manifest.project_id,
                "startup-isolation-certification",
                certification_actor(),
            )
            .unwrap()
    } else {
        state
    };
    assert_eq!(state.revision, 1, "disposable proof must start at genesis");
    let signed = SignedCommandEnvelope::sign(
        CommandEnvelope {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            command_id: "certification-lifecycle-running".into(),
            frontier: Some(state.frontier.clone()),
            expected_revision: state.revision,
            actor: certification_actor(),
            command: CommandKind::LifecycleTransition {
                to: LifecycleState::Running,
                reason: "startup isolation live certification".into(),
            },
        },
        &runtime.device_signer().unwrap(),
    )
    .unwrap();
    fs::write(output, serde_json::to_vec_pretty(&signed).unwrap()).unwrap();
}

#[test]
#[ignore = "operator proof requiring KBD_CERT_PROJECT_ROOT and KBD_CERT_DATA_ROOT"]
fn export_disposable_audit_without_touching_the_worktree() {
    let output = required_path("KBD_CERT_AUDIT_OUTPUT");
    assert!(
        !output.exists(),
        "refusing to replace existing proof output"
    );
    let runtime = certification_runtime();
    let status_before = git_stdout(runtime.project_root(), &["status", "--porcelain=v1"]);
    let export = runtime.export_audit_to_git().unwrap();
    let status_after = git_stdout(runtime.project_root(), &["status", "--porcelain=v1"]);
    assert_eq!(
        status_after, status_before,
        "audit export mutated the worktree"
    );
    fs::write(output, serde_json::to_vec_pretty(&export).unwrap()).unwrap();
}

#[test]
#[ignore = "operator proof requiring KBD_CERT_PROJECT_ROOT and KBD_CERT_DATA_ROOT"]
fn sigkill_leaves_fsynced_event_for_service_startup_reconciliation() {
    let marker = required_path("KBD_CERT_CRASH_MARKER");
    let command_id = "certification-fsynced-before-sigkill";

    if env::var_os("KBD_CERT_CRASH_CHILD").is_some() {
        let event: Event =
            serde_json::from_slice(&fs::read(required_path("KBD_CERT_CRASH_EVENT")).unwrap())
                .unwrap();
        let mut journal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(required_path("KBD_CERT_CRASH_JOURNAL"))
            .unwrap();
        serde_json::to_writer(&mut journal, &event).unwrap();
        journal.write_all(b"\n").unwrap();
        journal.sync_data().unwrap();
        File::open(required_path("KBD_CERT_CRASH_JOURNAL_ROOT"))
            .unwrap()
            .sync_all()
            .unwrap();
        fs::write(
            &marker,
            b"journal fsynced; Loro import intentionally pending\n",
        )
        .unwrap();
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }

    assert!(
        !marker.exists(),
        "refusing to replace existing crash marker"
    );
    let event_path = marker.with_extension("event.json");
    assert!(
        !event_path.exists(),
        "refusing to replace existing crash event"
    );
    let runtime = certification_runtime();
    let state = runtime.replay_authority().unwrap();
    let event = runtime
        .prepare_signed_command(
            &state,
            CommandEnvelope {
                schema_version: EVENT_SCHEMA_VERSION.into(),
                project_id: state.project_id.clone(),
                run_id: state.run_id.clone(),
                command_id: command_id.into(),
                frontier: Some(state.frontier.clone()),
                expected_revision: state.revision,
                actor: certification_actor(),
                command: CommandKind::PlanRevise {
                    reason: "prove journal-first crash recovery".into(),
                    exact_next_work: Some("restart and reconcile Loro".into()),
                },
            },
        )
        .unwrap();
    fs::write(&event_path, serde_json::to_vec_pretty(&event).unwrap()).unwrap();
    let journal_count = runtime.replica_events().unwrap().len();
    let document_count = runtime.events().unwrap().len();
    assert_eq!(journal_count, document_count);
    let current_exe = env::current_exe().unwrap();
    let test_name =
        "live_certification_proof::sigkill_leaves_fsynced_event_for_service_startup_reconciliation";
    let mut child = ProcessCommand::new(current_exe)
        .args(["--exact", test_name, "--ignored", "--nocapture"])
        .env("KBD_CERT_CRASH_CHILD", "1")
        .env("KBD_CERT_CRASH_EVENT", &event_path)
        .env("KBD_CERT_CRASH_JOURNAL", runtime.events_path())
        .env("KBD_CERT_CRASH_JOURNAL_ROOT", runtime.journal_root())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !marker.exists() && Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("crash writer exited before fsync marker: {status}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    if !marker.exists() {
        child.kill().unwrap();
        let _ = child.wait();
        panic!("child did not fsync the journal in time");
    }
    child.kill().unwrap();
    let killed = child.wait().unwrap();
    assert!(!killed.success());

    assert_eq!(runtime.replica_events().unwrap().len(), journal_count + 1);
    assert_eq!(runtime.events().unwrap().len(), document_count);
    assert!(runtime
        .replica_events()
        .unwrap()
        .iter()
        .any(|event| event.command_id.as_deref() == Some(command_id)));
}

#[test]
#[ignore = "operator proof requiring KBD_CERT_PROJECT_ROOT and KBD_CERT_DATA_ROOT"]
fn inject_torn_tail_for_service_startup_recovery() {
    let marker = required_path("KBD_CERT_TORN_MARKER");
    assert!(
        !marker.exists(),
        "refusing to replace existing torn-tail marker"
    );
    let journal_path = required_path("KBD_CERT_TORN_JOURNAL");
    let journal_root = required_path("KBD_CERT_TORN_JOURNAL_ROOT");
    let torn = br#"{"schemaVersion":"2","eventId":"certification-interrupted"#;
    let mut journal = OpenOptions::new().append(true).open(&journal_path).unwrap();
    journal.write_all(torn).unwrap();
    journal.sync_data().unwrap();
    File::open(journal_root).unwrap().sync_all().unwrap();
    let torn_sha256 = Sha256::digest(torn);
    fs::write(
        marker,
        format!("{}\n{torn_sha256:x}\n", journal_path.display()),
    )
    .unwrap();
}
