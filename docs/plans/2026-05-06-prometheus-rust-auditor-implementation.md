# prometheus-rust-auditor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a installable Rust CLI binary (`prometheus-rust-auditor`) and matching Claude Code skill + agent that together form a staged autonomous Rust code quality remediation pipeline.

**Architecture:** The binary handles all deterministic phases (Clippy, fmt, cargo-deny, cargo-audit, geiger, inventory/partition, CI gen) and emits structured JSON. The skill (`/rust-auditor`) reads that JSON and orchestrates the AI audit loop per domain partition. The agent (`agents/rust-auditor.md`) handles multi-session orchestration.

**Tech Stack:** Rust + clap 4 + serde_json + anyhow + thiserror + tokio (minimal), `cargo metadata`, `cargo clippy`, `cargo fmt`, `cargo-deny`, `cargo-audit`, `cargo-geiger`, agentskills.io skill format.

**Scope:** Phases 1–5 + 10 fully implemented; Phases 6–9 stubbed with TODOs.

---

## Task 1: Create the binary crate skeleton

**Files:**
- Create: `tools/prometheus-rust-auditor/Cargo.toml`
- Create: `tools/prometheus-rust-auditor/src/main.rs`

**Step 1: Create the Cargo.toml**

```toml
[package]
name        = "prometheus-rust-auditor"
description = "Staged autonomous Rust code quality remediation pipeline for Prometheus AGS projects"
version     = "0.1.0"
edition     = "2021"
authors     = ["Travis James <travis@prometheusags.ai>"]
license     = "MIT"
repository  = "https://github.com/Prometheus-AGS/prometheus-skill-pack"

[[bin]]
name = "prometheus-rust-auditor"
path = "src/main.rs"

[dependencies]
anyhow      = "1"
thiserror   = "2"
clap        = { version = "4", features = ["derive"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
toml        = "0.8"
glob        = "0.3"
tikv-jemallocator = "0.6"

[dev-dependencies]
tempfile    = "3"
assert_cmd  = "2"
predicates  = "3"
```

**Step 2: Create src/main.rs skeleton**

```rust
//! prometheus-rust-auditor — staged Rust code quality remediation pipeline.
//!
//! Commands:
//!   audit      Run the full pipeline (phases 1-10)
//!   enforce    Phase 1-2: strict Clippy + cargo fmt
//!   deps       Phase 3: cargo-deny + cargo-audit
//!   inventory  Phases 4-5: geiger scan + crate graph + partition JSON
//!   ci         Phase 10: generate .github/workflows/rust-quality.yml
//!   config     Print default prometheus-auditor.toml to stdout

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod phases;
mod reporter;
mod scanner;

#[derive(Parser)]
#[command(
    name = "prometheus-rust-auditor",
    version,
    about = "Staged autonomous Rust code quality remediation for Prometheus AGS projects"
)]
struct Cli {
    /// Path to prometheus-auditor.toml (default: workspace root)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Output format: text | json | sarif
    #[arg(long, global = true, default_value = "text")]
    output: reporter::OutputFormat,

    /// Show subprocess output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the full pipeline (all enabled phases)
    Audit {
        /// Apply auto-fixable issues
        #[arg(long)]
        fix: bool,
        /// Run Phase 6-9 AI loop via claude --headless (STUBBED)
        #[arg(long)]
        autonomous: bool,
        /// Scope to one named partition
        #[arg(long)]
        partition: Option<String>,
        /// Workspace path (default: current directory)
        #[arg(default_value = ".")]
        workspace: PathBuf,
    },
    /// Phase 1-2: strict Clippy + cargo fmt
    Enforce {
        #[arg(long)]
        fix: bool,
        #[arg(default_value = ".")]
        workspace: PathBuf,
    },
    /// Phase 3: cargo-deny + cargo-audit
    Deps {
        #[arg(default_value = ".")]
        workspace: PathBuf,
    },
    /// Phases 4-5: geiger + crate graph + partition JSON
    Inventory {
        #[arg(default_value = ".")]
        workspace: PathBuf,
    },
    /// Phase 10: generate .github/workflows/rust-quality.yml
    Ci {
        #[arg(default_value = ".")]
        workspace: PathBuf,
    },
    /// Print default prometheus-auditor.toml to stdout
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = build_context(&cli)?;

    let exit_code = match cli.command {
        Command::Audit { fix, autonomous, partition, workspace } => {
            phases::run_audit(&ctx, &workspace, fix, autonomous, partition.as_deref())?
        }
        Command::Enforce { fix, workspace } => phases::run_enforce(&ctx, &workspace, fix)?,
        Command::Deps { workspace } => phases::run_deps(&ctx, &workspace)?,
        Command::Inventory { workspace } => phases::run_inventory(&ctx, &workspace)?,
        Command::Ci { workspace } => phases::run_ci(&ctx, &workspace)?,
        Command::Config => {
            print!("{}", config::DEFAULT_TOML);
            0
        }
    };

    std::process::exit(exit_code);
}

fn build_context(cli: &Cli) -> Result<AppContext> {
    let cfg = config::load(cli.config.as_deref())?;
    Ok(AppContext {
        cfg,
        output: cli.output.clone(),
        verbose: cli.verbose,
    })
}

pub struct AppContext {
    pub cfg: config::AuditorConfig,
    pub output: reporter::OutputFormat,
    pub verbose: bool,
}
```

**Step 3: Verify the file structure exists**

```bash
ls tools/prometheus-rust-auditor/src/
```
Expected: `main.rs`

**Step 4: Commit**

```bash
git add tools/prometheus-rust-auditor/
git commit -m "feat(auditor): scaffold prometheus-rust-auditor binary crate"
```

---

## Task 2: Config loader (`config.rs`)

**Files:**
- Create: `tools/prometheus-rust-auditor/src/config.rs`
- Create: `tools/prometheus-rust-auditor/tests/config_test.rs`

**Step 1: Write the failing test**

```rust
// tests/config_test.rs
use prometheus_rust_auditor::config;
use std::io::Write;

#[test]
fn load_default_config_when_no_file() {
    let cfg = config::load(None).unwrap();
    assert_eq!(cfg.workspace.path, std::path::PathBuf::from("."));
}

#[test]
fn load_config_from_toml_file() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    write!(f, r#"
[workspace]
path = "/tmp/myworkspace"

[invariants]
actor_no_shared_mutable_state = true
"#).unwrap();
    let cfg = config::load(Some(f.path())).unwrap();
    assert_eq!(cfg.workspace.path, std::path::PathBuf::from("/tmp/myworkspace"));
    assert!(cfg.invariants.actor_no_shared_mutable_state);
}

#[test]
fn default_toml_is_valid_toml() {
    let parsed: toml::Value = toml::from_str(config::DEFAULT_TOML).unwrap();
    assert!(parsed.get("workspace").is_some());
}
```

**Step 2: Run tests to verify they fail**

```bash
cd tools/prometheus-rust-auditor
cargo test --test config_test 2>&1 | head -20
```
Expected: compile error — `config` module not found

**Step 3: Implement `config.rs`**

```rust
// src/config.rs
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_TOML: &str = include_str!("../default-config.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditorConfig {
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub invariants: InvariantsConfig,
    #[serde(default)]
    pub clippy: ClippyConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_dot")]
    pub path: PathBuf,
    #[serde(default)]
    pub partitions: Vec<PartitionDef>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self { path: PathBuf::from("."), partitions: vec![] }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PartitionDef {
    pub name: String,
    pub crates: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InvariantsConfig {
    #[serde(default)]
    pub actor_no_shared_mutable_state: bool,
    #[serde(default)]
    pub wasm_unsafe_confined: bool,
    #[serde(default)]
    pub async_cancellation_safe: bool,
    #[serde(default)]
    pub zero_copy_preference: bool,
    #[serde(default)]
    pub no_platform_coupling: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClippyConfig {
    #[serde(default = "default_true")]
    pub workspace_lints: bool,
    #[serde(default = "default_warn")]
    pub pedantic: String,
    #[serde(default = "default_warn")]
    pub nursery: String,
    #[serde(default = "default_warn")]
    pub cargo: String,
    #[serde(default = "default_deny")]
    pub unwrap_used: String,
    #[serde(default = "default_warn")]
    pub expect_used: String,
    #[serde(default = "default_deny")]
    pub panic: String,
    #[serde(default = "default_deny")]
    pub redundant_clone: String,
    #[serde(default = "default_deny")]
    pub await_holding_lock: String,
}

impl Default for ClippyConfig {
    fn default() -> Self {
        toml::from_str("").unwrap_or_else(|_| Self {
            workspace_lints: true,
            pedantic: "warn".into(),
            nursery: "warn".into(),
            cargo: "warn".into(),
            unwrap_used: "deny".into(),
            expect_used: "warn".into(),
            panic: "deny".into(),
            redundant_clone: "deny".into(),
            await_holding_lock: "deny".into(),
        })
    }
}

fn default_dot() -> PathBuf { PathBuf::from(".") }
fn default_true() -> bool { true }
fn default_warn() -> String { "warn".into() }
fn default_deny() -> String { "deny".into() }

pub fn load(path: Option<&Path>) -> Result<AuditorConfig> {
    let candidate = path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        PathBuf::from("prometheus-auditor.toml")
    });

    if candidate.exists() {
        let raw = std::fs::read_to_string(&candidate)
            .with_context(|| format!("reading {}", candidate.display()))?;
        toml::from_str(&raw)
            .with_context(|| format!("parsing {}", candidate.display()))
    } else {
        toml::from_str(DEFAULT_TOML).context("parsing built-in default config")
    }
}
```

**Step 4: Create `default-config.toml`** at `tools/prometheus-rust-auditor/default-config.toml`:

```toml
[workspace]
path = "."
partitions = [
  { name = "actor",       crates = ["*-actor", "*-supervisor"] },
  { name = "mcp",         crates = ["*-mcp", "*-protocol"] },
  { name = "wasm",        crates = ["*-wasm", "*-wasmtime"] },
  { name = "persistence", crates = ["*-store", "*-db"] },
  { name = "runtime",     crates = ["scheduler", "orchestration"] },
  { name = "networking",  crates = ["*-sync", "*-webrtc"] },
]

[invariants]
actor_no_shared_mutable_state = true
wasm_unsafe_confined = true
async_cancellation_safe = true
zero_copy_preference = true
no_platform_coupling = true

[clippy]
workspace_lints = true
pedantic = "warn"
nursery  = "warn"
cargo    = "warn"
unwrap_used    = "deny"
expect_used    = "warn"
panic          = "deny"
redundant_clone = "deny"
await_holding_lock = "deny"
```

**Step 5: Add `lib.rs` so tests can reference the crate**

Add to `Cargo.toml`:
```toml
[lib]
name = "prometheus_rust_auditor"
path = "src/lib.rs"
```

Create `src/lib.rs`:
```rust
pub mod config;
pub mod phases;
pub mod reporter;
pub mod scanner;
```

**Step 6: Run tests to verify they pass**

```bash
cd tools/prometheus-rust-auditor
cargo test --test config_test
```
Expected: 3 passing tests

**Step 7: Commit**

```bash
git add tools/prometheus-rust-auditor/
git commit -m "feat(auditor): add config loader with default prometheus-auditor.toml"
```

---

## Task 3: Reporter (`reporter.rs`)

**Files:**
- Create: `tools/prometheus-rust-auditor/src/reporter.rs`
- Create: `tools/prometheus-rust-auditor/tests/reporter_test.rs`

**Step 1: Write the failing test**

```rust
// tests/reporter_test.rs
use prometheus_rust_auditor::reporter::{Finding, OutputFormat, Report, Severity};
use std::str::FromStr;

#[test]
fn output_format_parses_from_str() {
    assert!(matches!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json));
    assert!(matches!(OutputFormat::from_str("text").unwrap(), OutputFormat::Text));
    assert!(OutputFormat::from_str("bogus").is_err());
}

#[test]
fn report_exit_code_zero_when_no_findings() {
    let r = Report { findings: vec![], phase: "enforce".into() };
    assert_eq!(r.exit_code(), 0);
}

#[test]
fn report_exit_code_one_when_findings_present() {
    let r = Report {
        findings: vec![Finding {
            severity: Severity::High,
            phase: "enforce".into(),
            crate_name: "my-crate".into(),
            message: "unwrap used".into(),
            file: None,
            line: None,
        }],
        phase: "enforce".into(),
    };
    assert_eq!(r.exit_code(), 1);
}

#[test]
fn json_output_is_valid_json() {
    let r = Report { findings: vec![], phase: "enforce".into() };
    let s = r.to_json().unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["findings"].as_array().unwrap().len(), 0);
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test --test reporter_test 2>&1 | head -10
```
Expected: compile error

**Step 3: Implement `reporter.rs`**

```rust
// src/reporter.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub phase: String,
    pub crate_name: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub phase: String,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn exit_code(&self) -> i32 {
        if self.findings.is_empty() { 0 } else { 1 }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    pub fn print(&self, fmt: &OutputFormat) {
        match fmt {
            OutputFormat::Json => {
                if let Ok(s) = self.to_json() { println!("{s}"); }
            }
            OutputFormat::Text => {
                if self.findings.is_empty() {
                    println!("✓ Phase [{}]: no findings", self.phase);
                } else {
                    println!("✗ Phase [{}]: {} finding(s)", self.phase, self.findings.len());
                    for f in &self.findings {
                        let loc = match (&f.file, &f.line) {
                            (Some(file), Some(line)) => format!(" @ {file}:{line}"),
                            (Some(file), None) => format!(" @ {file}"),
                            _ => String::new(),
                        };
                        println!("  [{:?}] {}{}", f.severity, f.message, loc);
                    }
                }
            }
            OutputFormat::Sarif => {
                // SARIF 2.1 stub — full implementation in Phase 6-9
                println!("{{\"version\":\"2.1.0\",\"runs\":[]}}");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Text,
    Json,
    Sarif,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "text"  => Ok(Self::Text),
            "json"  => Ok(Self::Json),
            "sarif" => Ok(Self::Sarif),
            other   => anyhow::bail!("unknown output format: {other}; use text|json|sarif"),
        }
    }
}
```

**Step 4: Run tests**

```bash
cargo test --test reporter_test
```
Expected: 4 passing

**Step 5: Commit**

```bash
git add tools/prometheus-rust-auditor/src/reporter.rs tools/prometheus-rust-auditor/tests/
git commit -m "feat(auditor): add structured reporter with JSON/text/SARIF output"
```

---

## Task 4: Workspace scanner (`scanner.rs`)

**Files:**
- Create: `tools/prometheus-rust-auditor/src/scanner.rs`
- Create: `tools/prometheus-rust-auditor/tests/scanner_test.rs`

**Step 1: Write the failing test**

```rust
// tests/scanner_test.rs
use prometheus_rust_auditor::scanner;
use std::path::PathBuf;

#[test]
fn discover_workspace_members_from_cargo_metadata() {
    // Use this repo's tools/forge-rs as a real workspace under test
    let forge_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("forge-rs");

    if !forge_path.exists() {
        return; // skip if not present in CI
    }

    let members = scanner::discover_workspace_members(&forge_path).unwrap();
    assert!(!members.is_empty(), "should find at least one crate");
    assert!(members.iter().any(|m| m.name.contains("forge")));
}

#[test]
fn glob_match_star_suffix() {
    assert!(scanner::glob_matches("my-actor", "*-actor"));
    assert!(scanner::glob_matches("session-actor", "*-actor"));
    assert!(!scanner::glob_matches("my-store", "*-actor"));
}

#[test]
fn partition_assigns_crates_to_domains() {
    use prometheus_rust_auditor::config::{AuditorConfig, PartitionDef, WorkspaceConfig};
    let cfg = AuditorConfig {
        workspace: WorkspaceConfig {
            path: ".".into(),
            partitions: vec![
                PartitionDef { name: "actor".into(), crates: vec!["*-actor".into()] },
                PartitionDef { name: "mcp".into(),   crates: vec!["*-mcp".into()] },
            ],
        },
        ..Default::default()
    };
    let members = vec![
        scanner::CrateMember { name: "session-actor".into(), path: ".".into() },
        scanner::CrateMember { name: "forge-mcp".into(),     path: ".".into() },
        scanner::CrateMember { name: "forge-core".into(),    path: ".".into() },
    ];
    let map = scanner::partition_members(&members, &cfg);
    assert_eq!(map["actor"].len(), 1);
    assert_eq!(map["mcp"].len(), 1);
    assert_eq!(map["unpartitioned"].len(), 1);
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test --test scanner_test 2>&1 | head -15
```

**Step 3: Implement `scanner.rs`**

```rust
// src/scanner.rs
use crate::config::AuditorConfig;
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateMember {
    pub name: String,
    pub path: PathBuf,
}

pub fn discover_workspace_members(workspace: &Path) -> Result<Vec<CrateMember>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .current_dir(workspace)
        .output()
        .context("running cargo metadata")?;

    if !output.status.success() {
        anyhow::bail!(
            "cargo metadata failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let meta: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("parsing cargo metadata JSON")?;

    let members = meta["packages"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let name = p["name"].as_str()?.to_owned();
            let manifest = p["manifest_path"].as_str()?;
            let path = PathBuf::from(manifest).parent()?.to_path_buf();
            Some(CrateMember { name, path })
        })
        .collect();

    Ok(members)
}

/// Match a crate name against a glob pattern (only `*` prefix/suffix supported).
pub fn glob_matches(crate_name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return crate_name.ends_with(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return crate_name.starts_with(prefix);
    }
    crate_name == pattern
}

/// Assign workspace members to named partitions from config.
/// Crates not matching any partition go into "unpartitioned".
pub fn partition_members(
    members: &[CrateMember],
    cfg: &AuditorConfig,
) -> HashMap<String, Vec<CrateMember>> {
    let mut map: HashMap<String, Vec<CrateMember>> = HashMap::new();

    for member in members {
        let assigned = cfg.workspace.partitions.iter().find(|p| {
            p.crates.iter().any(|pat| glob_matches(&member.name, pat))
        });
        let key = assigned.map(|p| p.name.clone()).unwrap_or_else(|| "unpartitioned".into());
        map.entry(key).or_default().push(member.clone());
    }

    Ok(map)
}

pub fn partition_members(
    members: &[CrateMember],
    cfg: &AuditorConfig,
) -> HashMap<String, Vec<CrateMember>> {
    let mut map: HashMap<String, Vec<CrateMember>> = HashMap::new();

    for member in members {
        let assigned = cfg.workspace.partitions.iter().find(|p| {
            p.crates.iter().any(|pat| glob_matches(&member.name, pat))
        });
        let key = assigned
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "unpartitioned".into());
        map.entry(key).or_default().push(member.clone());
    }

    map
}
```

> **Note:** The duplicate function above is a mistake introduced for illustration — remove the first `partition_members` implementation, keep only the second.

**Step 4: Run tests**

```bash
cargo test --test scanner_test
```
Expected: all passing (workspace test skips gracefully if forge-rs absent)

**Step 5: Commit**

```bash
git add tools/prometheus-rust-auditor/src/scanner.rs tools/prometheus-rust-auditor/tests/scanner_test.rs
git commit -m "feat(auditor): workspace scanner with cargo metadata + partition map"
```

---

## Task 5: Phase runners (`phases/`)

**Files:**
- Create: `tools/prometheus-rust-auditor/src/phases/mod.rs`
- Create: `tools/prometheus-rust-auditor/src/phases/enforce.rs`
- Create: `tools/prometheus-rust-auditor/src/phases/format.rs`
- Create: `tools/prometheus-rust-auditor/src/phases/deps.rs`
- Create: `tools/prometheus-rust-auditor/src/phases/inventory.rs`
- Create: `tools/prometheus-rust-auditor/src/phases/partition.rs`
- Create: `tools/prometheus-rust-auditor/src/phases/autonomous.rs`

**Step 1: Write the failing integration test**

```rust
// tests/phases_test.rs
use assert_cmd::Command;

#[test]
fn enforce_command_exits_cleanly_on_clean_workspace() {
    // Use the auditor's own source as a test target (it should be clean)
    let mut cmd = Command::cargo_bin("prometheus-rust-auditor").unwrap();
    cmd.args(["enforce", env!("CARGO_MANIFEST_DIR")])
       .assert()
       .success(); // exit 0 = clean
}

#[test]
fn config_command_emits_valid_toml() {
    let mut cmd = Command::cargo_bin("prometheus-rust-auditor").unwrap();
    let output = cmd.arg("config").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&stdout);
    assert!(parsed.is_ok(), "config output must be valid TOML");
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test --test phases_test 2>&1 | head -15
```
Expected: compile error (phases module missing)

**Step 3: Implement `phases/mod.rs`**

```rust
// src/phases/mod.rs
mod autonomous;
mod deps;
mod enforce;
mod format;
mod inventory;
mod partition;

pub use inventory::InventoryReport;

use crate::{reporter::Report, AppContext};
use anyhow::Result;
use std::path::Path;

pub fn run_audit(
    ctx: &AppContext,
    workspace: &Path,
    fix: bool,
    autonomous: bool,
    partition: Option<&str>,
) -> Result<i32> {
    let mut max_code = 0i32;

    max_code = max_code.max(run_enforce(ctx, workspace, fix)?);
    max_code = max_code.max(run_deps(ctx, workspace)?);
    max_code = max_code.max(run_inventory(ctx, workspace)?);
    max_code = max_code.max(run_ci(ctx, workspace)?);

    if autonomous {
        max_code = max_code.max(autonomous::run_stub(ctx, workspace, partition)?);
    }

    Ok(max_code)
}

pub fn run_enforce(ctx: &AppContext, workspace: &Path, fix: bool) -> Result<i32> {
    let r1 = enforce::run(ctx, workspace, fix)?;
    let r2 = format::run(ctx, workspace, fix)?;
    let code = r1.exit_code().max(r2.exit_code());
    r1.print(&ctx.output);
    r2.print(&ctx.output);
    Ok(code)
}

pub fn run_deps(ctx: &AppContext, workspace: &Path) -> Result<i32> {
    let report = deps::run(ctx, workspace)?;
    report.print(&ctx.output);
    Ok(report.exit_code())
}

pub fn run_inventory(ctx: &AppContext, workspace: &Path) -> Result<i32> {
    let report = inventory::run(ctx, workspace)?;
    report.print(&ctx.output);
    Ok(report.exit_code())
}

pub fn run_ci(ctx: &AppContext, workspace: &Path) -> Result<i32> {
    partition::run_ci(ctx, workspace)?;
    Ok(0)
}
```

**Step 4: Implement `phases/enforce.rs`**

```rust
// src/phases/enforce.rs
use crate::{reporter::{Finding, Report, Severity}, AppContext};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(ctx: &AppContext, workspace: &Path, fix: bool) -> Result<Report> {
    let mut args = vec![
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--message-format=json",
    ];
    if fix {
        args.extend_from_slice(&["--fix", "--allow-dirty"]);
    }
    args.extend_from_slice(&["--", "-D", "warnings"]);

    if ctx.verbose {
        eprintln!("[enforce] running: cargo {}", args.join(" "));
    }

    let output = Command::new("cargo")
        .args(&args)
        .current_dir(workspace)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let findings = parse_clippy_json_findings(&String::from_utf8_lossy(&output.stdout));

    if ctx.verbose && !stderr.is_empty() {
        eprintln!("{stderr}");
    }

    Ok(Report { phase: "enforce:clippy".into(), findings })
}

fn parse_clippy_json_findings(stdout: &str) -> Vec<Finding> {
    stdout.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|v| v["reason"] == "compiler-message")
        .filter_map(|v| {
            let msg = v["message"].as_object()?;
            let level = msg["level"].as_str()?;
            if !matches!(level, "error" | "warning") { return None; }
            let text = msg["message"].as_str()?.to_owned();
            let severity = if level == "error" { Severity::High } else { Severity::Medium };
            let (file, line) = msg["spans"].as_array()
                .and_then(|s| s.first())
                .map(|s| (
                    s["file_name"].as_str().map(|f| f.to_owned()),
                    s["line_start"].as_u64().map(|l| l as u32),
                ))
                .unwrap_or((None, None));
            Some(Finding {
                severity,
                phase: "enforce:clippy".into(),
                crate_name: String::new(),
                message: text,
                file,
                line,
            })
        })
        .collect()
}
```

**Step 5: Implement `phases/format.rs`**

```rust
// src/phases/format.rs
use crate::{reporter::{Finding, Report, Severity}, AppContext};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(ctx: &AppContext, workspace: &Path, fix: bool) -> Result<Report> {
    let mut args = vec!["fmt", "--all"];
    if !fix {
        args.push("--check");
    }

    if ctx.verbose {
        eprintln!("[format] running: cargo {}", args.join(" "));
    }

    let output = Command::new("cargo")
        .args(&args)
        .current_dir(workspace)
        .output()?;

    let findings = if !output.status.success() && !fix {
        vec![Finding {
            severity: Severity::Medium,
            phase: "enforce:fmt".into(),
            crate_name: "workspace".into(),
            message: "cargo fmt --check failed: unformatted files detected".into(),
            file: None,
            line: None,
        }]
    } else {
        vec![]
    };

    Ok(Report { phase: "enforce:fmt".into(), findings })
}
```

**Step 6: Implement `phases/deps.rs`**

```rust
// src/phases/deps.rs
use crate::{reporter::{Finding, Report, Severity}, AppContext};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(ctx: &AppContext, workspace: &Path) -> Result<Report> {
    let mut findings = vec![];

    // cargo deny check
    let deny = Command::new("cargo")
        .args(["deny", "check"])
        .current_dir(workspace)
        .output();

    match deny {
        Ok(out) if !out.status.success() => {
            findings.push(Finding {
                severity: Severity::High,
                phase: "deps:deny".into(),
                crate_name: "workspace".into(),
                message: format!(
                    "cargo deny check failed:\n{}",
                    String::from_utf8_lossy(&out.stderr).chars().take(500).collect::<String>()
                ),
                file: None,
                line: None,
            });
        }
        Err(_) => {
            findings.push(Finding {
                severity: Severity::Info,
                phase: "deps:deny".into(),
                crate_name: "workspace".into(),
                message: "cargo-deny not installed — skipped (run install-tools.sh)".into(),
                file: None,
                line: None,
            });
        }
        _ => {}
    }

    // cargo audit
    let audit = Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(workspace)
        .output();

    match audit {
        Ok(out) if !out.status.success() => {
            findings.push(Finding {
                severity: Severity::Critical,
                phase: "deps:audit".into(),
                crate_name: "workspace".into(),
                message: format!(
                    "cargo audit found vulnerabilities:\n{}",
                    String::from_utf8_lossy(&out.stdout).chars().take(500).collect::<String>()
                ),
                file: None,
                line: None,
            });
        }
        Err(_) => {
            findings.push(Finding {
                severity: Severity::Info,
                phase: "deps:audit".into(),
                crate_name: "workspace".into(),
                message: "cargo-audit not installed — skipped (run install-tools.sh)".into(),
                file: None,
                line: None,
            });
        }
        _ => {}
    }

    Ok(Report { phase: "deps".into(), findings })
}
```

**Step 7: Implement `phases/inventory.rs`**

```rust
// src/phases/inventory.rs
use crate::{reporter::{Finding, Report, Severity}, scanner, AppContext};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

#[derive(serde::Serialize)]
pub struct InventoryReport {
    pub crates: Vec<scanner::CrateMember>,
    pub partitions: std::collections::HashMap<String, Vec<scanner::CrateMember>>,
    pub unsafe_summary: UnsafeSummary,
}

#[derive(serde::Serialize, Default)]
pub struct UnsafeSummary {
    pub geiger_available: bool,
    pub raw_output: String,
}

pub fn run(ctx: &AppContext, workspace: &Path) -> Result<Report> {
    let members = scanner::discover_workspace_members(workspace)?;
    let partitions = scanner::partition_members(&members, &ctx.cfg);

    // cargo geiger (best-effort — not a hard failure if unavailable)
    let unsafe_summary = run_geiger(ctx, workspace);

    let inventory = InventoryReport {
        crates: members.clone(),
        partitions: partitions.clone(),
        unsafe_summary,
    };

    // Always print inventory JSON to stdout regardless of output format flag
    // (the skill reads this to know partition structure)
    println!("{}", serde_json::to_string_pretty(&inventory)?);

    let findings = if members.is_empty() {
        vec![Finding {
            severity: Severity::High,
            phase: "inventory".into(),
            crate_name: "workspace".into(),
            message: "No crates found — is this a Cargo workspace?".into(),
            file: None,
            line: None,
        }]
    } else {
        vec![]
    };

    Ok(Report { phase: "inventory".into(), findings })
}

fn run_geiger(ctx: &AppContext, workspace: &Path) -> UnsafeSummary {
    let result = Command::new("cargo")
        .args(["geiger", "--all-features"])
        .current_dir(workspace)
        .output();

    match result {
        Ok(out) => UnsafeSummary {
            geiger_available: true,
            raw_output: String::from_utf8_lossy(&out.stdout).chars().take(2000).collect(),
        },
        Err(_) => {
            if ctx.verbose {
                eprintln!("[inventory] cargo-geiger not installed — skipped");
            }
            UnsafeSummary::default()
        }
    }
}
```

**Step 8: Implement `phases/partition.rs` (CI generation)**

```rust
// src/partition.rs (lives in phases/ as ci.rs — rename)
// src/phases/partition.rs — re-export ci generation
use crate::AppContext;
use anyhow::{Context as _, Result};
use std::path::Path;

const CI_TEMPLATE: &str = r#"# Generated by prometheus-rust-auditor ci
# Enforces Prometheus AGS Rust quality standards
name: Rust Quality

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy (deny warnings)
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings

      - name: Tests
        run: cargo test --workspace --all-features

      - name: cargo-deny
        uses: EmbarkStudios/cargo-deny-action@v1
        continue-on-error: true

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: cargo audit
        uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
"#;

pub fn run_ci(ctx: &AppContext, workspace: &Path) -> Result<()> {
    let ci_dir = workspace.join(".github").join("workflows");
    std::fs::create_dir_all(&ci_dir).context("creating .github/workflows")?;
    let ci_path = ci_dir.join("rust-quality.yml");
    std::fs::write(&ci_path, CI_TEMPLATE)
        .with_context(|| format!("writing {}", ci_path.display()))?;
    if !matches!(ctx.output, crate::reporter::OutputFormat::Json) {
        eprintln!("✓ CI workflow written to {}", ci_path.display());
    }
    Ok(())
}
```

**Step 9: Implement `phases/autonomous.rs` (stub)**

```rust
// src/phases/autonomous.rs
use crate::AppContext;
use anyhow::Result;
use std::path::Path;

/// Phases 6-9: AI-driven audit loop via `claude --headless`.
///
/// TODO(phase-6-9): Implement the following:
///   1. For each partition in ctx.cfg.workspace.partitions:
///      a. Load INVARIANTS.md + AI_AUDIT_PROMPT.md from the skill's references/
///      b. Run: claude --headless --print "<prompt>" to get audit findings as JSON
///      c. Parse findings, generate patches, apply with `git apply`
///      d. Re-run enforce phase to verify fixes don't introduce new issues
///      e. If clean, commit; else rollback and report manual intervention needed
///   2. Phase 7: criterion benchmarks — run before/after, compare allocations
///   3. Phase 8: loom concurrency test — cargo test --features loom
///   4. Phase 9: gh pr create with structured findings summary
pub fn run_stub(_ctx: &AppContext, _workspace: &Path, _partition: Option<&str>) -> Result<i32> {
    eprintln!(
        "⚠ --autonomous mode is not yet implemented (Phases 6-9 stub).\n\
         Use the /rust-auditor skill in Claude Code to run the AI audit loop manually."
    );
    Ok(0)
}
```

**Step 10: Run integration tests**

```bash
cargo build && cargo test --test phases_test
```
Expected: both tests pass

**Step 11: Commit**

```bash
git add tools/prometheus-rust-auditor/src/phases/
git commit -m "feat(auditor): implement phases 1-5 + 10 (enforce, fmt, deps, inventory, CI gen)"
```

---

## Task 6: Wire `main.rs` to compile clean

**Files:**
- Modify: `tools/prometheus-rust-auditor/src/main.rs` (ensure `build_context` uses correct types)

**Step 1: Full build check**

```bash
cd tools/prometheus-rust-auditor
cargo build 2>&1
```

If there are type errors (e.g. `AppContext` missing `Default`, `OutputFormat` needing `Clone`):
- Add `#[derive(Clone)]` to `OutputFormat` in `reporter.rs`
- Add `impl Default for AuditorConfig` in `config.rs` (delegates to `toml::from_str(DEFAULT_TOML).unwrap()`)

**Step 2: Run all tests**

```bash
cargo test
```
Expected: all tests pass, no warnings treated as errors

**Step 3: Smoke test the binary**

```bash
cargo run -- --help
cargo run -- config
cargo run -- enforce .
```

**Step 4: Commit**

```bash
git add tools/prometheus-rust-auditor/
git commit -m "fix(auditor): wire main.rs, ensure clean build + all tests green"
```

---

## Task 7: Install script and tooling prerequisites

**Files:**
- Create: `skills/rust/prometheus-rust-auditor/scripts/install-tools.sh`

**Step 1: Create the script**

```bash
#!/usr/bin/env bash
# install-tools.sh — install deterministic audit tooling for prometheus-rust-auditor
set -euo pipefail

echo "Installing prometheus-rust-auditor tooling..."

# cargo-deny: dependency policy enforcement
if ! command -v cargo-deny &>/dev/null; then
    echo "  Installing cargo-deny..."
    cargo install cargo-deny --locked
fi

# cargo-audit: CVE / RUSTSEC database scan
if ! command -v cargo-audit &>/dev/null; then
    echo "  Installing cargo-audit..."
    cargo install cargo-audit --locked
fi

# cargo-geiger: unsafe usage map
if ! command -v cargo-geiger &>/dev/null; then
    echo "  Installing cargo-geiger..."
    cargo install cargo-geiger --locked
fi

echo "All tools installed."
echo ""
echo "Next: cargo install --path tools/prometheus-rust-auditor"
echo "Then: prometheus-rust-auditor config > prometheus-auditor.toml"
```

**Step 2: Make executable**

```bash
chmod +x skills/rust/prometheus-rust-auditor/scripts/install-tools.sh
```

**Step 3: Commit**

```bash
git add skills/rust/prometheus-rust-auditor/scripts/
git commit -m "feat(auditor): add install-tools.sh for audit toolchain prerequisites"
```

---

## Task 8: Reference documents

**Files:**
- Create: `skills/rust/prometheus-rust-auditor/references/INVARIANTS.md`
- Create: `skills/rust/prometheus-rust-auditor/references/AI_AUDIT_PROMPT.md`

**Step 1: Create `INVARIANTS.md`**

```markdown
# Prometheus AGS Rust Architectural Invariants

These invariants are loaded by the `/rust-auditor` skill and encoded as hard
constraints in every AI audit prompt. Violations at any invariant marked **CRITICAL**
must be fixed before a partition is considered clean.

## Actor System Invariants

| ID | Invariant | Severity | Crate Pattern |
|----|-----------|----------|---------------|
| ACT-01 | No shared mutable state — actors communicate via mpsc channels only | CRITICAL | `*-actor`, `*-supervisor` |
| ACT-02 | No `Arc<Mutex<T>>` inside actor state structs | HIGH | `*-actor` |
| ACT-03 | All actor `run()` loops must handle `Shutdown` message and return cleanly | HIGH | `*-actor` |

## WASM Safety Invariants

| ID | Invariant | Severity | Crate Pattern |
|----|-----------|----------|---------------|
| WASM-01 | `unsafe` code confined to `*-wasm` and `*-wasmtime` crates only | CRITICAL | all |
| WASM-02 | No `std::process::exit` in WASM-capable crates | HIGH | `*-wasm` |

## Async Safety Invariants

| ID | Invariant | Severity | Crate Pattern |
|----|-----------|----------|---------------|
| ASYNC-01 | No `parking_lot::Mutex` or `std::sync::Mutex` guard held across `.await` | CRITICAL | all |
| ASYNC-02 | All `tokio::spawn` tasks must be `Send + 'static` | HIGH | all |
| ASYNC-03 | `select!` arms must be cancellation-safe | HIGH | all |

## Zero-Copy / Allocation Invariants

| ID | Invariant | Severity | Crate Pattern |
|----|-----------|----------|---------------|
| ALLOC-01 | No `.clone()` in hot-path loops (>1000 calls/sec paths) | WARN | all |
| ALLOC-02 | Accept `&[T]` and `&str` at API boundaries, not `Vec<T>` or `String` | WARN | `*-core` |
| ALLOC-03 | No `Box<dyn Future>` unless behind a trait object that crosses a crate boundary | WARN | all |

## Core / Platform Coupling Invariants

| ID | Invariant | Severity | Crate Pattern |
|----|-----------|----------|---------------|
| CORE-01 | `*-core` crates must not depend on any other workspace crate | CRITICAL | `*-core` |
| CORE-02 | No `cfg(target_os)` in `*-core` — platform-specific code belongs in `*-cli` or `*-mcp` | HIGH | `*-core` |
```

**Step 2: Create `AI_AUDIT_PROMPT.md`**

````markdown
# Canonical Per-Domain AI Audit Prompt

This template is used by the `/rust-auditor` skill for each partition.
Replace `{{PARTITION_NAME}}`, `{{CRATE_LIST}}`, and `{{ACTIVE_INVARIANTS}}` before sending.

---

You are performing a strict enterprise Rust audit of the **{{PARTITION_NAME}}** partition.

**Crates in scope:** {{CRATE_LIST}}

**Active architectural invariants:**
{{ACTIVE_INVARIANTS}}

Apply ALL of the following standards:
- Microsoft Pragmatic Rust Guidelines
- Rust API Guidelines (https://rust-lang.github.io/api-guidelines/)
- Effective Rust (idiomatic ownership, error propagation, trait design)
- Clippy pedantic + nursery (treat all warnings as findings)
- Tokio async best practices (cancellation safety, task lifecycle)
- Zero-copy and ownership-first design (avoid clone proliferation)
- Actor-model concurrency correctness (no shared mutable state)

Your task:
1. Identify all violations of the standards and invariants above
2. Explain WHY each is a violation (which rule/invariant it breaks)
3. Provide the exact code diff to fix it
4. Preserve all existing semantics
5. Never introduce new `.clone()` calls to fix a borrow issue — restructure ownership instead
6. Never introduce heap allocation increases
7. Preserve async cancellation safety in all fixes
8. Preserve Send + Sync correctness
9. Preserve WASM safety boundaries (no unsafe outside WASM crates)

Return structured JSON with this schema:
```json
{
  "partition": "{{PARTITION_NAME}}",
  "findings": [
    {
      "severity": "critical|high|medium|low",
      "invariant_id": "ACT-01",
      "crate": "session-actor",
      "file": "src/actor.rs",
      "line": 42,
      "description": "Arc<Mutex<State>> used in actor struct — violates ACT-02",
      "diff": "--- a/src/actor.rs\n+++ b/src/actor.rs\n..."
    }
  ],
  "architectural_concerns": [],
  "performance_concerns": [],
  "concurrency_concerns": []
}
```
````

**Step 3: Commit**

```bash
git add skills/rust/prometheus-rust-auditor/references/
git commit -m "feat(auditor): add INVARIANTS.md and canonical AI_AUDIT_PROMPT.md"
```

---

## Task 9: SKILL.md — the `/rust-auditor` slash command

**Files:**
- Create: `skills/rust/prometheus-rust-auditor/SKILL.md`

**Step 1: Create the skill**

```markdown
---
license: MIT
name: prometheus-rust-auditor
version: '1.0.0'
description: >
  Staged autonomous Rust code quality remediation pipeline for Prometheus AGS projects.
  Runs deterministic enforcement (Clippy, fmt, cargo-deny, cargo-audit, geiger), generates
  a partition-aware architectural inventory, and orchestrates per-domain AI audits guided
  by Prometheus invariants. Use when auditing any Rust workspace for quality, safety, and
  architectural correctness. Invoke with /rust-auditor.
language: rust
argument-hint: "[workspace_path] [--partition <name>] [--fix]"
metadata:
  tags: [rust, quality, audit, clippy, cargo, enforcement, architecture]
---

# Prometheus Rust Auditor

## Prerequisites

Before using this skill, ensure the binary is installed:

```bash
# Install binary from skill pack
cargo install --path tools/prometheus-rust-auditor

# Install tooling (cargo-deny, cargo-audit, cargo-geiger)
bash scripts/install-tools.sh
```

If `prometheus-rust-auditor --help` fails, stop and run the above.

## Workflow

### Step 1 — Prerequisite check

Run:
```bash
prometheus-rust-auditor --help
```

If not found: emit install instructions above and stop.

### Step 2 — Inventory

Run:
```bash
prometheus-rust-auditor inventory --output json <workspace_path>
```

Parse the JSON output to extract:
- `crates[]` — full workspace member list
- `partitions{}` — crate-to-domain mapping
- `unsafe_summary` — geiger results

### Step 3 — Deterministic enforcement

Run phases 1-5 with auto-fix:
```bash
prometheus-rust-auditor enforce --fix <workspace_path>
prometheus-rust-auditor deps <workspace_path>
```

Report findings. If CRITICAL findings remain after `--fix`, stop and report manual intervention required.

### Step 4 — Load invariants

Read `references/INVARIANTS.md` and `references/AI_AUDIT_PROMPT.md` from this skill directory.
Read `prometheus-auditor.toml` in the workspace root to get active invariant flags and partition config.

### Step 5 — AI audit loop (per partition)

For each partition in the inventory JSON:

1. Build the audit prompt from `AI_AUDIT_PROMPT.md` substituting:
   - `{{PARTITION_NAME}}` — partition name
   - `{{CRATE_LIST}}` — comma-separated crate names from partition
   - `{{ACTIVE_INVARIANTS}}` — invariants from `prometheus-auditor.toml` that are `true`

2. Read the source files for each crate in the partition

3. Apply the audit prompt to the code, identifying violations

4. For each finding with a diff: apply the patch using Edit tool

5. Re-run `prometheus-rust-auditor enforce` to verify no regressions introduced

6. If clean: commit with message `fix(rust-audit): <partition> — <summary of fixes>`

7. Repeat for next partition

### Step 6 — CI enforcement

```bash
prometheus-rust-auditor ci <workspace_path>
```

Verify `.github/workflows/rust-quality.yml` was written. Commit if new.

### Step 7 — Summary

Emit a findings summary:
- Total findings before / after
- Partitions audited
- Invariant violations resolved
- CI workflow status
- Any remaining manual items

## Partition Scoping

To audit a single partition only:
```bash
prometheus-rust-auditor audit --partition actor <workspace_path>
```

Then in Step 5, only process that partition.

## References

- [Architectural Invariants](references/INVARIANTS.md) — hard constraints per domain
- [AI Audit Prompt](references/AI_AUDIT_PROMPT.md) — canonical per-partition prompt template
```

**Step 2: Validate the skill**

```bash
npm run validate:strict skills/rust/prometheus-rust-auditor
```
Expected: no errors

**Step 3: Commit**

```bash
git add skills/rust/prometheus-rust-auditor/SKILL.md
git commit -m "feat(auditor): add /rust-auditor SKILL.md slash command"
```

---

## Task 10: Agent orchestrator (`agents/rust-auditor.md`)

**Files:**
- Create: `agents/rust-auditor.md`

**Step 1: Create the agent**

```markdown
---
name: rust-auditor
description: >
  Orchestrating agent for staged Prometheus Rust code quality remediation. Coordinates the
  prometheus-rust-auditor binary (deterministic phases) and the /rust-auditor skill (AI audit
  phases) in the correct sequence. Handles multi-session state, partition checkpointing, and
  architectural invariant enforcement. Use for complex end-to-end Rust quality audits that
  span multiple partitions or require resuming across context resets.
---

# Rust Auditor Agent

You are the Rust Auditor for Prometheus AGS. You orchestrate the prometheus-rust-auditor
binary and /rust-auditor skill to deliver complete, standards-compliant Rust workspaces.

## Your Responsibilities

1. **Understand scope** — ask the minimum set of questions:
   - Target workspace path
   - Which partitions to audit (all, or specific subset)
   - Whether `--fix` auto-apply is desired
   - Whether CI generation is needed

2. **Run detection** — before planning, always run:
   ```bash
   prometheus-rust-auditor inventory --output json <workspace>
   ```
   Ground the plan in reality. Never assume partition structure.

3. **Produce a work plan** — list every phase and partition in order.
   Present the plan and get explicit approval before executing.

4. **Track partition state** — maintain a checkpoint of which partitions are:
   - `pending` — not yet audited
   - `in_progress` — currently being audited
   - `clean` — audited, no remaining findings
   - `manual_required` — findings that couldn't be auto-fixed

5. **Delegate to binary + skill** — execute in this order:
   - Phase 1-2 (enforce): `prometheus-rust-auditor enforce --fix <workspace>`
   - Phase 3 (deps): `prometheus-rust-auditor deps <workspace>`
   - Phase 4-5 (inventory): `prometheus-rust-auditor inventory --output json <workspace>`
   - Phase 6 (AI audit): invoke `/rust-auditor` skill per partition
   - Phase 10 (CI): `prometheus-rust-auditor ci <workspace>`

6. **Invariant enforcement** — every proposed fix MUST be checked against
   `skills/rust/prometheus-rust-auditor/references/INVARIANTS.md`.
   Never accept a fix that resolves a borrow error by adding `.clone()` if an
   ownership restructuring is viable.

7. **Summarize results** — after completion:
   - Findings before / after per partition
   - Invariant violations resolved
   - CI workflow written
   - Any CRITICAL items requiring manual attention

## Resuming Across Sessions

If context resets mid-audit, ask the user:
> "I'm resuming the Rust audit. Last checkpoint: [partition X was in_progress].
> Should I re-run the enforce phase before continuing, or proceed from inventory?"

Always re-run `prometheus-rust-auditor inventory` at session start to confirm workspace state.

## Architectural Invariants (Hard Constraints)

These are non-negotiable. No fix is acceptable that violates them:

- **ACT-01**: Actors communicate via channels only — no shared mutable state
- **WASM-01**: `unsafe` confined to `*-wasm` crates only
- **ASYNC-01**: No mutex guard held across `.await`
- **CORE-01**: `*-core` crates have zero workspace dependencies

See full invariant table in `skills/rust/prometheus-rust-auditor/references/INVARIANTS.md`.
```

**Step 2: Commit**

```bash
git add agents/rust-auditor.md
git commit -m "feat(auditor): add rust-auditor orchestrator agent"
```

---

## Task 11: Register skill in `plugin.json` and validate

**Files:**
- Modify: `.claude-plugin/plugin.json`

**Step 1: Add to skills array in `plugin.json`**

Open `.claude-plugin/plugin.json` and add `"./skills/rust/prometheus-rust-auditor"` to the `"skills"` array.

**Step 2: Run full validation**

```bash
npm run validate:strict skills/rust/prometheus-rust-auditor
npm run build
```
Expected: no errors

**Step 3: End-to-end smoke test**

```bash
# Install the binary
cargo install --path tools/prometheus-rust-auditor

# Verify CLI works
prometheus-rust-auditor --help
prometheus-rust-auditor config
prometheus-rust-auditor inventory --output json tools/prometheus-rust-auditor

# Verify skill is available
npm run validate:strict skills/rust/prometheus-rust-auditor
```

**Step 4: Final commit**

```bash
git add .claude-plugin/plugin.json
git commit -m "feat(auditor): register prometheus-rust-auditor in plugin manifest"
```

---

## Verification Checklist

- [ ] `cargo build` in `tools/prometheus-rust-auditor/` — exits 0
- [ ] `cargo install --path tools/prometheus-rust-auditor` — binary on PATH
- [ ] `prometheus-rust-auditor --help` — renders CLI surface
- [ ] `prometheus-rust-auditor config` — emits valid TOML
- [ ] `prometheus-rust-auditor enforce tools/prometheus-rust-auditor` — exits 0 (own source is clean)
- [ ] `prometheus-rust-auditor inventory --output json tools/forge-rs` — valid JSON with partition map
- [ ] `prometheus-rust-auditor ci /tmp/testrepo` — writes `.github/workflows/rust-quality.yml`
- [ ] `npm run validate:strict skills/rust/prometheus-rust-auditor` — no errors
- [ ] `cargo test` in binary crate — all tests green
- [ ] `/rust-auditor` in Claude Code — skill loads, reads inventory, applies AI audit to first partition
