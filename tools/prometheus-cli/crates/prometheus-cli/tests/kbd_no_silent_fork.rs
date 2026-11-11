//! A checkout whose KBD plan came from another machine's journal must not silently start a
//! second journal. Operator A writes the plan; teammate B shares the checkout but has an empty
//! data directory (no copy of A's journal). B's first write is refused and creates nothing,
//! unless B explicitly opts in with `PROMETHEUS_KBD_NEW_JOURNAL=1`. A legacy (pre-runtime)
//! waypoint still bootstraps without the opt-in.

use kbd_runtime::Runtime;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Machine {
    data: PathBuf,
    config: PathBuf,
    key: PathBuf,
}

fn machine(root: &Path, name: &str) -> Machine {
    let base = root.join(name);
    let config = base.join("config");
    fs::create_dir_all(&config).unwrap();
    let source = Runtime::open(base.join("key"));
    source.device_signer().expect("generate machine key");
    Machine {
        data: base.join("data"),
        config,
        key: source.runtime_root().join("device-key.json"),
    }
}

fn run(project: &Path, machine: &Machine, new_journal: bool, args: &[&str]) -> Output {
    let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
    let mut command = Command::new(binary);
    command
        .current_dir(project)
        .env("XDG_CONFIG_HOME", &machine.config)
        .env("PROMETHEUS_DATA_DIR", &machine.data)
        .env("PROMETHEUS_DEVICE_KEY_FILE", &machine.key)
        .env("PROMETHEUS_DEVICE_ID", "kbd-no-silent-fork")
        .env("PROMETHEUS_CONTROL_ENDPOINT", "http://127.0.0.1:1")
        .env("PROMETHEUS_HARNESS", "kbd-no-silent-fork-integration")
        .env_remove("PROMETHEUS_KBD_NEW_JOURNAL")
        .args(["kbd", "--path", project.to_str().unwrap()])
        .args(args);
    if new_journal {
        command.env("PROMETHEUS_KBD_NEW_JOURNAL", "1");
    }
    command.output().expect("run prometheus CLI")
}

fn succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn journal_len(project: &Path, machine: &Machine) -> usize {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(project.join(".prometheus/project.json")).unwrap())
            .unwrap();
    Runtime::open_registered_at(project, &machine.data, manifest["projectId"].as_str().unwrap())
        .and_then(|runtime| runtime.events())
        .map(|events| events.len())
        .unwrap_or(0)
}

fn decision(id: &str) -> [&str; 8] {
    ["decision", "record", "--command-id", id, "--id", id, "--summary", "fork check"]
}

fn project(temp: &TempDir) -> PathBuf {
    let project = temp.path().join("project");
    fs::create_dir_all(project.join(".kbd-orchestrator")).unwrap();
    project
}

#[test]
fn teammate_without_the_journal_is_refused_instead_of_forking() {
    let temp = tempfile::tempdir().unwrap();
    let project = project(&temp);
    let (a, b) = (machine(temp.path(), "a"), machine(temp.path(), "b"));

    succeeded(&run(&project, &a, false, &["phase", "create", "--command-id", "a-phase", "--id", "p1", "--title", "Phase one"]));
    let waypoint: serde_json::Value = serde_json::from_slice(
        &fs::read(project.join(".kbd-orchestrator/current-waypoint.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(waypoint["generatedBy"], "kbd-runtime");
    assert!(waypoint["sourceRevision"].as_u64().unwrap() > 0);

    let refused = run(&project, &b, false, &decision("b-decision"));
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(!refused.status.success(), "B forked the journal:\n{stderr}");
    assert!(stderr.contains("PROMETHEUS_KBD_NEW_JOURNAL"), "{stderr}");
    assert_eq!(journal_len(&project, &b), 0, "refusal created a journal");

    succeeded(&run(&project, &b, true, &decision("b-new-journal")));
    assert!(journal_len(&project, &b) > 0);
}

#[test]
fn legacy_waypoint_still_bootstraps_without_opt_in() {
    let temp = tempfile::tempdir().unwrap();
    let project = project(&temp);
    fs::write(
        project.join(".kbd-orchestrator/current-waypoint.json"),
        br#"{"phase":"legacy-phase","status":"executing","planRevision":2}"#,
    )
    .unwrap();
    let a = machine(temp.path(), "a");

    succeeded(&run(&project, &a, false, &decision("legacy-decision")));
    assert!(journal_len(&project, &a) > 0);
}
