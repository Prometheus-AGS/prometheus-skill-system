use crate::AppContext;
use anyhow::Result;
use prometheus_rust_auditor::reporter::{Finding, Report, Severity};
use std::process::Command;

pub fn run(ctx: &AppContext) -> Result<i32> {
    let output = Command::new("cargo")
        .args(["clippy", "--workspace", "--message-format=json"])
        .current_dir(&ctx.config.workspace.path)
        .output()?;

    let mut findings = Vec::new();

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }
        let msg = match v.get("message") {
            Some(m) => m,
            None => continue,
        };
        let level = msg.get("level").and_then(|l| l.as_str()).unwrap_or("");
        let severity = match level {
            "error" => Severity::High,
            "warning" => Severity::Medium,
            _ => continue,
        };
        let text = msg
            .get("rendered")
            .and_then(|r| r.as_str())
            .or_else(|| msg.get("message").and_then(|m| m.as_str()))
            .unwrap_or("unknown")
            .to_owned();
        let crate_name = v
            .get("target")
            .and_then(|t| t.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("workspace")
            .to_owned();
        let file = msg
            .get("spans")
            .and_then(|s| s.get(0))
            .and_then(|s| s.get("file_name"))
            .and_then(|f| f.as_str())
            .map(|s| s.to_owned());
        let line_no = msg
            .get("spans")
            .and_then(|s| s.get(0))
            .and_then(|s| s.get("line_start"))
            .and_then(|l| l.as_u64())
            .map(|n| n as u32);

        findings.push(Finding {
            severity,
            phase: "enforce".into(),
            crate_name,
            message: text,
            file,
            line: line_no,
        });
    }

    let report = Report { phase: "enforce".into(), findings };
    report.print(&ctx.format);
    Ok(report.exit_code())
}
