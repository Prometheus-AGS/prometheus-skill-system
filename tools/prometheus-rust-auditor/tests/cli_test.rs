use assert_cmd::Command;
use predicates::prelude::*;

fn auditor() -> Command {
    Command::cargo_bin("prometheus-rust-auditor").unwrap()
}

// ── help / version ──────────────────────────────────────────────────────────

#[test]
fn help_exits_zero() {
    auditor().arg("--help").assert().success();
}

#[test]
fn version_exits_zero() {
    auditor().arg("--version").assert().success();
}

#[test]
fn help_lists_all_subcommands() {
    auditor()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("audit"))
        .stdout(predicate::str::contains("enforce"))
        .stdout(predicate::str::contains("format"))
        .stdout(predicate::str::contains("deps"))
        .stdout(predicate::str::contains("inventory"))
        .stdout(predicate::str::contains("partition"))
        .stdout(predicate::str::contains("ci"))
        .stdout(predicate::str::contains("autonomous"));
}

// ── output formats ───────────────────────────────────────────────────────────

#[test]
fn unknown_format_exits_nonzero() {
    auditor()
        .args(["--format", "bogus", "enforce"])
        .assert()
        .failure();
}

#[test]
fn json_format_produces_valid_json() {
    let output = auditor()
        .args(["--format", "json", "inventory"])
        .output()
        .unwrap();
    // inventory always exits 0 (no findings on success)
    assert!(output.status.success(), "inventory should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("--format json output must be valid JSON");
    assert!(parsed.get("phase").is_some(), "JSON must have a 'phase' key");
    assert!(parsed.get("findings").is_some(), "JSON must have a 'findings' key");
}

#[test]
fn text_format_is_default() {
    let output = auditor().args(["inventory"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // text format starts with ✓ or ✗
    assert!(
        stdout.starts_with("✓") || stdout.starts_with("✗"),
        "default text output should start with ✓ or ✗, got: {stdout}"
    );
}

// ── subcommand exit codes ────────────────────────────────────────────────────

#[test]
fn enforce_exits_zero_on_clean_workspace() {
    // The auditor crate itself is clean (no clippy errors)
    auditor()
        .args(["--format", "json", "enforce"])
        .assert()
        .success();
}

#[test]
fn inventory_exits_zero() {
    auditor()
        .args(["--format", "json", "inventory"])
        .assert()
        .success();
}

#[test]
fn partition_exits_zero() {
    // partition always returns Ok(0) even with Info findings
    auditor()
        .args(["--format", "json", "partition"])
        .assert()
        .success();
}

#[test]
fn ci_exits_zero() {
    auditor()
        .args(["--format", "json", "ci"])
        .assert()
        .success();
}

#[test]
fn autonomous_exits_zero() {
    auditor()
        .args(["--format", "json", "autonomous"])
        .assert()
        .success();
}

#[test]
fn format_phase_exits_correctly() {
    // This is the auditor's own source tree. If it has fmt issues the test
    // still verifies the exit code matches the presence/absence of findings.
    let output = auditor()
        .args(["--format", "json", "format"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("format --format json must produce valid JSON");
    let findings = parsed["findings"].as_array().unwrap();
    if findings.is_empty() {
        assert!(output.status.success(), "no findings → exit 0");
    } else {
        assert!(!output.status.success(), "findings present → exit non-zero");
    }
}

#[test]
fn deps_exits_zero_regardless_of_tool_presence() {
    // cargo-deny and cargo-audit may or may not be installed; either way
    // the phase must not crash and must exit 0 when only Info findings exist
    // (graceful skip), or 1 when High findings exist.
    let output = auditor()
        .args(["--format", "json", "deps"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: serde_json::Value =
        serde_json::from_str(&stdout).expect("deps output must be valid JSON");
    // Just verify it doesn't crash (exit code 2 = tool error, which we don't want)
    assert_ne!(
        output.status.code(),
        Some(2),
        "deps phase must not exit with code 2 (tool error)"
    );
}

// ── config flag ──────────────────────────────────────────────────────────────

#[test]
fn nonexistent_config_falls_back_to_default() {
    auditor()
        .args(["--config", "/tmp/this-does-not-exist-auditor.toml", "--format", "json", "inventory"])
        .assert()
        .success();
}

// ── verbose flag ─────────────────────────────────────────────────────────────

#[test]
fn verbose_flag_accepted() {
    auditor()
        .args(["--verbose", "--format", "json", "inventory"])
        .assert()
        .success();
}

// ── audit (full pipeline) ────────────────────────────────────────────────────

#[test]
fn audit_subcommand_runs_all_phases_without_crashing() {
    let output = auditor()
        .args(["--format", "json", "audit"])
        .output()
        .unwrap();
    // exit 2 means a hard tool error — that must never happen
    assert_ne!(
        output.status.code(),
        Some(2),
        "audit must not exit with code 2 (tool error); stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
