use std::collections::BTreeSet;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("prometheus-cli-{label}-{nanos}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent directory");
    }
    fs::write(path, contents).expect("write file");
}

fn collect_paths(root: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            let relative = path
                .strip_prefix(root)
                .expect("relative path")
                .to_string_lossy()
                .replace('\\', "/");
            paths.insert(relative.clone());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }

    paths
}

fn prepared_environment(label: &str) -> (PathBuf, PathBuf) {
    let project_root = unique_temp_dir(&format!("{label}-project"));
    let home_dir = unique_temp_dir(&format!("{label}-home"));

    fs::create_dir_all(project_root.join("skills")).expect("create skills dir");
    fs::create_dir_all(project_root.join(".kbd-orchestrator")).expect("create kbd dir");
    fs::create_dir_all(home_dir.join(".claude")).expect("create faux claude dir");
    write_file(&project_root.join("Skills.toml"), "[skills]\n");

    (project_root, home_dir)
}

fn base_command(project_root: &Path, home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_prometheus"));
    command
        .current_dir(project_root)
        .env("HOME", home_dir)
        .env("SURREAL_MEMORY_URL", "http://127.0.0.1:9")
        .env_remove("CLAUDE_CODE_CONFIG")
        .env_remove("PROMETHEUS_HOME");
    command
}

#[test]
fn doctor_reports_unreachable_memory_as_failure() {
    let (project_root, home_dir) = prepared_environment("doctor-unreachable-memory");

    let output = base_command(&project_root, &home_dir)
        .arg("doctor")
        .output()
        .expect("run doctor");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !output.status.success(),
        "doctor should exit nonzero when required memory is unreachable; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Surreal-memory"),
        "doctor output should mention the memory check; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("unhealthy or not ready"),
        "doctor output should report the memory service as unhealthy or not ready; stdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("All checks passed"),
        "doctor must not false-green when required memory is unreachable; stdout:\n{stdout}"
    );
}

#[test]
fn doctor_json_mode_emits_versioned_schema() {
    let (project_root, home_dir) = prepared_environment("doctor-json");

    let output = base_command(&project_root, &home_dir)
        .args(["doctor", "--json"])
        .output()
        .expect("run doctor --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "doctor --json must be a supported surface; stderr:\n{stderr}"
    );
    let payload: serde_json::Value =
        serde_json::from_str(&stdout).expect("doctor --json should emit valid JSON");

    assert!(
        payload.get("schema_version").is_some(),
        "doctor --json must include a versioned schema root; payload: {payload}"
    );
    assert_eq!(
        payload
            .get("contractVersion")
            .and_then(|value| value.as_str()),
        Some("2.0.0"),
        "doctor --json must expose the stable control-plane contract"
    );
    assert!(
        payload.get("summary").is_some(),
        "doctor --json must include a summary object; payload: {payload}"
    );
    assert!(
        payload.get("checks").and_then(|v| v.as_array()).is_some(),
        "doctor --json must include a checks array; payload: {payload}"
    );
    assert!(
        !stdout.contains('\u{1b}'),
        "doctor --json must not contain ANSI escapes; payload: {stdout}"
    );
}

#[test]
fn doctor_json_reports_rotation_dependencies() {
    let (project_root, home_dir) = prepared_environment("doctor-rotation-dependencies");

    let output = base_command(&project_root, &home_dir)
        .args(["doctor", "--json"])
        .output()
        .expect("run doctor --json");
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor --json should emit valid JSON");
    let rotation = payload["checks"]
        .as_array()
        .expect("checks array")
        .iter()
        .find(|check| check["id"] == "hooks.rotation")
        .expect("rotation check");
    let details = rotation["details"].as_array().expect("rotation details");

    assert!(
        details.iter().any(|detail| detail
            .as_str()
            .is_some_and(|value| value.starts_with("logrotate: "))),
        "rotation check must report the configured logrotate dependency: {rotation}"
    );
    assert!(
        details.iter().any(|detail| detail
            .as_str()
            .is_some_and(|value| value.starts_with("flock: "))),
        "rotation check must report the configured flock dependency: {rotation}"
    );
}

#[test]
fn doctor_dry_run_fix_is_non_mutating() {
    let (project_root, home_dir) = prepared_environment("doctor-dry-run-fix");
    write_file(&project_root.join("before.txt"), "marker");
    let before = collect_paths(&project_root);

    let output = base_command(&project_root, &home_dir)
        .args(["doctor", "--dry-run", "--fix"])
        .output()
        .expect("run doctor --dry-run --fix");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "doctor --dry-run --fix must be a supported surface; stderr:\n{stderr}"
    );

    let after = collect_paths(&project_root);
    assert_eq!(before, after, "dry-run fix must not mutate the filesystem");
}

#[test]
fn doctor_dry_run_refresh_is_non_mutating() {
    let (project_root, home_dir) = prepared_environment("doctor-dry-run-refresh");
    write_file(&project_root.join("before.txt"), "marker");
    let before = collect_paths(&project_root);

    let output = base_command(&project_root, &home_dir)
        .args(["doctor", "--dry-run", "--refresh"])
        .output()
        .expect("run doctor --dry-run --refresh");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "doctor --dry-run --refresh must be a supported surface; stderr:\n{stderr}"
    );

    let after = collect_paths(&project_root);
    assert_eq!(
        before, after,
        "dry-run refresh must not mutate the filesystem"
    );
}

#[test]
fn doctor_refresh_json_emits_scoped_repair_plan() {
    let (project_root, home_dir) = prepared_environment("doctor-refresh-plan");

    let output = base_command(&project_root, &home_dir)
        .args(["doctor", "--json", "--refresh"])
        .output()
        .expect("run doctor --json --refresh");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload: serde_json::Value =
        serde_json::from_str(&stdout).expect("doctor --json --refresh should emit valid JSON");

    let repair_plan = payload
        .get("repair_plan")
        .expect("repair plan should be present for refresh mode");
    let safe_actions = repair_plan
        .get("safe_actions")
        .and_then(|value| value.as_array())
        .expect("repair plan should contain safe actions");
    let manual_actions = repair_plan
        .get("manual_actions")
        .and_then(|value| value.as_array())
        .expect("repair plan should contain manual actions");

    assert!(
        safe_actions.iter().any(|action| {
            action.get("id").and_then(|value| value.as_str())
                == Some("services.install-mcp-services")
        }),
        "refresh plan should surface the managed services repair action; payload: {payload}"
    );
    assert!(
        manual_actions.iter().any(|action| {
            action.get("id").and_then(|value| value.as_str()) == Some("manual.review-hooks")
        }),
        "refresh plan should preserve manual-only boundaries; payload: {payload}"
    );
}

#[test]
fn doctor_refresh_requires_yes_or_dry_run_for_mutation() {
    let (project_root, home_dir) = prepared_environment("doctor-refresh-confirmation");
    let before = collect_paths(&project_root);

    let output = base_command(&project_root, &home_dir)
        .args(["doctor", "--refresh"])
        .output()
        .expect("run doctor --refresh");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "doctor --refresh should not proceed without confirmation; stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("Confirmation required"),
        "doctor --refresh should explain the deny-by-default confirmation boundary; stdout:\n{stdout}"
    );

    let after = collect_paths(&project_root);
    assert_eq!(
        before, after,
        "refresh without --yes must not mutate the filesystem"
    );
}

#[test]
fn doctor_exclusions_are_applied_before_kbd_checks_execute() {
    let (project_root, home_dir) = prepared_environment("doctor-lazy-exclusions");
    write_file(
        &project_root.join(".prometheus/project.json"),
        r#"{"schemaVersion":"1","projectId":"00000000-0000-4000-8000-000000000001","repositoryFingerprint":"sha256:test"}"#,
    );
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind KBD sentinel");
    listener
        .set_nonblocking(true)
        .expect("configure KBD sentinel");
    let endpoint = format!(
        "http://{}",
        listener.local_addr().expect("sentinel address")
    );

    let output = base_command(&project_root, &home_dir)
        .env("PROMETHEUS_CONTROL_ENDPOINT", endpoint)
        .args([
            "doctor",
            "--json",
            "--check",
            "skills",
            "--exclude",
            "control.kbd-runtime",
            "--exclude",
            "state.kbd-orchestrator",
            "--exclude",
            "control.kbd-rollout",
            "--exclude",
            "service:sovereign-sync",
        ])
        .output()
        .expect("run filtered doctor");

    assert!(
        output.status.success(),
        "filtered doctor should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        listener.accept().is_err(),
        "an excluded KBD check must not open a control-plane connection"
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("filtered doctor should emit JSON");
    let checks = payload["checks"].as_array().expect("checks array");
    assert!(checks.iter().all(|check| check["group"] == "skills"));
    assert_eq!(
        payload["selection"]["excluded"].as_array().map(Vec::len),
        Some(4)
    );
}

#[test]
fn sovereign_exclusion_is_propagated_to_service_repairs() {
    let (project_root, home_dir) = prepared_environment("doctor-service-exclusion");
    let output = base_command(&project_root, &home_dir)
        .args([
            "doctor",
            "--json",
            "--refresh",
            "--exclude",
            "control.kbd-runtime",
            "--exclude",
            "state.kbd-orchestrator",
            "--exclude",
            "control.kbd-rollout",
            "--exclude",
            "service:sovereign-sync",
        ])
        .output()
        .expect("run scoped refresh plan");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("refresh JSON");
    let safe_actions = payload["repair_plan"]["safe_actions"]
        .as_array()
        .expect("safe actions");
    let service_actions = safe_actions.iter().filter(|action| {
        action["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("services."))
    });
    for action in service_actions {
        assert!(
            action["command_hint"]
                .as_str()
                .is_some_and(|hint| hint.contains("--exclude sovereign-sync")),
            "service repair must preserve sovereign exclusion: {action}"
        );
    }
}

#[cfg(unix)]
#[test]
fn execution_doctor_receives_exclusions_before_optional_remote_configuration() {
    use std::os::unix::fs::PermissionsExt as _;

    let (project_root, home_dir) = prepared_environment("doctor-exec-exclusion");
    let arguments = home_dir.join("exec-doctor-arguments.txt");
    let binary = home_dir.join(".local/bin/prometheus-exec");
    write_file(
        &binary,
        &format!(
            "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{}'\nprintf '%s\\n' '{{\"healthy\":true,\"checks\":[]}}'\n",
            arguments.display()
        ),
    );
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    write_file(
        &home_dir.join("Library/LaunchAgents/ai.prometheus.exec.plist"),
        "prometheus-exec daemon",
    );
    fs::create_dir_all(home_dir.join(".prometheus/exec/remote")).unwrap();

    let output = base_command(&project_root, &home_dir)
        .args([
            "doctor",
            "--json",
            "--check",
            "execution",
            "--exclude",
            "control.kbd-runtime",
            "--exclude",
            "state.kbd-orchestrator",
            "--exclude",
            "control.kbd-rollout",
            "--exclude",
            "service:sovereign-sync",
        ])
        .output()
        .expect("run execution doctor");
    assert!(
        output.status.success(),
        "execution doctor failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let invoked = fs::read_to_string(arguments).unwrap();
    assert!(invoked.contains("service:sovereign-sync"));
    assert!(!invoked.contains("--remote-queue"));
}
