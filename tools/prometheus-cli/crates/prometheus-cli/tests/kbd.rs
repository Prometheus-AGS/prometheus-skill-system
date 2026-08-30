use kbd_runtime::{registry::ProjectRegistry, EventKind, Runtime, RuntimeError, WorkStatus};
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
fn projects_prune_missing_enforces_apply_authority_and_reports_human_and_json_evidence() {
    let fixture = KbdFixture::registered_without_legacy_phase();
    let stale_checkout = fixture._temp.path().join("stale-checkout");
    fs::create_dir_all(stale_checkout.join(".prometheus"))
        .expect("create stale checkout manifest directory");
    fs::copy(
        fixture.project_root.join(".prometheus/project.json"),
        stale_checkout.join(".prometheus/project.json"),
    )
    .expect("copy declared project identity");

    let registry = ProjectRegistry::open_at(&fixture.data_root);
    let stale = registry
        .register_existing(&stale_checkout)
        .expect("register checkout that will become stale");
    let registry_before = fs::read(registry.registry_path()).expect("read registry before prune");
    fs::remove_dir_all(&stale_checkout).expect("remove registered checkout");

    let invalid_apply = fixture.command(&["projects", "--apply"]);
    assert!(!invalid_apply.status.success());
    assert!(stderr(&invalid_apply).contains("--prune-missing"));
    assert_eq!(
        fs::read(registry.registry_path()).expect("read registry after rejected apply"),
        registry_before
    );

    let human_dry_run = fixture.command(&["projects", "--prune-missing"]);
    assert!(human_dry_run.status.success(), "{}", stderr(&human_dry_run));
    let human_dry_run = stdout(&human_dry_run);
    assert!(human_dry_run.contains("Registry prune dry run"));
    assert!(human_dry_run.contains("Candidates: 1"));
    assert!(human_dry_run.contains(&stale.path));
    assert!(human_dry_run.contains("Removed: 0"));
    assert!(human_dry_run.contains("--prune-missing --apply"));
    assert_eq!(
        fs::read(registry.registry_path()).expect("read registry after human dry run"),
        registry_before
    );

    let json_dry_run = fixture.command(&["projects", "--prune-missing", "--json"]);
    assert!(json_dry_run.status.success(), "{}", stderr(&json_dry_run));
    let json_dry_run: serde_json::Value =
        serde_json::from_slice(&json_dry_run.stdout).expect("parse dry-run report");
    assert_eq!(json_dry_run["applyRequested"], false);
    assert_eq!(json_dry_run["applied"], false);
    assert_eq!(json_dry_run["candidates"][0]["path"], stale.path);
    assert_eq!(json_dry_run["removed"].as_array().unwrap().len(), 0);

    let json_apply = fixture.command(&["projects", "--prune-missing", "--apply", "--json"]);
    assert!(json_apply.status.success(), "{}", stderr(&json_apply));
    let json_apply: serde_json::Value =
        serde_json::from_slice(&json_apply.stdout).expect("parse apply report");
    assert_eq!(json_apply["applyRequested"], true);
    assert_eq!(json_apply["applied"], true);
    assert_eq!(json_apply["removed"][0]["path"], stale.path);
    for field in ["backupPath", "backupSha256", "checksumPath", "receiptPath"] {
        assert!(
            json_apply[field]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "missing apply evidence field {field}"
        );
    }
    assert!(Path::new(json_apply["backupPath"].as_str().unwrap()).is_file());
    assert!(Path::new(json_apply["checksumPath"].as_str().unwrap()).is_file());
    assert!(Path::new(json_apply["receiptPath"].as_str().unwrap()).is_file());

    let repeated_human_apply = fixture.command(&["projects", "--prune-missing", "--apply"]);
    assert!(
        repeated_human_apply.status.success(),
        "{}",
        stderr(&repeated_human_apply)
    );
    let repeated_human_apply = stdout(&repeated_human_apply);
    assert!(repeated_human_apply.contains("Registry prune apply"));
    assert!(repeated_human_apply.contains("Candidates: 0"));
    assert!(repeated_human_apply.contains("Removed: 0"));
    assert!(repeated_human_apply.contains("No registry changes were required."));
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
    assert!(
        !stderr(&first).contains("control plane"),
        "ordinary typed mutations must not probe or warn about the optional control plane: {}",
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
    assert!(
        !stderr(&second).contains("control plane"),
        "successive local mutations must remain daemon-free: {}",
        stderr(&second)
    );

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

fn require_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn create_receipt_fixture() -> KbdFixture {
    let fixture = KbdFixture::registered_without_legacy_phase();
    require_success(
        &fixture.command(&[
            "phase",
            "create",
            "--command-id",
            "receipt-phase-create",
            "--id",
            "receipt-phase",
            "--title",
            "Receipt phase",
        ]),
        "create phase",
    );
    require_success(
        &fixture.command(&[
            "phase",
            "activate",
            "--command-id",
            "receipt-phase-activate",
            "--id",
            "receipt-phase",
        ]),
        "activate phase",
    );
    require_success(
        &fixture.command(&[
            "phase",
            "transition",
            "--command-id",
            "receipt-phase-start",
            "--id",
            "receipt-phase",
            "--status",
            "in-progress",
        ]),
        "start phase",
    );
    require_success(
        &fixture.command(&[
            "change",
            "register",
            "--command-id",
            "receipt-change-register",
            "--phase",
            "receipt-phase",
            "--id",
            "receipt-change",
            "--title",
            "Receipt change",
            "--sequence",
            "1",
        ]),
        "register change",
    );
    require_success(
        &fixture.command(&[
            "task",
            "register",
            "--command-id",
            "receipt-task-register",
            "--phase",
            "receipt-phase",
            "--change",
            "receipt-change",
            "--id",
            "task-1",
            "--title",
            "Canonical integration task",
            "--sequence",
            "1",
        ]),
        "register task",
    );
    fixture
}

#[test]
fn boundary_receipts_projection_repair_and_gate_receipts_complete_end_to_end() {
    let fixture = create_receipt_fixture();

    let before_precommit = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "before",
        "--subject",
        "Canonical integration task",
        "--json",
        "--repair-projections",
        "--precommit",
    ]);
    require_success(&before_precommit, "precommit task start");
    let revision_before_start = fixture.runtime.replay().expect("replay precommit").revision;

    require_success(
        &fixture.command(&[
            "task",
            "transition",
            "--command-id",
            "receipt-task-start",
            "--phase",
            "receipt-phase",
            "--change",
            "receipt-change",
            "--id",
            "task-1",
            "--status",
            "in-progress",
        ]),
        "start task",
    );
    let before = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "before",
        "--subject",
        "Canonical integration task",
        "--json",
        "--repair-projections",
    ]);
    require_success(&before, "record task start receipt");
    let before_json: serde_json::Value =
        serde_json::from_slice(&before.stdout).expect("parse start receipt JSON");
    assert!(matches!(
        before_json["outcome"].as_str(),
        Some("pass" | "repaired")
    ));
    assert_eq!(
        before_json["exactSignal"],
        "Starting task 1 out of 1: Canonical integration task"
    );
    assert!(before_json["authoritativeRevision"].as_u64().unwrap() > revision_before_start);
    assert_eq!(
        fixture.runtime.replay().unwrap().boundary_obligations.len(),
        1
    );

    let after_precommit = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "after",
        "--subject",
        "Canonical integration task",
        "--json",
        "--repair-projections",
        "--precommit",
    ]);
    require_success(&after_precommit, "precommit task completion");
    require_success(
        &fixture.command(&[
            "task",
            "transition",
            "--command-id",
            "receipt-task-complete",
            "--phase",
            "receipt-phase",
            "--change",
            "receipt-change",
            "--id",
            "task-1",
            "--status",
            "complete",
        ]),
        "complete task",
    );

    fs::write(
        fixture
            .project_root
            .join(".kbd-orchestrator/current-waypoint.json"),
        "{}\n",
    )
    .expect("corrupt compatibility projection");
    let source_revision = fixture.runtime.replay().unwrap().revision;
    let after = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "after",
        "--subject",
        "Canonical integration task",
        "--json",
        "--repair-projections",
    ]);
    require_success(&after, "record completion receipt and repair projection");
    let after_json: serde_json::Value =
        serde_json::from_slice(&after.stdout).expect("parse completion receipt JSON");
    assert_eq!(after_json["outcome"], "repaired");
    assert_eq!(after_json["sourceRevision"], source_revision);
    assert_eq!(after_json["authoritativeRevision"], source_revision + 1);
    assert_eq!(
        after_json["exactSignal"],
        "Completed task 1 out of 1: Canonical integration task"
    );
    assert!(fixture
        .runtime
        .replay()
        .unwrap()
        .boundary_obligations
        .is_empty());

    require_success(
        &fixture.command(&[
            "gate",
            "run",
            "--kind",
            "integration",
            "--scope",
            "receipt-phase",
            "--",
            "/usr/bin/true",
        ]),
        "run integration gate",
    );
    require_success(
        &fixture.command(&[
            "gate",
            "run",
            "--kind",
            "certification",
            "--scope",
            "receipt-phase",
            "--",
            "/usr/bin/true",
        ]),
        "run certification gate",
    );
    let state = fixture.runtime.replay().expect("replay certified state");
    assert!(state.active_gates.is_empty());
    assert!(state.latest_gate_receipts.values().any(|receipt| {
        receipt.kind == kbd_runtime::GateKind::Integration
            && receipt.outcome == kbd_runtime::GateOutcome::Passed
    }));
    assert!(state.latest_gate_receipts.values().any(|receipt| {
        receipt.kind == kbd_runtime::GateKind::Certification
            && receipt.outcome == kbd_runtime::GateOutcome::Passed
    }));
}

#[test]
fn duplicate_missing_and_rust_contention_boundaries_fail_closed() {
    let fixture = create_receipt_fixture();
    require_success(
        &fixture.command(&[
            "task",
            "transition",
            "--command-id",
            "blocked-task-start",
            "--phase",
            "receipt-phase",
            "--change",
            "receipt-change",
            "--id",
            "task-1",
            "--status",
            "in-progress",
        ]),
        "start blocked task fixture",
    );
    let missing = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "after",
        "--subject",
        "task-1",
        "--json",
        "--repair-projections",
    ]);
    assert!(!missing.status.success());
    let after_missing = fixture.runtime.replay().expect("replay missing boundary");
    assert!(after_missing.boundary_obligations.is_empty());
    assert_eq!(
        after_missing.phases["receipt-phase"].changes["receipt-change"].tasks["task-1"].status,
        WorkStatus::InProgress
    );
    assert!(after_missing
        .blockers
        .values()
        .any(|blocker| !blocker.resolved));

    let before = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "before",
        "--subject",
        "task-1",
        "--json",
        "--repair-projections",
    ]);
    require_success(&before, "record first start boundary");
    let duplicate = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "before",
        "--subject",
        "task-1",
        "--json",
        "--repair-projections",
    ]);
    assert!(!duplicate.status.success());
    assert_eq!(
        fixture
            .runtime
            .replay()
            .expect("replay duplicate boundary")
            .boundary_obligations
            .len(),
        1
    );

    let rust_gate = fixture.command(&[
        "gate",
        "run",
        "--kind",
        "compiler-check",
        "--scope",
        "receipt-change",
        "--",
        "cargo",
        "--version",
    ]);
    assert!(!rust_gate.status.success());
    assert!(stdout(&rust_gate).contains("another Cargo/rustc process is active"));
    let state = fixture.runtime.replay().expect("replay blocked Rust gate");
    assert!(state.active_gates.is_empty());
    assert!(state.latest_gate_receipts.values().any(|receipt| {
        receipt.kind == kbd_runtime::GateKind::CompilerCheck
            && receipt.outcome == kbd_runtime::GateOutcome::Blocked
    }));
}

#[test]
fn ambiguous_signed_authority_writes_only_an_atomic_recovery_receipt() {
    let fixture = create_receipt_fixture();
    let journal_before = fs::read(fixture.runtime.events_path()).expect("read signed journal");
    let pointer: serde_json::Value = serde_json::from_slice(
        &fs::read(
            fixture
                .runtime
                .runtime_root()
                .join("checkpoints/current.json"),
        )
        .expect("read checkpoint pointer"),
    )
    .expect("parse checkpoint pointer");
    let checkpoint = pointer["checkpoint"].as_str().expect("checkpoint filename");
    fs::write(
        fixture
            .runtime
            .runtime_root()
            .join("checkpoints")
            .join(checkpoint),
        "{}\n",
    )
    .expect("corrupt signed folded checkpoint");

    let evaluation = fixture.command(&[
        "guard",
        "evaluate",
        "--boundary",
        "task",
        "--edge",
        "before",
        "--subject",
        "task-1",
        "--json",
        "--precommit",
        "--repair-projections",
    ]);
    assert!(!evaluation.status.success());
    assert_eq!(
        fs::read(fixture.runtime.events_path()).expect("reread signed journal"),
        journal_before,
        "ambiguous authority must not mutate the canonical journal"
    );

    let recovery_root = fixture
        .project_root
        .join(".kbd-orchestrator/recovery/bottleneck");
    let receipts = fs::read_dir(&recovery_root)
        .expect("recovery receipt directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("read recovery receipts");
    assert_eq!(receipts.len(), 1);
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(receipts[0].path()).expect("read recovery receipt"))
            .expect("parse recovery receipt");
    assert_eq!(receipt["outcome"], "blocked");
    assert_eq!(receipt["canonicalMutation"], false);
    assert!(fs::read_dir(recovery_root).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with('.')));
}
