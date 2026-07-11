# Plan — phase-codex-plugin-distribution-and-ci

_Planned 2026-07-11 from `assessment.md` (seeded by the prior phase's reflection deltas). Backend: **native KBD**. No evolver bridge._

## Ordering rationale

`constraints.md` (001) goes first so the artifact-refiner QA gate covers every
subsequent change. Then the two smallest CI/tooling wins (002 CI, 003 install
integration), then the generator source-type extension (004) and env helper
(005). The **manual** hook-trust verification (006) is last and is explicitly a
doc/evidence change — not automatable. Changes are largely independent (all touch
different files); the order is by value + the QA-enablement dependency.

## Change list (6)

| # | Change | Goal | Pri | Effort | Manual? |
|---|--------|------|-----|--------|---------|
| 001 | `change-cpd-001-constraints` — author `.kbd-orchestrator/constraints.md` (generated-artifacts-in-sync, no-committed-secrets, docs-updated) | G-06 | P1 | S | no |
| 002 | `change-cpd-002-ci-validate-codex` — add `npm run validate:codex` step to `.github/workflows/validate.yml` | G-02 | P1 | S | no |
| 003 | `change-cpd-003-install-platforms-build-codex` — run `build:codex` in the `install-platforms.ts` codex target so install regenerates the plugin artifacts | G-01 | P1 | S–M | no |
| 004 | `change-cpd-004-generator-git-sources` — make marketplace `source` type configurable (`local` default, `git-subdir`/`git` for publish) in `build-codex-plugin.js` | G-05 | P2 | S–M | no |
| 005 | `change-cpd-005-env-provisioning-helper` — helper that seeds `~/.codex/config.toml` env for the 7 servers' keys from the environment (no committed secrets) | G-04 | P1 | M | no |
| 006 | `change-cpd-006-hook-trust-verification` — interactively trust the plugin, confirm a hook fires, record evidence + update docs | G-03 | P2 | S | **YES (interactive)** |

## Goal → change coverage

G-06→001 · G-02→002 · G-01→003 · G-05→004 · G-04→005 · G-03→006

## Notes for execute

- **006 is a manual interactive step** — `kbd-execute` cannot fully automate it; it produces `references/hook-trust-verification.md`. If an interactive Codex session isn't available in the run, mark 006 BLOCKED with a clear "needs human interactive trust" note rather than faking a pass.
- Reuse existing tooling: 003 extends `install-platforms.ts` (mind bash-3.2 for any launchd path); 005 reuses the tavily/firecrawl env-seeding pattern + `configure-mcp-all-tools.sh`.
- 004 `git-subdir` only becomes meaningful once the plugin is published externally — scope as "support + document," keep `local` the default.
- After 001 lands, subsequent changes are QA-gated by artifact-refiner **if** the binary is present; otherwise QA logs the skip (as prior phases).

## First change to apply

`change-cpd-001-constraints`.
