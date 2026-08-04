use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

fn command() -> Command {
    Command::cargo_bin("prometheus-exec").expect("binary is built")
}

#[cfg(target_os = "macos")]
struct DaemonGuard(Option<Child>);

#[cfg(target_os = "macos")]
impl DaemonGuard {
    fn child_mut(&mut self) -> &mut Child {
        self.0.as_mut().expect("daemon child is present")
    }

    fn shutdown(mut self) {
        let mut child = self.0.take().expect("daemon child is present");
        child.kill().unwrap();
        child.wait().unwrap();
    }
}

#[cfg(target_os = "macos")]
impl Drop for DaemonGuard {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[test]
fn version_is_release_aligned() {
    command()
        .arg("--version")
        .assert()
        .success()
        .stdout("prometheus-exec 1.7.0\n")
        .stderr("");
}

#[test]
fn init_is_atomic_private_and_does_not_disclose_secret() {
    let directory = tempdir().unwrap();
    let identity = directory.path().join("identity.json");
    let assertion = command()
        .args(["init", "--identity"])
        .arg(&identity)
        .assert()
        .success()
        .stdout(predicate::str::contains("publicKey"))
        .stdout(predicate::str::contains("privateKey").not());
    drop(assertion);

    let value: serde_json::Value = serde_json::from_slice(&fs::read(&identity).unwrap()).unwrap();
    assert!(value["privateKey"].as_str().is_some());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(&identity).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    let before = fs::read(&identity).unwrap();
    command()
        .args(["init", "--identity"])
        .arg(&identity)
        .assert()
        .failure();
    assert_eq!(fs::read(identity).unwrap(), before);
}

#[test]
fn verify_failure_is_nonzero_and_creates_no_state() {
    let directory = tempdir().unwrap();
    let receipt = directory.path().join("missing-receipt.json");
    command()
        .args([
            "verify",
            "--receipt",
            receipt.to_str().unwrap(),
            "--public-key",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "--format",
            "json",
        ])
        .env("HOME", directory.path())
        .env("XDG_STATE_HOME", directory.path().join("state"))
        .env("XDG_RUNTIME_DIR", directory.path().join("runtime"))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("prometheus-exec:"));

    assert!(!directory.path().join("state").exists());
    assert!(!directory.path().join("runtime").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
fn doctor_failure_is_structured_non_mutating_and_never_false_green() {
    let directory = tempdir().unwrap();
    let socket = directory.path().join("runtime/exec.sock");
    let state = directory.path().join("state");
    let identity = directory.path().join("identity.json");
    command()
        .args(["doctor", "--socket"])
        .arg(&socket)
        .args(["--state-dir"])
        .arg(&state)
        .args(["--identity"])
        .arg(&identity)
        .args(["--format", "json"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"healthy\": false"))
        .stdout(predicate::str::contains("socket-permissions"))
        .stdout(predicate::str::contains("state-reconciliation"));
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[cfg(target_os = "macos")]
#[test]
fn daemon_run_status_verify_and_doctor_execute_a_real_python_use_case() {
    let directory = tempdir().unwrap();
    let identity = directory.path().join("identity.json");
    let socket = directory.path().join("runtime/exec.sock");
    let state = directory.path().join("state");
    let code = directory.path().join("use_case.py");
    fs::write(
        &code,
        r#"import json
import os
from pathlib import Path
payload = {"engine": "prometheus-exec", "value": 6 * 7}
Path(os.environ["PROMETHEUS_OUTPUT_DIR"], "result.json").write_text(
    json.dumps(payload, sort_keys=True), encoding="utf-8"
)
print(json.dumps(payload, sort_keys=True))
"#,
    )
    .unwrap();
    command()
        .args(["init", "--identity"])
        .arg(&identity)
        .assert()
        .success();
    let mut daemon = DaemonGuard(Some(
        command()
            .args(["daemon", "--socket"])
            .arg(&socket)
            .args(["--state-dir"])
            .arg(&state)
            .args(["--identity"])
            .arg(&identity)
            .args(["--artifact-budget-mb", "64"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    ));

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let ready = command()
            .args(["doctor", "--socket"])
            .arg(&socket)
            .args(["--state-dir"])
            .arg(&state)
            .args(["--identity"])
            .arg(&identity)
            .args(["--format", "json"])
            .output()
            .unwrap();
        if ready.status.success() {
            break;
        }
        assert!(Instant::now() < deadline, "daemon did not become ready");
        if let Some(status) = daemon.child_mut().try_wait().unwrap() {
            panic!("daemon exited before readiness with {status}");
        }
        thread::sleep(Duration::from_millis(50));
    }

    let executed = command()
        .args(["run", "--socket"])
        .arg(&socket)
        .args(["--state-dir"])
        .arg(&state)
        .args(["--identity"])
        .arg(&identity)
        .args(["--runtime", "python3", "--code"])
        .arg(&code)
        .args([
            "--timeout-ms",
            "5000",
            "--output-mb",
            "2",
            "--artifact-budget-mb",
            "64",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        executed.status.success(),
        "run failed: {}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let run: serde_json::Value = serde_json::from_slice(&executed.stdout).unwrap();
    assert_eq!(run["state"], "succeeded");
    assert_eq!(run["receipt"]["backend"], "seatbelt");
    assert_eq!(run["receipt"]["evidenceClass"], "attested");
    assert_eq!(run["receipt"]["exit"]["status"], 0);
    let run_id = run["runId"].as_str().unwrap();
    let artifact = &run["receipt"]["outputs"]["artifacts"][0];
    assert_eq!(artifact["path"], "outputs/result.json");
    let artifact_bytes = read_cas(&state, artifact["hash"].as_str().unwrap());
    let payload: serde_json::Value = serde_json::from_slice(&artifact_bytes).unwrap();
    assert_eq!(payload["value"], 42);

    command()
        .args(["status", "--socket"])
        .arg(&socket)
        .args(["--run-id", run_id, "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(run_id))
        .stdout(predicate::str::contains("\"state\": \"succeeded\""));

    let before_doctor = file_snapshot(&state);
    command()
        .args(["doctor", "--socket"])
        .arg(&socket)
        .args(["--state-dir"])
        .arg(&state)
        .args(["--identity"])
        .arg(&identity)
        .args(["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"healthy\": true"))
        .stdout(predicate::str::contains("1 records are structurally valid"));
    assert_eq!(file_snapshot(&state), before_doctor);

    let receipt_path = directory.path().join("receipt.json");
    fs::write(
        &receipt_path,
        serde_json::to_vec_pretty(&run["receipt"]).unwrap(),
    )
    .unwrap();
    let identity_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&identity).unwrap()).unwrap();
    command()
        .args(["verify", "--receipt"])
        .arg(&receipt_path)
        .args([
            "--public-key",
            identity_value["publicKey"].as_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("VALID sha256:"));

    daemon.shutdown();
}

#[cfg(target_os = "macos")]
fn read_cas(state: &Path, digest: &str) -> Vec<u8> {
    let hex = digest.strip_prefix("sha256:").unwrap();
    fs::read(
        state
            .join("artifacts/blobs/sha256")
            .join(&hex[..2])
            .join(&hex[2..]),
    )
    .unwrap()
}

#[cfg(target_os = "macos")]
fn file_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                visit(root, &path, snapshot);
            } else if metadata.is_file() {
                snapshot.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut snapshot = BTreeMap::new();
    visit(root, root, &mut snapshot);
    snapshot
}
