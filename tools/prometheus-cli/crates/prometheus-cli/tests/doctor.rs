use std::collections::BTreeSet;
use std::fs;
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
        stdout.contains("unreachable"),
        "doctor output should report the memory service as unreachable; stdout:\n{stdout}"
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
    assert_eq!(before, after, "dry-run refresh must not mutate the filesystem");
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
            action
                .get("id")
                .and_then(|value| value.as_str())
                == Some("services.install-mcp-services")
        }),
        "refresh plan should surface the managed services repair action; payload: {payload}"
    );
    assert!(
        manual_actions.iter().any(|action| {
            action
                .get("id")
                .and_then(|value| value.as_str())
                == Some("manual.review-hooks")
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
    assert_eq!(before, after, "refresh without --yes must not mutate the filesystem");
}
