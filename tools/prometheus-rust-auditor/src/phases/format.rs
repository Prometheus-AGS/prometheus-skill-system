use crate::AppContext;
use anyhow::Result;
use prometheus_rust_auditor::reporter::{Finding, Report, Severity};
use std::process::Command;

pub fn run(ctx: &AppContext) -> Result<i32> {
    // Capture stdout so cargo fmt's diff doesn't corrupt our report output.
    // Emit it to stderr only in verbose mode.
    let output = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .current_dir(&ctx.config.workspace.path)
        .output()?;

    if ctx.verbose && !output.stdout.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
    }

    let status = output.status;

    let findings = if status.success() {
        vec![]
    } else {
        vec![Finding {
            severity: Severity::High,
            phase: "format".into(),
            crate_name: "workspace".into(),
            message: "cargo fmt check failed: run `cargo fmt --all` to fix".into(),
            file: None,
            line: None,
        }]
    };

    let report = Report { phase: "format".into(), findings };
    report.print(&ctx.format);
    Ok(report.exit_code())
}
