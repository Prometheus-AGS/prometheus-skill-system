//! Process-level integration coverage for `prometheus contract`.
//!
//! These tests run the compiled CLI as a real process across its real argument
//! and environment boundary, because the guarantees under test are process
//! guarantees: exit status, stdout shape, and — the one that matters most —
//! **an empty stderr when no control endpoint exists**. Seam 1 of
//! `docs/integration-contract.md` promises the pack never warns about a missing
//! extension; only running the binary can prove that.
//!
//! `PROMETHEUS_CLI_TEST_BINARY` overrides the binary under test so the same
//! target can be pointed at an installed CLI, matching `tests/kbd.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("prometheus-contract-{label}-{nanos}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

/// Run the CLI with a hermetic environment: no inherited control-endpoint
/// configuration, and a data dir that contains no socket, so discovery has
/// nothing to find unless a test puts it there.
fn run(args: &[&str], project: &Path, data_dir: &Path) -> Output {
    let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
    Command::new(binary)
        .args(args)
        .env_remove("PROMETHEUS_CONTROL_ENDPOINT")
        .env_remove("SOVEREIGN_SYNC_SOCKET")
        .env("XDG_DATA_HOME", data_dir)
        .env("HOME", data_dir)
        .current_dir(project)
        .output()
        .expect("run prometheus contract")
}

fn stdout_json(output: &Output) -> serde_json::Value {
    let text = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&text).unwrap_or_else(|error| {
        panic!("stdout is not JSON ({error}): {text}");
    })
}

#[test]
fn contract_show_is_silent_and_successful_when_no_endpoint_exists() {
    let project = unique_temp_dir("absent-project");
    let data = unique_temp_dir("absent-data");

    let output = run(&["contract", "show", "--json"], &project, &data);

    assert!(
        output.status.success(),
        "expected exit 0 with no endpoint, got {:?}",
        output.status
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim().is_empty(),
        "the pack must not warn when no extension is present; stderr was: {stderr}"
    );

    let report = stdout_json(&output);
    assert_eq!(report["contract_version"], "1.0.0");
    assert!(
        report["endpoint"].is_null(),
        "endpoint must be null when absent, got {}",
        report["endpoint"]
    );
    assert_eq!(report["endpoint_source"], "absent");
}

#[test]
fn contract_show_names_an_explicit_endpoint_override() {
    let project = unique_temp_dir("explicit-project");
    let data = unique_temp_dir("explicit-data");

    let binary = std::env::var_os("PROMETHEUS_CLI_TEST_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_prometheus").into());
    let output = Command::new(binary)
        .args(["contract", "show", "--json"])
        .env("PROMETHEUS_CONTROL_ENDPOINT", "http://127.0.0.1:7892/")
        .env_remove("SOVEREIGN_SYNC_SOCKET")
        .env("XDG_DATA_HOME", &data)
        .env("HOME", &data)
        .current_dir(&project)
        .output()
        .expect("run prometheus contract");

    assert!(output.status.success());
    let report = stdout_json(&output);
    assert_eq!(
        report["endpoint"], "http://127.0.0.1:7892",
        "trailing slash must be trimmed"
    );
    assert_eq!(report["endpoint_source"], "env:PROMETHEUS_CONTROL_ENDPOINT");
}

#[test]
fn contract_show_reports_the_service_manifest_when_present() {
    let project = unique_temp_dir("manifest-project");
    let data = unique_temp_dir("manifest-data");
    fs::create_dir_all(project.join("shared")).expect("create shared dir");
    fs::write(
        project.join("shared/services.manifest.json"),
        r#"{"contractVersion":"1.0.0","services":[]}"#,
    )
    .expect("write manifest");

    let output = run(&["contract", "show", "--json"], &project, &data);
    assert!(output.status.success());
    assert_eq!(
        stdout_json(&output)["service_manifest"],
        "shared/services.manifest.json"
    );
}

#[test]
fn contract_validate_accepts_a_conforming_declaration() {
    let project = unique_temp_dir("valid-project");
    let data = unique_temp_dir("valid-data");
    let declaration = project.join("skill-package.json");
    fs::write(
        &declaration,
        r#"{
          "name": "prometheus-companion",
          "version": "0.1.0",
          "minimumContractVersion": "1.0.0",
          "skills": "skills/",
          "hooks": { "bundles": ["prometheus-companion/sync-status"] },
          "mcpServers": {
            "sovereign-sync": { "command": "sovereign-sync", "args": ["--mode", "mcp"] }
          }
        }"#,
    )
    .expect("write declaration");

    let output = run(
        &["contract", "validate", declaration.to_str().unwrap()],
        &project,
        &data,
    );

    assert!(
        output.status.success(),
        "expected a conforming declaration to validate; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid: prometheus-companion 0.1.0"), "{stdout}");
}

#[test]
fn contract_validate_refuses_a_declaration_requiring_a_newer_contract() {
    let project = unique_temp_dir("newer-project");
    let data = unique_temp_dir("newer-data");
    let declaration = project.join("skill-package.json");
    fs::write(
        &declaration,
        r#"{"name":"future-pack","version":"1.0.0","minimumContractVersion":"2.0.0"}"#,
    )
    .expect("write declaration");

    let output = run(
        &["contract", "validate", declaration.to_str().unwrap()],
        &project,
        &data,
    );

    assert!(
        !output.status.success(),
        "a declaration requiring a newer contract must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("2.0.0") && stderr.contains("1.0.0"),
        "the failure must name both the required and the available version; stderr: {stderr}"
    );
}

#[test]
fn contract_validate_refuses_a_bare_hook_bundle_name() {
    let project = unique_temp_dir("bundle-project");
    let data = unique_temp_dir("bundle-data");
    let declaration = project.join("skill-package.json");
    fs::write(
        &declaration,
        r#"{
          "name": "third-party",
          "version": "0.1.0",
          "minimumContractVersion": "1.0.0",
          "hooks": { "bundles": ["kbd-control"] }
        }"#,
    )
    .expect("write declaration");

    let output = run(
        &["contract", "validate", declaration.to_str().unwrap()],
        &project,
        &data,
    );

    assert!(
        !output.status.success(),
        "an un-namespaced bundle could collide with a pack bundle and must be refused"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("namespaced"),
        "the failure must explain the namespacing rule"
    );
}
