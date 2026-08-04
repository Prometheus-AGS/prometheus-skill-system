use std::{fs, process::Command};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::tempdir;

fn command() -> Command {
    Command::cargo_bin("prometheus-exec").expect("binary is built")
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
