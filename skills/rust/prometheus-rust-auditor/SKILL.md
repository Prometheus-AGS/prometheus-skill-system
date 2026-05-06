---
name: prometheus-rust-auditor
description: Staged autonomous Rust code quality remediation pipeline for Prometheus AGS projects. Runs Clippy enforcement, formatting checks, dependency policy, workspace inventory, partition-based architectural invariant audits, and CI generation. Use when auditing a Rust workspace, remediating code quality issues, or generating CI workflows.
version: '1.0.0'
license: MIT
metadata:
  author: travis-james
  category: rust
  tags: [rust, auditor, clippy, quality, invariants, ci, workspace, cargo]
argument-hint: "[phase] [--config path] [--format text|json|sarif] [--verbose]"
---

# prometheus-rust-auditor

Staged autonomous Rust code quality remediation pipeline.

## When to Use

- Auditing a Rust workspace for code quality issues
- Enforcing Clippy lints, formatting, and dependency policy in CI
- Generating a `.github/workflows/rust-audit.yml` CI workflow
- Mapping workspace crates to architectural partitions
- Running the full audit pipeline before a release or PR merge

## Quick Start

Install the binary and optional tools:

```bash
bash scripts/install-tools.sh
```

Run the full audit pipeline:

```bash
prometheus-rust-auditor audit
```

Run a specific phase:

```bash
prometheus-rust-auditor enforce          # Phase 1: Clippy
prometheus-rust-auditor format           # Phase 2: cargo fmt check
prometheus-rust-auditor deps             # Phase 3: cargo-deny + cargo-audit
prometheus-rust-auditor inventory        # Phase 4: workspace crate map
prometheus-rust-auditor partition        # Phase 5: architectural invariants
prometheus-rust-auditor ci               # Phase 10: generate CI workflow
prometheus-rust-auditor autonomous       # Phases 6-9: AI audit loop (stub)
```

## Configuration

Create `prometheus-auditor.toml` in your workspace root (optional — built-in defaults work out of the box):

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

## Output Formats

```bash
# Human-readable (default)
prometheus-rust-auditor audit --format text

# Machine-readable JSON
prometheus-rust-auditor audit --format json

# SARIF 2.1 (for GitHub Code Scanning)
prometheus-rust-auditor audit --format sarif
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0    | Clean — no findings |
| 1    | Findings present |
| 2    | Tool/config error |

## Phase Pipeline

| Phase | Command | Tool | Description |
|-------|---------|------|-------------|
| 1 | enforce | cargo clippy | Lint enforcement |
| 2 | format | cargo fmt | Style check |
| 3 | deps | cargo-deny, cargo-audit | Dep policy + CVE scan |
| 4 | inventory | cargo metadata | Workspace crate map |
| 5 | partition | (config-driven) | Invariant stubs by partition |
| 6–9 | autonomous | claude --headless | AI invariant + unsafe audit |
| 10 | ci | (built-in) | GitHub Actions workflow gen |

## Workflow in AI Sessions

When the user asks to audit a Rust workspace:

1. Check if `prometheus-auditor.toml` exists; if not, offer to create one from defaults
2. Run `prometheus-rust-auditor audit --format json` and capture output
3. Parse findings and group by severity
4. For `High` findings, propose specific fixes with code snippets
5. For `Info` findings, summarize without blocking
6. After fixes are applied, re-run to confirm clean

## Detailed References

- [Architectural Invariants](references/INVARIANTS.md) — full invariant catalog with ACT/WASM/ASYNC/ALLOC/CORE IDs
- [AI Audit Prompts](references/AI_AUDIT_PROMPT.md) — prompt templates for autonomous phases 6–9
