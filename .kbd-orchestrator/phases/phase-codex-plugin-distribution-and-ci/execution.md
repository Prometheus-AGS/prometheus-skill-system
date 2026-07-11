# Execution — phase-codex-plugin-distribution-and-ci

_2026-07-11. Backend: **native-tool (claude-code)**._

## Result: 5/6 DONE, 1 BLOCKED (manual)

| Change | Status | Notes |
|---|---|---|
| 001 constraints | **DONE** | `.kbd-orchestrator/constraints.md` (C-01…C-05) — QA gate now defined |
| 002 ci-validate-codex | **DONE** | `validate:codex` step added to `.github/workflows/validate.yml`; YAML valid |
| 003 install-platforms-build-codex | **DONE** | codex install path runs `npm run build:codex`; `install-platforms.ts --list` parses |
| 004 generator-git-sources | **DONE** | `CODEX_MARKETPLACE_SOURCE=local\|git-subdir\|git`; local byte-stable, git-subdir emits `{url,ref,path}` |
| 005 env-provisioning-helper | **DONE** | `scripts/codex-provision-mcp-env.sh` (bash 3.2, idempotent, no secrets) → `shell_environment_policy.inherit="all"` |
| 006 hook-trust-verification | **BLOCKED** | **Manual/interactive** — requires a live Codex session to trust the plugin and observe a hook write to `${PLUGIN_DATA}`. Cannot be done headlessly. |

QA gate: each change modified <3 files (or docs) → artifact-refiner skipped per the
size rule; `constraints.md` (C-01…C-05) now governs future phases. Final
`validate:codex` green.

## 006 — what the human must do

1. `npm run build:codex` (or install codex) → `codex plugin marketplace add .` →
   `codex plugin add prometheus-skill-pack@prometheus-skill-pack`.
2. Start an interactive `codex` session; review + **trust** the plugin's hooks.
3. Confirm a `SessionStart` hook writes to `${PLUGIN_DATA}` (or record that it does
   not fire).
4. Write `references/hook-trust-verification.md` with the verdict; update
   `docs/codex-plugin.md` + CLAUDE.md; mark 006 DONE, then `/kbd-reflect`.

Until 006 is DONE, `/kbd-reflect` is correctly blocked by `pipeline-enforce` (5/6).
