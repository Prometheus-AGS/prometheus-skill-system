use std::{fs, path::Path, process::Command};

#[cfg(target_os = "macos")]
use std::{
    collections::BTreeMap,
    io::{Read as _, Write as _},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::{Child, Stdio},
    thread,
    time::{Duration, Instant},
};

use assert_cmd::prelude::*;
#[cfg(target_os = "macos")]
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
#[cfg(target_os = "macos")]
use chrono::Utc;
#[cfg(target_os = "macos")]
use ed25519_dalek::SigningKey;
use predicates::prelude::*;
#[cfg(target_os = "macos")]
use prometheus_exec_contracts::{
    hash_bytes, sign_request_ed25519, CapabilityManifest, CodeIdentity, CodeKind, ExecutionLimits,
    ExecutionProvenance, NetworkCapabilities, RequestedTier, RuntimeKind, SignatureAlgorithm,
    SignedExecRequest, SCHEMA_VERSION,
};
use tempfile::tempdir;
#[cfg(target_os = "macos")]
use uuid::Uuid;

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

    fn sigkill(mut self) {
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
fn cli_exposes_component_submission_and_offline_replay_options() {
    command()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wasm-component"))
        .stdout(predicate::str::contains("--plugin-root"));
    command()
        .args(["verify", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--component"))
        .stdout(predicate::str::contains("--input"));
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

#[test]
fn contracts_regenerate_checked_in_references_byte_for_byte() {
    let directory = tempdir().unwrap();
    command()
        .args(["contracts", "--output-dir"])
        .arg(directory.path())
        .assert()
        .success();

    let references = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference/api");
    for name in [
        "prometheus-exec.openapi.json",
        "prometheus-exec.schemas.json",
    ] {
        assert_eq!(
            fs::read(directory.path().join(name)).unwrap(),
            fs::read(references.join(name)).unwrap(),
            "generated {name} drifted from its checked-in reference"
        );
    }
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

    wait_until_ready(&mut daemon, &socket, &state, &identity);

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

    daemon.sigkill();
    assert!(
        socket.exists(),
        "SIGKILL should leave a stale socket to recover"
    );

    let mut restarted = DaemonGuard(Some(
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
    wait_until_ready(&mut restarted, &socket, &state, &identity);
    command()
        .args(["status", "--socket"])
        .arg(&socket)
        .args(["--run-id", run_id, "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(run_id))
        .stdout(predicate::str::contains("\"state\": \"succeeded\""));
    restarted.sigkill();
}

#[cfg(target_os = "macos")]
#[test]
fn privileged_request_becomes_durable_grant_pending_without_spawn() {
    let directory = tempdir().unwrap();
    let identity = directory.path().join("identity.json");
    let socket = directory.path().join("runtime/exec.sock");
    let state = directory.path().join("state");
    command()
        .args(["init", "--identity"])
        .arg(&identity)
        .assert()
        .success();
    let mut daemon = spawn_daemon(&socket, &state, &identity);
    wait_until_ready(&mut daemon, &socket, &state, &identity);

    let identity_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&identity).unwrap()).unwrap();
    let private = URL_SAFE_NO_PAD
        .decode(identity_value["privateKey"].as_str().unwrap())
        .unwrap();
    let signing_key = SigningKey::from_bytes(private.as_slice().try_into().unwrap());
    let request_id = Uuid::new_v4();
    let mut request = SignedExecRequest {
        schema_version: SCHEMA_VERSION.into(),
        request_id,
        issued_at: Utc::now(),
        queued_at: None,
        validity_window_secs: 3600,
        tier: RequestedTier::P,
        code: CodeIdentity {
            kind: CodeKind::Inline,
            hash: hash_bytes(b"print('must not spawn')"),
            runtime: RuntimeKind::Python3,
            toolchain_pin: None,
        },
        inputs: vec![],
        capabilities: CapabilityManifest {
            net: NetworkCapabilities {
                egress: vec!["https://example.com".into()],
            },
            ..CapabilityManifest::default()
        },
        limits: ExecutionLimits::default(),
        targets: vec![],
        provenance: ExecutionProvenance {
            harness: Some("grant-pending-integration-test".into()),
            ..ExecutionProvenance::default()
        },
        signer_key_id: None,
        sig_alg: SignatureAlgorithm::Ed25519,
        signature: None,
    };
    sign_request_ed25519(&mut request, &signing_key).unwrap();
    let accepted: serde_json::Value = serde_json::from_slice(&uds_post_json(
        &socket,
        "/api/v2/exec/runs",
        &serde_json::to_vec(&request).unwrap(),
    ))
    .unwrap();
    let run_id = accepted["runId"].as_str().unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = command()
            .args(["status", "--socket"])
            .arg(&socket)
            .args(["--run-id", run_id, "--format", "json"])
            .output()
            .unwrap();
        assert!(status.status.success());
        let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
        if status["state"] == "grant-pending" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "privileged request never became grant-pending"
        );
        thread::sleep(Duration::from_millis(25));
    }
    let record: serde_json::Value = serde_json::from_slice(
        &fs::read(
            state
                .join("service/ledger/runs")
                .join(format!("{request_id}.json")),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(record["state"], "grant-pending");
    assert_eq!(record["spawn"]["status"], "notSpawned");
    assert!(record["terminal"].is_null());

    daemon.sigkill();
    let mut restarted = spawn_daemon(&socket, &state, &identity);
    wait_until_ready(&mut restarted, &socket, &state, &identity);
    command()
        .args(["status", "--socket"])
        .arg(&socket)
        .args(["--run-id", run_id, "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"grant-pending\""));
    restarted.sigkill();

    let record_path = state
        .join("service/ledger/runs")
        .join(format!("{request_id}.json"));
    let mut corrupted: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).unwrap()).unwrap();
    corrupted.as_object_mut().unwrap().remove("grantPending");
    let mut corrupted_bytes = serde_json::to_vec_pretty(&corrupted).unwrap();
    corrupted_bytes.push(b'\n');
    fs::write(&record_path, &corrupted_bytes).unwrap();
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
        .stdout(predicate::str::contains("state-reconciliation"))
        .stdout(predicate::str::contains("invalid run record"));
    assert_eq!(fs::read(record_path).unwrap(), corrupted_bytes);
}

#[cfg(target_os = "macos")]
fn spawn_daemon(socket: &Path, state: &Path, identity: &Path) -> DaemonGuard {
    DaemonGuard(Some(
        command()
            .args(["daemon", "--socket"])
            .arg(socket)
            .args(["--state-dir"])
            .arg(state)
            .args(["--identity"])
            .arg(identity)
            .args(["--artifact-budget-mb", "64"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    ))
}

#[cfg(target_os = "macos")]
fn uds_post_json(socket: &Path, target: &str, body: &[u8]) -> Vec<u8> {
    let mut stream = UnixStream::connect(socket).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(
        stream,
        "POST {target} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).unwrap();
    let body_start = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .unwrap();
    assert!(response.starts_with(b"HTTP/1.1 202"));
    response[body_start..].to_vec()
}

#[cfg(target_os = "macos")]
fn wait_until_ready(daemon: &mut DaemonGuard, socket: &Path, state: &Path, identity: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let ready = command()
            .args(["doctor", "--socket"])
            .arg(socket)
            .args(["--state-dir"])
            .arg(state)
            .args(["--identity"])
            .arg(identity)
            .args(["--format", "json"])
            .output()
            .unwrap();
        if ready.status.success() {
            return;
        }
        assert!(Instant::now() < deadline, "daemon did not become ready");
        if let Some(status) = daemon.child_mut().try_wait().unwrap() {
            panic!("daemon exited before readiness with {status}");
        }
        thread::sleep(Duration::from_millis(50));
    }
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
