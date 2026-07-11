# Plan — phase-codex-plugin-implementation

_Planned 2026-07-11 from `assessment.md`. Backend: **native KBD** (openspec/ exists but `opsx` tooling is absent and recent phases used native KBD change files). No evolver bridge._

## Ordering rationale

The assessment flagged two empirical unknowns (plugin non-managed hooks firing; plugin `.mcp.json` `env` support). **change-cpi-001 is a de-risking spike** that resolves both before scope is committed. The manifest (002) is the foundation the component changes (003–006) attach to; build tooling (007) depends on all components; docs + UAR-compat verification (008) closes the phase. All Codex artifacts are **generated output** — single source of truth stays `.claude-plugin/` + `.mcp.json` + `hooks/hooks.json` + `skills/`.

## Change list (8)

| # | Change | Gaps | Pri | Effort | Depends on |
|---|--------|------|-----|--------|-----------|
| 001 | `change-cpi-001-runtime-spike` — empirically verify Codex plugin install, non-managed hook firing, and `.mcp.json` env in codex-cli 0.144.1 | G-01,G-05,G-06 | P0 | S | — |
| 002 | `change-cpi-002-plugin-manifest` — generate `.codex-plugin/plugin.json` from `.claude-plugin/plugin.json` + `interface` block | G-02 | P1 | M | 001 |
| 003 | `change-cpi-003-codex-marketplace` — `.agents/plugins/marketplace.json` (repo) transforming the 11 plugins to Codex `source`/`policy` schema; document personal path; keep `.claude-plugin/marketplace.json` as legacy fallback | G-03 | P1 | M | 002 |
| 004 | `change-cpi-004-plugin-mcp` — emit plugin `.mcp.json` (7 servers) with the env strategy 001 determined; carry forge-token / liter-llm-proxy / tavily-name / keys-from-env fixes | G-05 | P1 | M | 001 |
| 005 | `change-cpi-005-hooks-wiring` — wire `plugin.json.hooks` → `hooks/hooks.json` (same PascalCase schema) + trust docs; scope set by 001's hook-firing result | G-06 | P1 | S–M | 001,002 |
| 006 | `change-cpi-006-skills-bundle` — wire plugin `skills` pointer to the real-dir tree (reuse non-symlink `codex-sync-skills.sh`), curate within catalog budget (`config/codex-catalog.txt`), confirm UAR skill-tree untouched | G-04,G-08 | P1 | M | 002 |
| 007 | `change-cpi-007-build-tooling` — extend build tooling to **generate + validate** the Codex plugin + marketplace idempotently; integrate `install-platforms.ts` codex target | G-07 | P1 | M | 002,003,004,005,006 |
| 008 | `change-cpi-008-parity-docs-uar` — parity checklist vs the Claude plugin, publishing checklist, CLAUDE.md Codex-section update, verify UAR submodule ingestion + `.codex/` regen unaffected | G-08,G-09 | P2 | M | 007 |

## Goal → change coverage

- G-01 → 001 (+ `references/codex-plugin-spec-digest.md` already vendored in assess)
- G-02 → 002 · G-03 → 003 · G-04 → 006 · G-05 → 004 · G-06 → 005 · G-07 → 007 · G-08 → 006,008 · G-09 → 008

## Gate on the spike

**001 gates the scope of 004 and 005.** If plugin non-managed hooks do NOT fire in 0.144 (as the `config.toml [hooks]` path didn't), 005 ships `hooks.json` + documents it as pending-upstream rather than claiming a working hook surface. If plugin `.mcp.json` ignores `env`, 004 routes server keys via `~/.codex/config.toml` env-passthrough + docs instead of inline env. Re-order/re-scope after 001 if its findings warrant.

## Recommended agent

`claude-code` (frontier) for all — transformation + tooling work with two empirical spikes; no parallelizable independent tracks until after 002.

## First change to apply

`change-cpi-001-runtime-spike` — `/kbd-execute` (or `/kbd-apply` per-task).
