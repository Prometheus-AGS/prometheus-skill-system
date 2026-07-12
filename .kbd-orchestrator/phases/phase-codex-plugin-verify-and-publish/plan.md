# Plan — phase-codex-plugin-verify-and-publish

_Planned 2026-07-12 from `assessment.md`. Backend: **native KBD**. No evolver bridge._

## Ordering rationale

The two CI-unblock fixes (001 cowork Progress Signals, 002 prettier) go first —
until they land, my `validate:codex` step never runs, so **G-01 is gated on them**.
003 pushes + confirms the run goes green (with `validate:codex` finally executing).
004 (real-hook headless test) and 005 (env round-trip) are independent and
automatable. 006 (git-subdir publish) is **gated on a user go-ahead** (external
publish) and is last.

## Change list (6)

| # | Change | Goal | Pri | Effort | Needs user? |
|---|--------|------|-----|--------|-------------|
| 001 | `change-cpv-001-cowork-progress-signals` — add `## Progress Signals` to `skills/process/cowork-management/SKILL.md` | G-01 | P0 | S | no |
| 002 | `change-cpv-002-format-fix` — resolve the 32 prettier failures (`npm run format`); **first reconcile local-vs-CI prettier** (local reports clean, CI flags 32 → likely a version/config mismatch) | G-01 | P0 | S–M | no |
| 003 | `change-cpv-003-ci-green-verify` — push, watch the `Validate Skills` run go green, confirm `validate:codex` actually executes (not `-`) | G-01 | P1 | S | no |
| 004 | `change-cpv-004-real-hooks-codex` — headless run of the **real** plugin hooks under Codex (`codex exec --dangerously-bypass-hook-trust`); confirm SessionStart executes with no empty-path errors under `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}`; record evidence | G-03 | P1 | S | no |
| 005 | `change-cpv-005-env-roundtrip` — run `codex-provision-mcp-env.sh` with keys sourced, install plugin, confirm `codex doctor` ⚠ clears / a keyed server sees its key | G-02 | P2 | S | keys from env |
| 006 | `change-cpv-006-git-subdir-publish` — generate a `git-subdir` marketplace, publish, `codex plugin marketplace add <git-url>` resolves it | G-04 | P2 | M | **YES (publish decision)** |

## Goal → change coverage

G-01 → 001, 002, 003 · G-03 → 004 · G-02 → 005 · G-04 → 006

## Notes for execute

- **002 prettier discrepancy is the real risk:** local `prettier --check` reports 0
  issues but CI reports 32. Before blindly `npm run format`, diagnose: compare the
  pinned prettier version (CI `npm ci`) vs local, and whether `.prettierignore`
  differs. The fix must make **CI** green, not just local.
- **003 requires a real Actions run** — after 001+002 land, push and read `gh run
  view` for the `Validate Skills` workflow; confirm the "Validate Codex plugin
  artifacts are in sync" step shows ✓ (not `-`).
- **004 is automatable headlessly** — reuse the proven `codex exec
  --dangerously-bypass-hook-trust` path from the prior phase, this time with the
  real (un-probed) hooks. Clean up `~/.codex` after.
- **005** needs `TAVILY_API_KEY`/`FORGE_MCP_TOKEN` exported (they live in
  `~/.bash_profile`, not the automation shell).
- **006 is gated** — do not publish externally without the user's explicit go-ahead;
  if declined, mark it a deliberate skip (`kbd_stage_handoff_skip`) with the reason.

## First change to apply

`change-cpv-001-cowork-progress-signals`.
