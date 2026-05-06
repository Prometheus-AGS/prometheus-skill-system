# prometheus-rust-auditor Design

**Date**: 2026-05-06  
**Author**: Travis James  
**Status**: Approved

## Context

Prometheus AGS projects are large Rust workspaces (UAR ecosystem, actor systems, WASM layers, MCP
servers, SurrealDB persistence). As AI-assisted code generation scales up, there is a compounding
risk of ownership regressions, clone proliferation, Arc abuse, unsafe boundary violations, and
async cancellation bugs being introduced faster than manual review can catch them.

The goal is a staged autonomous remediation pipeline that:
1. Runs deterministic enforcement first (Clippy, fmt, cargo-deny, geiger)
2. Generates a structured architectural inventory
3. Partitions the workspace into domain-aligned chunks
4. Applies per-partition AI audits guided by Prometheus-specific invariants
5. Verifies fixes with benchmarks and concurrency tools (criterion, loom)
6. Generates permanent CI enforcement

The system is both a CLI binary (installable via `cargo install`) and a Claude Code skill
(`/rust-auditor`) that orchestrates the AI audit phases.

---

## Architecture

### Repository Layout

```
prometheus-skill-pack/
├── tools/
│   └── prometheus-rust-auditor/           ← Rust binary crate
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                    ← clap CLI entrypoint
│           ├── config.rs                  ← prometheus-auditor.toml loader
│           ├── scanner.rs                 ← workspace discovery + crate graph
│           ├── reporter.rs                ← structured JSON/SARIF/text output
│           └── phases/
│               ├── mod.rs
│               ├── enforce.rs             ← Phase 1: strict Clippy lint enforcement
│               ├── format.rs              ← Phase 2: cargo fmt workspace
│               ├── deps.rs                ← Phase 3: cargo-deny + cargo-audit
│               ├── inventory.rs           ← Phase 4-5: geiger + crate graph JSON
│               ├── partition.rs           ← Phase 5: domain partition map
│               ├── ci.rs                  ← Phase 10: GitHub Actions YAML generator
│               └── autonomous.rs          ← Phase 6-9: claude --headless (STUBBED)
│
├── skills/rust/prometheus-rust-auditor/
│   ├── SKILL.md                           ← /rust-auditor slash command
│   ├── scripts/
│   │   └── install-tools.sh               ← installs cargo-geiger, cargo-deny, etc.
│   └── references/
│       ├── INVARIANTS.md                  ← UAR architectural invariants reference
│       └── AI_AUDIT_PROMPT.md             ← canonical per-domain AI audit prompt template
│
└── agents/
    └── rust-auditor.md                    ← orchestrator agent brain
```

---

## Binary CLI Surface

```
prometheus-rust-auditor [COMMAND] [OPTIONS] [WORKSPACE_PATH]

Commands:
  audit       Run the full pipeline (all enabled phases)
  enforce     Phase 1-2 only: strict Clippy + fmt
  deps        Phase 3: cargo-deny + cargo-audit
  inventory   Phases 4-5: geiger scan + crate graph + domain partition JSON
  ci          Phase 10: generate .github/workflows/rust-quality.yml
  config      Emit a default prometheus-auditor.toml to stdout

Options:
  --phases <1,3,5>       Run specific phases only
  --partition <name>     Scope to one named partition (actor, mcp, wasm, etc.)
  --output <fmt>         json | sarif | text  (default: text)
  --fix                  Apply auto-fixable issues (clippy --fix, cargo fmt)
  --autonomous           Phase 6-9: AI loop via claude --headless (STUBBED)
  --config <path>        Path to prometheus-auditor.toml (default: workspace root)
  -v, --verbose          Show per-phase subprocess output
```

**Exit codes**: `0` = clean, `1` = findings present, `2` = tool/config error.  
All phases emit structured JSON to stdout when `--output json` is set.

---

## `prometheus-auditor.toml` Schema

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
actor_no_shared_mutable_state = true   # deny Arc<Mutex<T>> in actor crates
wasm_unsafe_confined = true            # unsafe only in *-wasm crates
async_cancellation_safe = true         # deny lock guards held across .await
zero_copy_preference = true            # warn on .clone() in hot paths
no_platform_coupling = true            # deny cfg(target_os) in *-core crates

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

---

## Phase Map

| Phase | Name | Status | Tool(s) |
|-------|------|--------|---------|
| 1 | Strict Clippy enforcement | **Full** | `cargo clippy --workspace --all-features --fix` |
| 2 | Format workspace | **Full** | `cargo fmt --all` |
| 3 | Dependency & security audit | **Full** | `cargo deny check`, `cargo audit` |
| 4 | Unsafe usage map | **Full** | `cargo geiger` |
| 5 | Architectural inventory + partition | **Full** | `cargo metadata`, crate-glob matching |
| 6 | Per-partition AI audit loop | **STUB** | `claude --headless` (Anthropic API) |
| 7 | Benchmark gates | **STUB** | `criterion`, flamegraphs |
| 8 | Concurrency verification | **STUB** | `loom`, `tokio-console` |
| 9 | PR generation | **STUB** | `gh pr create` |
| 10 | CI enforcement generation | **Full** | template-based YAML emit |

---

## Skill: `/rust-auditor`

The skill acts as the AI-side orchestrator when invoked from Claude Code:

1. **Prerequisite check** — verify `prometheus-rust-auditor` is on PATH; emit install instructions if not
2. **Inventory** — run `prometheus-rust-auditor inventory --output json` and parse the partition map
3. **Load invariants** — read `prometheus-auditor.toml` to scope audit prompts per partition
4. **Deterministic phases** — run phases 1-5 via the binary with `--fix` flag
5. **AI audit loop** — for each partition, apply the canonical prompt from `AI_AUDIT_PROMPT.md` enriched with partition-specific invariants; generate diffs; apply; re-run Clippy to verify
6. **CI generation** — run `prometheus-rust-auditor ci` to write/update the GitHub Actions workflow
7. **Summary** — emit a structured findings report with severity tiers

---

## Agent: `rust-auditor.md`

The agent orchestrates multi-session audits. It:
- Tracks which partitions have been audited and which findings remain open
- Delegates deterministic work to the binary via shell commands
- Delegates AI audit work to the skill's prompt templates
- Maintains a session checkpoint so long audits can resume across context resets
- Enforces the architectural invariants defined in `INVARIANTS.md` as hard constraints on every proposed fix

---

## Installation

```bash
# 1. Install the binary from this repo
cargo install --path tools/prometheus-rust-auditor

# 2. Install tooling dependencies
bash .claude/skills/prometheus-rust-auditor/scripts/install-tools.sh

# 3. Scaffold config in target workspace
prometheus-rust-auditor config > prometheus-auditor.toml
# Edit partitions to match your crate names

# 4. Run via Claude Code skill
/rust-auditor

# 5. Or run the binary directly
prometheus-rust-auditor audit --fix
prometheus-rust-auditor audit --partition actor --output json
prometheus-rust-auditor ci
```

---

## Verification Plan

1. `cargo build` in `tools/prometheus-rust-auditor/` — binary compiles clean
2. `cargo install --path tools/prometheus-rust-auditor` — installs to `~/.cargo/bin/`
3. `prometheus-rust-auditor --help` — CLI surface renders correctly
4. `prometheus-rust-auditor config` — emits valid TOML to stdout
5. Run `prometheus-rust-auditor enforce` against a known-dirty Rust workspace — exits `1` with findings
6. Run with `--fix` — Clippy auto-fixes applied, `cargo fmt` runs, workspace is cleaner
7. `prometheus-rust-auditor inventory --output json` — valid JSON with crate list and partition map
8. `prometheus-rust-auditor ci` — emits a valid `.github/workflows/rust-quality.yml`
9. Validate skill: `npm run validate:strict skills/rust/prometheus-rust-auditor`
10. `/rust-auditor` in Claude Code — skill triggers, reads binary output, applies AI audit to first partition
