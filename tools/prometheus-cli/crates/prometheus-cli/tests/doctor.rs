use std::collections::BTreeSet;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest as _, Sha256};

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

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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

#[cfg(unix)]
fn structured_hook_fixture(label: &str, hooks: &serde_json::Value) -> (PathBuf, PathBuf) {
    use std::os::unix::fs::symlink;

    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .expect("repository root");
    let project = unique_temp_dir(&format!("{label}-project"));
    let home = unique_temp_dir(&format!("{label}-home"));
    for relative in [
        "shared/harnesses/generated/release-manifest.json",
        "shared/harnesses/hook-contract.json",
        "dist/plugins/codex/prometheus-skill-pack/.codex-plugin/plugin.json",
    ] {
        let destination = project.join(relative);
        fs::create_dir_all(destination.parent().unwrap()).expect("fixture parent");
        fs::copy(repository.join(relative), destination).expect("copy fixture source");
    }
    write_file(
        &project.join("hooks/codex-hooks.json"),
        &format!("{}\n", serde_json::to_string_pretty(hooks).unwrap()),
    );
    let plugin_root = home.join(".prometheus/plugins/prometheus-skill-pack");
    let generation = "fixture-generation";
    let generation_root = plugin_root.join("generations").join(generation);
    let release: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join("shared/harnesses/generated/release-manifest.json"))
            .expect("release manifest"),
    )
    .expect("release manifest JSON");
    let bundle = release["bundleId"].as_str().expect("bundle id");
    let source_plugin =
        project.join("dist/plugins/codex/prometheus-skill-pack/.codex-plugin/plugin.json");
    let plugin: serde_json::Value =
        serde_json::from_slice(&fs::read(&source_plugin).expect("source plugin"))
            .expect("source plugin JSON");
    let version = plugin["version"].as_str().expect("plugin version");
    let runner = b"fixture runner\n";

    fs::create_dir_all(generation_root.join("hooks")).expect("generation hooks");
    fs::create_dir_all(generation_root.join(".codex-plugin")).expect("generation plugin");
    fs::copy(
        project.join("hooks/codex-hooks.json"),
        generation_root.join("hooks/codex-hooks.json"),
    )
    .expect("copy hooks");
    fs::copy(
        &source_plugin,
        generation_root.join(".codex-plugin/plugin.json"),
    )
    .expect("copy plugin");
    write_file(
        &generation_root.join("manifest.json"),
        &format!(
            "{{\"bundleId\":\"{bundle}\",\"hookRuntime\":{{\"runnerSha256\":\"{}\"}}}}\n",
            sha256(runner)
        ),
    );
    write_file(
        &plugin_root.join("runtime/v1/run-hook"),
        std::str::from_utf8(runner).unwrap(),
    );
    fs::create_dir_all(plugin_root.join("bundles")).expect("bundle index");
    symlink(
        Path::new("generations").join(generation),
        plugin_root.join("current"),
    )
    .expect("active generation");
    symlink(
        Path::new("../generations").join(generation),
        plugin_root.join("bundles").join(bundle),
    )
    .expect("bundle generation");
    let cache = home
        .join(".codex/plugins/cache/prometheus-skill-pack/prometheus-skill-pack")
        .join(version)
        .join(".codex-plugin/plugin.json");
    fs::create_dir_all(cache.parent().unwrap()).expect("Codex cache");
    fs::copy(source_plugin, cache).expect("copy Codex plugin cache");

    (project, home)
}

#[cfg(unix)]
fn run_hook_doctor(project: &Path, home: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_prometheus"))
        .current_dir(project)
        .env("HOME", home)
        .args(["doctor", "--json", "--check", "hooks.harness-adapters"])
        .output()
        .expect("run hook graph doctor")
}

fn first_hook_args_mut(value: &mut serde_json::Value) -> Option<&mut Vec<serde_json::Value>> {
    match value {
        serde_json::Value::Object(object) => {
            if object.get("command").and_then(serde_json::Value::as_str) == Some("node") {
                return object
                    .get_mut("args")
                    .and_then(serde_json::Value::as_array_mut);
            }
            object.values_mut().find_map(first_hook_args_mut)
        }
        serde_json::Value::Array(values) => values.iter_mut().find_map(first_hook_args_mut),
        _ => None,
    }
}

#[cfg(unix)]
#[test]
fn doctor_accepts_generated_structured_codex_hook_graph() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .expect("repository root");
    let hooks: serde_json::Value = serde_json::from_slice(
        &fs::read(repository.join("hooks/codex-hooks.json")).expect("source hooks"),
    )
    .expect("source hooks JSON");
    let (project, home) = structured_hook_fixture("doctor-structured-codex", &hooks);

    let output = run_hook_doctor(&project, &home);

    assert!(
        output.status.success(),
        "hook graph doctor failed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    assert_eq!(payload["summary"]["failed"], 0);
    assert_eq!(payload["checks"][0]["id"], "hooks.harness-adapters");
}

#[cfg(unix)]
#[test]
fn doctor_rejects_unpinned_structured_codex_hook_graphs() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .expect("repository root");
    let original: serde_json::Value = serde_json::from_slice(
        &fs::read(repository.join("hooks/codex-hooks.json")).expect("source hooks"),
    )
    .expect("source hooks JSON");

    for (label, mutate) in [
        "wrong-entry",
        "wrong-hook-id",
        "mutable-runtime",
        "duplicate-hook-flag",
    ]
    .into_iter()
    .enumerate()
    {
        let mut hooks = original.clone();
        let args = first_hook_args_mut(&mut hooks).expect("first hook arguments");
        match mutate {
            "wrong-entry" => {
                args[0] = serde_json::Value::String(
                    "${CLAUDE_PLUGIN_ROOT}/unexpected/scripts/hook-entry.mjs".into(),
                )
            }
            "wrong-hook-id" => {
                let position = args
                    .iter()
                    .position(|value| value.as_str() == Some("--hook"))
                    .unwrap();
                args[position + 1] = serde_json::Value::String("fabricated-hook".into());
            }
            "mutable-runtime" => args.push(serde_json::Value::String("/stable/unpinned".into())),
            "duplicate-hook-flag" => {
                args.push(serde_json::Value::String("--hook".into()));
                args.push(serde_json::Value::String("fabricated-hook".into()));
            }
            _ => unreachable!(),
        }
        let (project, home) = structured_hook_fixture(&format!("doctor-invalid-{label}"), &hooks);
        let output = run_hook_doctor(&project, &home);
        assert!(!output.status.success(), "invalid case {mutate} passed");
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("doctor JSON");
        assert_eq!(payload["summary"]["failed"], 1, "{mutate}: {payload}");
        assert!(
            payload["checks"][0]["details"]
                .as_array()
                .is_some_and(|details| details.iter().any(|detail| detail
                    .as_str()
                    .is_some_and(|text| text.contains("not pinned")))),
            "{mutate}: {payload}"
        );
    }
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
        Some(3)
    );
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
            "service:remote-queue",
        ])
        .output()
        .expect("run execution doctor");
    assert!(
        output.status.success(),
        "execution doctor failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let invoked = fs::read_to_string(arguments).unwrap();
    assert!(invoked.contains("service:remote-queue"));
    assert!(!invoked.contains(" --remote-queue "));
}

/// change-cpc-009: the pack must be complete and silent about the Companion
/// when it is absent (D-02) — no service name, no warning, exit 0.
///
/// Reproduces "the Companion is absent" for real, not by mocking the check:
/// no `PROMETHEUS_CONTROL_ENDPOINT`, no `SOVEREIGN_SYNC_SOCKET`, and `HOME`
/// pointed at a fresh directory with no control socket ever created under it,
/// so `contract::report`'s own discovery chain (which this check now shares —
/// see `check_kbd_control_plane`'s doc comment) genuinely finds nothing.
#[test]
fn control_plane_check_is_silent_when_the_companion_is_absent() {
    let (project_root, home_dir) = prepared_environment("doctor-no-companion");

    let output = base_command(&project_root, &home_dir)
        .env_remove("PROMETHEUS_CONTROL_ENDPOINT")
        .env_remove("SOVEREIGN_SYNC_SOCKET")
        .args(["doctor", "--json", "--check", "control"])
        .output()
        .expect("run control-only doctor");

    assert!(
        output.status.success(),
        "doctor must exit 0 with no Companion installed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor should emit JSON");
    let checks = payload["checks"].as_array().expect("checks array");
    let control_plane_check = checks
        .iter()
        .find(|check| check["id"] == "control.kbd-runtime")
        .expect("control.kbd-runtime check must run");

    assert_eq!(control_plane_check["status"], "skip");
    let rendered = control_plane_check.to_string();
    assert!(
        !rendered.to_lowercase().contains("sovereign"),
        "no item may mention sovereign-sync when the Companion is absent: {rendered}"
    );
    assert!(
        !rendered.to_lowercase().contains("warn"),
        "an absent, optional extension must not be reported as a warning: {rendered}"
    );
}
