//! Two-user KBD enrollment through the real `prometheus kbd` binary.
//!
//! Operator A sets up the plan; teammate B is refused until A enrolls B's key with
//! `device enroll --operator`; B then writes and controls the run lifecycle; a third device C
//! enrolled without `--operator` can write but cannot cancel the run or grant itself operator;
//! revoking B refuses B's later writes. The three users share one data directory, standing in
//! for a synced journal, and sign with separate key files.

use kbd_runtime::{EventKind, Runtime};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Team {
    temp: TempDir,
    project: PathBuf,
    data: PathBuf,
}

struct Member {
    key: PathBuf,
}

impl Team {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("create team fixture");
        let project = temp.path().join("project");
        let data = temp.path().join("data");
        fs::create_dir_all(project.join(".kbd-orchestrator")).unwrap();
        fs::create_dir_all(temp.path().join("config")).unwrap();
        Self { temp, project, data }
    }

    /// A member with a fresh signing key file, created by the runtime's own key writer.
    fn member(&self, name: &str) -> Member {
        let source = Runtime::open(self.temp.path().join(format!("key-{name}")));
        source.device_signer().expect("generate member key");
        Member {
            key: source.runtime_root().join("device-key.json"),
        }
    }

    fn run(&self, member: &Member, args: &[&str]) -> Output {
        let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
        Command::new(binary)
            .current_dir(&self.project)
            .env("XDG_CONFIG_HOME", self.temp.path().join("config"))
            .env("PROMETHEUS_DATA_DIR", &self.data)
            .env("PROMETHEUS_DEVICE_KEY_FILE", &member.key)
            .env("PROMETHEUS_DEVICE_ID", "kbd-team-enrollment")
            .env("PROMETHEUS_CONTROL_ENDPOINT", "http://127.0.0.1:1")
            .env("PROMETHEUS_HARNESS", "kbd-team-enrollment-integration")
            .args(["kbd", "--path", self.project.to_str().unwrap()])
            .args(args)
            .output()
            .expect("run prometheus CLI")
    }

    fn events(&self) -> Vec<kbd_runtime::Event> {
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(self.project.join(".prometheus/project.json")).unwrap(),
        )
        .unwrap();
        Runtime::open_registered_at(&self.project, &self.data, manifest["projectId"].as_str().unwrap())
            .unwrap()
            .events()
            .unwrap()
    }

    fn identity(&self, member: &Member) -> (String, String) {
        let output = self.run(member, &["device", "identity", "--json"]);
        succeeded(&output);
        let identity: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        (
            identity["publicKey"].as_str().unwrap().into(),
            identity["keyId"].as_str().unwrap().into(),
        )
    }

    /// Runs a command that must be refused and must not append an event.
    fn refused(&self, member: &Member, args: &[&str]) -> String {
        let before = self.events().len();
        let output = self.run(member, args);
        assert!(!output.status.success(), "expected refusal for {args:?}");
        assert_eq!(self.events().len(), before, "refused {args:?} appended an event");
        String::from_utf8_lossy(&output.stderr).into_owned()
    }
}

fn succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn decision(id: &str) -> [&str; 8] {
    ["decision", "record", "--command-id", id, "--id", id, "--summary", "integration decision"]
}

#[test]
fn operator_enrolls_teammate_as_co_operator_and_revokes_them() {
    let team = Team::new();
    let (a, b, c) = (team.member("a"), team.member("b"), team.member("c"));

    succeeded(&team.run(&a, &["phase", "create", "--command-id", "a-phase", "--id", "p1", "--title", "Phase one"]));
    let (b_public, b_key) = team.identity(&b);

    // Before enrollment B is refused, and the refusal names the enrollment path.
    let stderr = team.refused(&b, &["revise", "--reason", "B before enrollment"]);
    assert!(stderr.contains("prometheus kbd device identity"), "{stderr}");

    succeeded(&team.run(&a, &[
        "device", "enroll", "--command-id", "enroll-b", "--device-id", "laptop-b",
        "--public-key", &b_public, "--operator",
    ]));
    let listed = team.run(&a, &["device", "list", "--json"]);
    succeeded(&listed);
    let roster: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert!(roster
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["keyId"] == b_key.as_str() && row["operator"] == true));

    // As co-operator, B writes, revises, and controls the run lifecycle.
    succeeded(&team.run(&b, &decision("b-decision")));
    succeeded(&team.run(&b, &["revise", "--reason", "B course-corrects the plan"]));
    succeeded(&team.run(&b, &["pause", "--reason", "B pauses as co-operator"]));
    succeeded(&team.run(&b, &["resume"]));

    // B enrolls C as a device only: C writes and revises, but holds no operator authority.
    let (c_public, c_key) = team.identity(&c);
    succeeded(&team.run(&b, &[
        "device", "enroll", "--command-id", "enroll-c", "--device-id", "laptop-c",
        "--public-key", &c_public,
    ]));
    team.refused(&c, &["device", "grant-operator", "--command-id", "c-grant", "--key-id", &c_key]);
    succeeded(&team.run(&c, &decision("c-decision")));
    succeeded(&team.run(&c, &["revise", "--reason", "C revises the plan"]));
    let stderr = team.refused(&c, &["cancel", "--reason", "C tries to cancel the run"]);
    assert!(stderr.contains("active enrolled operator"), "{stderr}");

    // Revoking B refuses B's later writes.
    succeeded(&team.run(&a, &[
        "device", "revoke", "--command-id", "revoke-b", "--key-id", &b_key,
        "--reason", "B left the project",
    ]));
    team.refused(&b, &decision("b-after-revoke"));

    let events = team.events();
    let granted: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.kind {
            EventKind::OperatorGranted { key_id } => Some(key_id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(granted, [b_key.as_str()]);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event.kind, EventKind::DeviceEnrolled { .. }))
            .count(),
        2
    );
}
