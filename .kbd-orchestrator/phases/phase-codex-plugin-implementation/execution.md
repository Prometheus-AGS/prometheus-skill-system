# Execution — phase-codex-plugin-implementation

_2026-07-11._

## Backend

**native-tool (claude-code, frontier).** Native KBD change files (openspec/ present but `opsx` tooling absent; recent phases used native KBD). No external tool dispatch (Roo/Cursor), no evolver bridge. KBD remains source of truth: each change tracked in `.kbd-orchestrator/changes/<id>/change.md`, progress in `progress.json`, position in the waypoint.

## Dispatch contract

- Changes applied in dependency order (see `plan.md`). The P0 spike (001) runs first and **gates 004/005 scope**.
- Per-change: implement tasks → mark `[x]` + `status: DONE` in `change.md` → QA gate (artifact-refiner) unless doc-only/<3 files/`--skip-qa` → archive to `.kbd-orchestrator/changes/archive/<date>-<id>/`.
- Empirical/CLI-facing changes verify against **codex-cli 0.144.1** directly.

## Progress

| Change | Status | Notes |
|---|---|---|
| 001 runtime-spike | **DONE** | Both unknowns resolved — see `references/runtime-spike-findings.md`. QA skipped (single findings doc). |
| 002 plugin-manifest | pending | next |
| 003 codex-marketplace | pending | |
| 004 plugin-mcp | pending | spike: inline `env` confirmed working |
| 005 hooks-wiring | pending | spike: wire+document; firing is interactive-trust only |
| 006 skills-bundle | pending | |
| 007 build-tooling | pending | |
| 008 parity-docs-uar | pending | |

## Spike outcome → scope adjustments

- **004:** plugin `.mcp.json` **direct-map with inline `env` is honored** (verified). Proceed as planned; keep secrets sourced from env.
- **005:** plugin hooks bundle + use Claude's PascalCase schema (no translation). Non-managed trust is **interactive** — ship + document, mark end-to-end firing as manual-verify (not CI-verifiable).
- **003:** `source.source="local"`/`source.path="./"` + `[marketplaces.*]`/`[plugins.*]` in `~/.codex/config.toml` confirmed.
