# Goals — phase-prometheus-research-binary

## Context

The `deep-research` skill (shipped in `phase-deep-research-skill`, commit `5397353`) implements a
10-stage research pipeline as SKILL.md instructions. The pipeline runs entirely within the agent's
context window, which means:

- Long-running research (exhaustive depth, 20+ sources) risks hitting context limits
- There is no background execution — the user must keep the session active
- No real-time progress streaming to the HTML UI prototype
- Job state is not persisted across sessions

This phase scaffolds `prometheus-research` — a Rust CLI + MCP server binary — to fix these constraints.

## Goals

- [ ] **G-01: Scaffold `prometheus-research` crate** via `native-agent` skill — Rust CLI binary with `clap` and a working `cargo build --release`
- [ ] **G-02: Implement `start` subcommand** — accepts `query`, `depth`, `max-sources`, `citation-style`; spawns a background research job; writes job state to `~/.research-jobs/<job-id>/`
- [ ] **G-03: Implement `status` subcommand** — reads job checkpoint from disk; prints stage, progress, elapsed time
- [ ] **G-04: Implement `cancel` subcommand** — sends SIGTERM to running job process; marks job as cancelled in checkpoint
- [ ] **G-05: Implement MCP server mode (`--mode mcp`)** — exposes `research_start`, `research_status`, `research_cancel`, `research_export` as MCP tools over stdio transport
- [ ] **G-06: Wire SSE streaming to surface-bridge** — POST stage-completion events to `http://127.0.0.1:7890/mcp/render-ui-intent` so the HTML UI receives real-time progress
- [ ] **G-07: Write `prometheus-research` launchd service plist** — auto-start on login; expose MCP on stdio; register with `install-skills-flat.sh`
- [ ] **G-08: Commit, tag `v1.6.0`, and push** — `prometheus-research` binary installable via `cargo install` from workspace
