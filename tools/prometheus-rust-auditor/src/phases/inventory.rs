use crate::AppContext;
use anyhow::Result;
use prometheus_rust_auditor::reporter::{Finding, Report, Severity};
use prometheus_rust_auditor::scanner::{assign_partitions, discover_workspace_crates};
use std::collections::BTreeMap;

pub fn run(ctx: &AppContext) -> Result<i32> {
    let crates = match discover_workspace_crates(&ctx.config.workspace.path) {
        Ok(c) => c,
        Err(e) => {
            let report = Report {
                phase: "inventory".into(),
                findings: vec![Finding {
                    severity: Severity::High,
                    phase: "inventory".into(),
                    crate_name: "workspace".into(),
                    message: format!("cargo metadata failed: {e}"),
                    file: None,
                    line: None,
                }],
            };
            report.print(&ctx.format);
            return Ok(report.exit_code());
        }
    };

    let assigned = assign_partitions(&crates, &ctx.config.workspace.partitions);

    if ctx.verbose {
        let mut by_partition: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (partition, krate) in &assigned {
            by_partition.entry(partition.as_str()).or_default().push(&krate.name);
        }
        eprintln!("Inventory: {} crates across {} partitions", crates.len(), by_partition.len());
        for (partition, names) in &by_partition {
            eprintln!("  {partition}: [{}]", names.join(", "));
        }
    }

    let report = Report { phase: "inventory".into(), findings: vec![] };
    report.print(&ctx.format);
    Ok(report.exit_code())
}
