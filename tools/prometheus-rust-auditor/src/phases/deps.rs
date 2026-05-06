use crate::AppContext;
use anyhow::Result;
use prometheus_rust_auditor::reporter::{Finding, Report, Severity};
use std::process::Command;

fn tool_not_found(stderr: &str) -> bool {
    stderr.contains("no such subcommand")
        || stderr.contains("not found")
        || stderr.contains("command not found")
}

pub fn run(ctx: &AppContext) -> Result<i32> {
    let mut findings = Vec::new();

    // cargo-deny
    let deny_result = Command::new("cargo")
        .args(["deny", "check"])
        .current_dir(&ctx.config.workspace.path)
        .output();

    match deny_result {
        Err(_) => findings.push(Finding {
            severity: Severity::Info,
            phase: "deps".into(),
            crate_name: "workspace".into(),
            message: "cargo-deny not installed; skipping".into(),
            file: None,
            line: None,
        }),
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if tool_not_found(&stderr) {
                findings.push(Finding {
                    severity: Severity::Info,
                    phase: "deps".into(),
                    crate_name: "workspace".into(),
                    message: "cargo-deny not installed; skipping".into(),
                    file: None,
                    line: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::High,
                    phase: "deps".into(),
                    crate_name: "workspace".into(),
                    message: "cargo deny check failed".into(),
                    file: None,
                    line: None,
                });
            }
        }
        Ok(_) => {}
    }

    // cargo-audit
    let audit_result = Command::new("cargo")
        .args(["audit"])
        .current_dir(&ctx.config.workspace.path)
        .output();

    match audit_result {
        Err(_) => findings.push(Finding {
            severity: Severity::Info,
            phase: "deps".into(),
            crate_name: "workspace".into(),
            message: "cargo-audit not installed; skipping".into(),
            file: None,
            line: None,
        }),
        Ok(out) if !out.status.success() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if tool_not_found(&stderr) {
                findings.push(Finding {
                    severity: Severity::Info,
                    phase: "deps".into(),
                    crate_name: "workspace".into(),
                    message: "cargo-audit not installed; skipping".into(),
                    file: None,
                    line: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::High,
                    phase: "deps".into(),
                    crate_name: "workspace".into(),
                    message: "cargo audit found vulnerabilities".into(),
                    file: None,
                    line: None,
                });
            }
        }
        Ok(_) => {}
    }

    let report = Report { phase: "deps".into(), findings };
    report.print(&ctx.format);
    Ok(report.exit_code())
}
