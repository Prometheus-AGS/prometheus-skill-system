// Integration tests for individual phase outputs via the CLI.
// These tests verify what each phase actually emits rather than testing the
// phase functions directly (which would require constructing AppContext).

use assert_cmd::Command;
use std::fs;

fn auditor() -> Command {
    Command::cargo_bin("prometheus-rust-auditor").unwrap()
}

fn json_output(args: &[&str]) -> serde_json::Value {
    let output = auditor()
        .args(args)
        .output()
        .expect("auditor must run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("phase output must be valid JSON. error={e}\nstdout={stdout}"))
}

// ── enforce phase ────────────────────────────────────────────────────────────

#[test]
fn enforce_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "enforce"]);
    assert_eq!(v["phase"], "enforce");
}

#[test]
fn enforce_phase_findings_have_required_fields() {
    let v = json_output(&["--format", "json", "enforce"]);
    for finding in v["findings"].as_array().unwrap() {
        assert!(finding.get("severity").is_some(), "finding must have severity");
        assert!(finding.get("crate_name").is_some(), "finding must have crate_name");
        assert!(finding.get("message").is_some(), "finding must have message");
    }
}

// ── format phase ─────────────────────────────────────────────────────────────

#[test]
fn format_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "format"]);
    assert_eq!(v["phase"], "format");
}

#[test]
fn format_phase_high_finding_when_fmt_fails() {
    let v = json_output(&["--format", "json", "format"]);
    for finding in v["findings"].as_array().unwrap() {
        // If there are findings, they must be High severity
        assert_eq!(finding["severity"], "high", "format findings must be severity 'high'");
    }
}

// ── deps phase ────────────────────────────────────────────────────────────────

#[test]
fn deps_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "deps"]);
    assert_eq!(v["phase"], "deps");
}

#[test]
fn deps_phase_tool_skip_is_info_severity() {
    let v = json_output(&["--format", "json", "deps"]);
    for finding in v["findings"].as_array().unwrap() {
        let sev = finding["severity"].as_str().unwrap();
        // Skipped-tool findings must be Info, not High/Critical
        if finding["message"].as_str().unwrap_or("").contains("not installed") {
            assert_eq!(sev, "info", "tool-skip finding must be 'info' severity");
        }
    }
}

// ── inventory phase ───────────────────────────────────────────────────────────

#[test]
fn inventory_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "inventory"]);
    assert_eq!(v["phase"], "inventory");
}

#[test]
fn inventory_phase_exits_zero_with_empty_findings() {
    // Inventory always produces no findings on a valid workspace
    let output = auditor()
        .args(["--format", "json", "inventory"])
        .output()
        .unwrap();
    assert!(output.status.success(), "inventory must exit 0 on a valid workspace");
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(v["findings"].as_array().unwrap().len(), 0, "inventory must have no findings");
}

// ── partition phase ───────────────────────────────────────────────────────────

#[test]
fn partition_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "partition"]);
    assert_eq!(v["phase"], "partition");
}

#[test]
fn partition_phase_emits_one_finding_per_partition() {
    let v = json_output(&["--format", "json", "partition"]);
    // Default config has 6 partitions; each gets one Info finding
    let findings = v["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 6, "default config has 6 partitions → 6 partition findings");
}

#[test]
fn partition_phase_findings_are_info_severity() {
    let v = json_output(&["--format", "json", "partition"]);
    for finding in v["findings"].as_array().unwrap() {
        assert_eq!(finding["severity"], "info", "partition findings must be 'info' severity");
    }
}

// ── ci phase ─────────────────────────────────────────────────────────────────

#[test]
fn ci_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "ci"]);
    assert_eq!(v["phase"], "ci");
}

#[test]
fn ci_phase_generates_workflow_file() {
    // Run CI phase and verify the file was created
    auditor()
        .args(["--format", "json", "ci"])
        .assert()
        .success();

    let workflow = std::path::Path::new(".github/workflows/rust-audit.yml");
    assert!(workflow.exists(), ".github/workflows/rust-audit.yml must be created by ci phase");

    let content = fs::read_to_string(workflow).unwrap();
    assert!(content.contains("cargo clippy"), "workflow must contain clippy step");
    assert!(content.contains("cargo fmt"), "workflow must contain fmt step");
    assert!(content.contains("cargo test"), "workflow must contain test step");
}

#[test]
fn ci_phase_finding_message_mentions_workflow_path() {
    let v = json_output(&["--format", "json", "ci"]);
    let findings = v["findings"].as_array().unwrap();
    assert!(!findings.is_empty(), "ci phase must emit at least one finding");
    let msg = findings[0]["message"].as_str().unwrap();
    assert!(msg.contains("rust-audit.yml"), "ci finding must mention the workflow filename");
}

// ── autonomous phase ──────────────────────────────────────────────────────────

#[test]
fn autonomous_phase_reports_correct_phase_name() {
    let v = json_output(&["--format", "json", "autonomous"]);
    assert_eq!(v["phase"], "autonomous");
}

#[test]
fn autonomous_phase_emits_stub_finding() {
    let v = json_output(&["--format", "json", "autonomous"]);
    let findings = v["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "autonomous stub must emit exactly one finding");
    assert_eq!(findings[0]["severity"], "info");
    assert!(
        findings[0]["message"].as_str().unwrap().contains("not yet implemented"),
        "autonomous finding must indicate stub status"
    );
}

// ── audit (full pipeline) ─────────────────────────────────────────────────────

#[test]
fn audit_runs_without_exit_code_2() {
    let output = auditor()
        .args(["--format", "text", "audit"])
        .output()
        .unwrap();
    assert_ne!(
        output.status.code(),
        Some(2),
        "full audit must never exit with code 2 (tool error)"
    );
}
