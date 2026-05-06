use crate::AppContext;
use anyhow::Result;
use prometheus_rust_auditor::reporter::{Finding, Report, Severity};

pub fn run(ctx: &AppContext) -> Result<i32> {
    let findings = ctx
        .config
        .workspace
        .partitions
        .iter()
        .map(|p| Finding {
            severity: Severity::Info,
            phase: "partition".into(),
            crate_name: p.name.clone(),
            message: "partition audit pending AI loop (phase 6–9)".into(),
            file: None,
            line: None,
        })
        .collect();

    let report = Report { phase: "partition".into(), findings };
    report.print(&ctx.format);
    Ok(0)
}
