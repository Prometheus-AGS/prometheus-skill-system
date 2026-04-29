---
id: change-004-native-agent-wasm-target
title: --target librefang-wasm flag in native-agent + WASM crate emission
phase: phase-compliance-and-power-multiplier
gaps: [G2, E1, E3]
priority: P0
effort: M
agent: code-architect
evolver_item_id: null
status: DONE
completed: 2026-04-29
---

# change-004 — Native-Agent WASM Target

## Context

The `/create-native-agent` command currently produces a Docker-targeted Rust agent
workspace. To make every generated agent portable to a LibreFang/bossfang instance,
the generator needs an opt-in `--target librefang-wasm` flag that emits an extra
`agent-skill` crate alongside `agent-server`, sharing types via `agent-core`.

This change also tackles two related gaps:
- **E1**: include a `crates/agent-tokenizer/` powered by `rustbpe` so the agent has
  a fast, modern tokenizer for context-window estimation.
- **E3**: turn `prometheus-knowledge` (`pk`) on by default in the docker-compose
  the generator emits, so the Karpathy learning loop is the default rather than
  opt-in.

## Scope

In:

- Update `skills/process/native-agent/SKILL.md` to document `--target` flag
  (allowed values: `docker` (default), `librefang-wasm`, `both`).
- Add new template `templates/rust/agent_skill.rs.tera` rendering a thin wrapper
  that re-uses domain types from `agent-core` and calls into a `Handler` trait,
  exposing it via the LibreFang WASM ABI established in change-003.
- Update `templates/rust/workspace.cargo.toml.tera` to conditionally include the
  `agent-skill` crate when `target == "librefang-wasm"` or `"both"`.
- Add new template `templates/rust/agent_tokenizer.rs.tera` — a small crate that
  exposes a `Tokenizer` struct backed by `rustbpe`, used by `agent-server` for
  budget enforcement and by `agent-skill` if WASM target is enabled.
- Add new template `templates/skill_toml.tera` rendering the LibreFang skill
  manifest at project root (only when WASM target is enabled).
- Update `templates/docker/docker-compose.yml.tera`: `prometheus-knowledge`
  service is now `enabled: true` by default (was opt-in).
- Update CLAUDE.md template to document the WASM build path.

Out:

- The `forge package-librefang` command — change-005.
- The upload command — change-005.

## Deliverables

1. Native-agent generator accepts `--target librefang-wasm` (and `--target both`).
2. With `--target librefang-wasm`, generated workspace builds successfully via
   `cargo build --target wasm32-wasip2 --release -p agent-skill`.
3. Default-on `prometheus-knowledge` service in the emitted docker-compose.
4. New `agent-tokenizer` crate using `rustbpe` for context budgeting.

## Acceptance Criteria

- `/create-native-agent --target librefang-wasm test-agent` produces a workspace
  that compiles cleanly to WASM.
- `/create-native-agent --target docker test-agent` matches today's behavior
  (no regressions).
- `/create-native-agent --target both test-agent` produces both a Docker target
  and a WASM target in one workspace.
- The generated `skill.toml` validates against LibreFang's `SkillManifest` schema
  (verified by deserializing it with the librefang-skills crate in CI).

## Files to Touch

- `skills/process/native-agent/SKILL.md`
- `skills/process/native-agent/templates/rust/agent_skill.rs.tera` (new)
- `skills/process/native-agent/templates/rust/agent_tokenizer.rs.tera` (new)
- `skills/process/native-agent/templates/rust/workspace.cargo.toml.tera`
- `skills/process/native-agent/templates/skill_toml.tera` (new)
- `skills/process/native-agent/templates/docker/docker-compose.yml.tera`
- `skills/process/native-agent/skills/create-native-agent/` — the actual
  command-emitter logic that processes the `--target` flag

## Test Plan

- Smoke: each of the 3 target modes produces a workspace that `cargo check`s
  cleanly.
- WASM-specific: the generated `.wasm` from `--target librefang-wasm` loads
  into `WasmSkillSandbox::new()` without panic.
- Integration with change-003: when both this change and change-003 land, a
  fresh native-agent generation has all the artifacts needed for change-005's
  packager to produce a valid `.lf-skill.zip`.
