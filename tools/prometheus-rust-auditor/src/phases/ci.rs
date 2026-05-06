use crate::AppContext;
use anyhow::{Context as _, Result};
use prometheus_rust_auditor::reporter::{Finding, Report, Severity};
use std::fs;

const WORKFLOW_YAML: &str = r#"name: Rust Audit

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Tests
        run: cargo test --workspace
"#;

pub fn run(ctx: &AppContext) -> Result<i32> {
    let workflows_dir = ctx.config.workspace.path.join(".github/workflows");
    fs::create_dir_all(&workflows_dir)
        .with_context(|| format!("creating {}", workflows_dir.display()))?;

    let workflow_path = workflows_dir.join("rust-audit.yml");
    fs::write(&workflow_path, WORKFLOW_YAML)
        .with_context(|| format!("writing {}", workflow_path.display()))?;

    let report = Report {
        phase: "ci".into(),
        findings: vec![Finding {
            severity: Severity::Info,
            phase: "ci".into(),
            crate_name: "workspace".into(),
            message: "generated .github/workflows/rust-audit.yml".into(),
            file: Some(workflow_path.to_string_lossy().into_owned()),
            line: None,
        }],
    };
    report.print(&ctx.format);
    Ok(0)
}
