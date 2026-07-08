---
id: change-prb-001-scaffold-crate
title: Scaffold prometheus-research Rust crate with Cargo.toml and main.rs
phase: phase-prometheus-research-binary
priority: P0
effort: M
wave: 1
agent: general-purpose
status: pending
gap_id: G-01
verdict: BUILD
depends_on: null
scope:
  - substrate/prometheus-research/Cargo.toml
  - substrate/prometheus-research/src/main.rs
  - substrate/prometheus-research/src/lib.rs
  - substrate/prometheus-research/src/config.rs
---

# Change: Scaffold prometheus-research crate

## Problem

`substrate/prometheus-research/` does not exist. The `deep-research` skill has no background
execution backend — long-running research jobs time out when the session context is exhausted.

## Solution

Scaffold a new Rust binary+lib crate following the sovereign-sync pattern exactly:
- `Cargo.toml` with all required dependencies (clap 4, axum 0.8, rmcp 1.8, tokio 1, nix 0.28,
  pulldown-cmark 0.11, serde 1, anyhow 1, thiserror 1, dirs-next 2, tracing 0.1)
- `src/main.rs` with clap `Cli` struct and `Mode` enum (`Mcp`, `Server`, `Daemon`, `Status`)
- `src/lib.rs` with pub module declarations for all planned modules
- `src/config.rs` with `ResearchConfig` struct (port, job_dir, surface_bridge_url)
- `cargo build --release` must succeed (empty stub implementations are acceptable)

## Acceptance Criteria

- [ ] `substrate/prometheus-research/Cargo.toml` exists with correct `[package]`, `[[bin]]`, `[lib]`
- [ ] `cargo build --release` in `substrate/prometheus-research/` completes with 0 errors
- [ ] `prometheus-research --help` outputs usage text
- [ ] `prometheus-research --mode server --port 7891` exits cleanly (stub OK)
