use kbd_runtime::{EventKind, Runtime, RuntimeError, WorkStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct KbdFixture {
    _temp: TempDir,
    project_root: PathBuf,
    data_root: PathBuf,
    device_key: PathBuf,
    runtime: Runtime,
}

impl KbdFixture {
    fn registered_but_uninitialized() -> Self {
        let temp = tempfile::tempdir().expect("create fixture root");
        let project_root = temp.path().join("project");
        let data_root = temp.path().join("data");
        let phase_root = project_root.join(".kbd-orchestrator/phases/legacy-phase");
        fs::create_dir_all(&phase_root).expect("create legacy phase directory");
        fs::write(
            project_root.join(".kbd-orchestrator/current-waypoint.json"),
            r#"{
                "phase":"legacy-phase",
                "status":"execute_ready",
                "planRevision":7,
                "exactNextCommand":"prometheus kbd stage enter --phase legacy-phase --id assess"
            }"#,
        )
        .expect("write legacy waypoint");
        fs::write(
            phase_root.join("progress.json"),
            r#"{
                "phase":"legacy-phase",
                "title":"Legacy phase",
                "changes":[
                    {"id":"change-1","title":"First change","status":"PENDING"}
                ]
            }"#,
        )
        .expect("write legacy phase progress");

        let key_runtime = Runtime::open(temp.path().join("key-runtime"));
        key_runtime
            .device_signer()
            .expect("create isolated signing key");
        let device_key = key_runtime.runtime_root().join("device-key.json");

        let runtime = Runtime::open_canonical_at(&project_root, &data_root)
            .expect("register canonical project without initializing a run");
        assert!(runtime_is_uninitialized(&runtime));

        Self {
            _temp: temp,
            project_root,
            data_root,
            device_key,
            runtime,
        }
    }

    fn registered_without_legacy_phase() -> Self {
        let temp = tempfile::tempdir().expect("create fixture root");
        let project_root = temp.path().join("project");
        let data_root = temp.path().join("data");
        fs::create_dir_all(project_root.join(".kbd-orchestrator"))
            .expect("create empty KBD directory");

        let key_runtime = Runtime::open(temp.path().join("key-runtime"));
        key_runtime
            .device_signer()
            .expect("create isolated signing key");
        let device_key = key_runtime.runtime_root().join("device-key.json");

        let runtime = Runtime::open_canonical_at(&project_root, &data_root)
            .expect("register canonical project without initializing a run");
        assert!(runtime_is_uninitialized(&runtime));

        Self {
            _temp: temp,
            project_root,
            data_root,
            device_key,
            runtime,
        }
    }

    fn command(&self, args: &[&str]) -> Output {
        let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
        Command::new(binary)
            .current_dir(&self.project_root)
            .env("PROMETHEUS_DATA_DIR", &self.data_root)
            .env("PROMETHEUS_DEVICE_KEY_FILE", &self.device_key)
            .env("PROMETHEUS_CONTROL_ENDPOINT", "http://127.0.0.1:1")
            .env("PROMETHEUS_HARNESS", "cli-integration-test")
            .args(["kbd", "--path", path_str(&self.project_root)])
            .args(args)
            .output()
            .expect("run prometheus kbd command")
    }
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("fixture path must be UTF-8")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn runtime_is_uninitialized(runtime: &Runtime) -> bool {
    matches!(runtime.replay(), Ok(state) if state.revision == 0)
        || matches!(runtime.replay(), Err(RuntimeError::NotInitialized))
}

#[test]
fn first_typed_mutation_initializes_and_imports_registered_legacy_project_once() {
    let fixture = KbdFixture::registered_but_uninitialized();

    let human_status = fixture.command(&["status"]);
    assert!(human_status.status.success(), "{}", stderr(&human_status));
    assert!(stdout(&human_status).contains("first typed mutation initializes automatically"));
    assert!(!stdout(&human_status).contains("migrate --apply"));

    let json_status = fixture.command(&["status", "--json"]);
    assert!(json_status.status.success(), "{}", stderr(&json_status));
    let status: serde_json::Value =
        serde_json::from_slice(&json_status.stdout).expect("parse status JSON");
    assert_eq!(status["runtimeInitialized"], false);
    assert_eq!(status["initializationRequired"], true);
    assert_eq!(
        status["runtimePath"].as_str(),
        Some(path_str(fixture.runtime.runtime_root()))
    );
    assert!(runtime_is_uninitialized(&fixture.runtime));

    let first = fixture.command(&[
        "stage",
        "enter",
        "--command-id",
        "first-stage-enter",
        "--phase",
        "legacy-phase",
        "--id",
        "assess",
        "--title",
        "Assess",
        "--sequence",
        "1",
    ]);
    assert!(
        first.status.success(),
        "first mutation failed\nstdout:\n{}\nstderr:\n{}",
        stdout(&first),
        stderr(&first)
    );

    let after_first = fixture.runtime.replay().expect("replay first mutation");
    assert_eq!(after_first.revision, 3);
    assert_eq!(after_first.plan_revision, 7);
    assert_eq!(
        after_first.exact_next_work.as_deref(),
        Some("prometheus kbd stage enter --phase legacy-phase --id assess")
    );
    assert!(after_first.run_id.starts_with("legacy-phase-"));
    assert_eq!(
        after_first.phases["legacy-phase"].stages["assess"].status,
        WorkStatus::InProgress
    );

    let first_run_id = after_first.run_id.clone();
    let second = fixture.command(&[
        "stage",
        "transition",
        "--command-id",
        "complete-assess",
        "--phase",
        "legacy-phase",
        "--id",
        "assess",
        "--status",
        "complete",
    ]);
    assert!(second.status.success(), "{}", stderr(&second));

    let after_second = fixture.runtime.replay().expect("replay second mutation");
    assert_eq!(after_second.revision, 4);
    assert_eq!(after_second.run_id, first_run_id);
    assert_eq!(
        fixture
            .runtime
            .events()
            .expect("read events")
            .iter()
            .filter(|event| matches!(event.kind, EventKind::RunInitialized { .. }))
            .count(),
        1
    );

    let rejected = fixture.command(&[
        "stage",
        "enter",
        "--command-id",
        "duplicate-stage-enter",
        "--phase",
        "legacy-phase",
        "--id",
        "assess",
        "--title",
        "Assess again",
        "--sequence",
        "2",
    ]);
    assert!(
        !rejected.status.success(),
        "rejected typed mutation must exit nonzero; stdout:\n{}",
        stdout(&rejected)
    );
    assert!(!stdout(&rejected).contains("committedLocally"));
    let after_rejection = fixture.runtime.replay().expect("replay after rejection");
    assert_eq!(after_rejection.revision, 4);
    assert!(!after_rejection
        .command_revisions
        .contains_key("duplicate-stage-enter"));
}

#[test]
fn initialization_failure_names_the_canonical_runtime_path() {
    let fixture = KbdFixture::registered_but_uninitialized();
    fs::remove_file(&fixture.device_key).expect("remove isolated signing key");

    let failed = fixture.command(&[
        "stage",
        "enter",
        "--command-id",
        "missing-key-stage-enter",
        "--phase",
        "legacy-phase",
        "--id",
        "assess",
        "--title",
        "Assess",
    ]);

    assert!(!failed.status.success());
    assert!(
        stderr(&failed).contains(path_str(fixture.runtime.runtime_root())),
        "initialization failure omitted runtime path\nstderr:\n{}",
        stderr(&failed)
    );
    assert!(runtime_is_uninitialized(&fixture.runtime));
}

#[test]
fn first_typed_mutation_without_legacy_phase_creates_one_ready_run() {
    let fixture = KbdFixture::registered_without_legacy_phase();

    let first = fixture.command(&[
        "phase",
        "create",
        "--command-id",
        "first-phase-create",
        "--id",
        "first-phase",
        "--title",
        "First phase",
    ]);

    assert!(
        first.status.success(),
        "first mutation failed\nstdout:\n{}\nstderr:\n{}",
        stdout(&first),
        stderr(&first)
    );
    let state = fixture.runtime.replay().expect("replay first mutation");
    assert_eq!(state.revision, 2);
    assert_eq!(state.lifecycle, kbd_runtime::LifecycleState::Ready);
    assert!(state.phases.contains_key("first-phase"));
    assert_eq!(
        fixture
            .runtime
            .events()
            .expect("read events")
            .iter()
            .filter(|event| matches!(event.kind, EventKind::RunInitialized { .. }))
            .count(),
        1
    );
}
