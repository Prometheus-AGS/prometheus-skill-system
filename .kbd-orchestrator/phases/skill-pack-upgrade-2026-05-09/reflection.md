# Reflection — skill-pack-upgrade-2026-05-09

**Date:** 2026-05-09  
**Reflector:** Claude Sonnet 4.6 (claude-sonnet-4-6)  
**Session:** f265e820-bad7-483c-9960-836e7a2574d8  
**Phase:** Phase 1 — Quick Wins (Day 0 to Day 2)

---

## Goal Achievement

| Goal ID | Goal | Status | Evidence |
|---------|------|--------|----------|
| BDD-006 | Immutable-tests rule in both CLAUDE.md files | **MET** | Added to `ssr-frontend/CLAUDE.md` (commits `ba52048`) and `prometheus-skill-pack/CLAUDE.md` (commit `38f83e0`). Explicit prose: "may not edit tests/steps/*.steps.ts". |
| SP-015 | Canonical hooks.json path documented + CI guard | **MET** | Assessment revealed direction inversion vs. task doc: `.claude-plugin/hooks` is already a directory symlink → `../hooks`; `hooks/hooks.json` is the physical canonical file. CI guard added to `.github/workflows/validate.yml` (`hooks-integrity` job). CLAUDE.md updated with authoritative path statement. Commit `c586a77`. |
| BDD-001 | Manifest hex-key cleanup + validator rule | **MET** | `docs/videos-manifest.json` normalized from 374 → 345 entries (29 hex orphans archived to `videos-manifest-legacy.json`). `assertNoHexKeysInManifest()` added to `validate-video-coverage.ts`. Migration script at `scripts/migrate-videos-manifest.ts`. Commit `b806e2c`. |
| BDD-002 | `@quarantine` retry + state machine | **MET** | `run-video-proof.ts` now retries quarantined scenarios up to 3×; `quarantine-state.json` tracks consecutive clean/retry runs; promote (5 clean) and escalate (10 retry) thresholds implemented. Report section added. `tests/README.md` documents lifecycle. Commit `e15efa8`. |
| SP-006 | Hook observability JSONL log at `~/.prometheus/hooks.log` | **MET** | `shared/scripts/lib/hook-log.sh` created with `hook_log_start`/`hook_log_end`/`hook_log_error` functions using `flock` serialization. Wired into all 5 Stop-chain hook scripts. `|| true` swallowing replaced with `|| hook_log_error "$LINENO"`. Logrotate config at `shared/config/logrotate.d/prometheus-hooks`. Commit `7cb20dd`. |
| SP-013 | Sycophancy gate on reflector SubagentStop | **MET** | `shared/scripts/sycophancy-check-reflection.sh` created; wired as first command in `reflector` SubagentStop matcher. Reads artifact from hook event (not conversation history). Invokes `detect_sycophancy` via JSON-RPC. 2-rejection soft cap. Configurable via `PROMETHEUS_REFLECT_STRICTNESS`. CLAUDE.md documents gate behavior. Commit `aa2a5b8`. |

**Overall: 6/6 goals MET (100%)**

---

## Delivered Changes

| # | Change | Repo | Commits | Files |
|---|--------|------|---------|-------|
| 1 | change-001-bdd006-immutable-tests-rule | skill-pack + ssr-frontend | `38f83e0`, `ba52048` | 2 CLAUDE.md files |
| 2 | change-002-sp015-hooks-json-canonical | skill-pack | `c586a77` | `.github/workflows/validate.yml`, CLAUDE.md |
| 3 | change-003-bdd001-manifest-dual-key-cleanup | ssr-frontend | `b806e2c` | 4 files (manifest, legacy, migration script, validator) |
| 4 | change-004-bdd002-flake-quarantine | ssr-frontend | `e15efa8` | 3 files (run-video-proof.ts, generate-video-run-report.ts, tests/README.md) |
| 5 | change-005-sp006-stop-hook-observability | skill-pack | `7cb20dd` | 7 files (hook-log.sh, 5 hook scripts, logrotate config) |
| 6 | change-006-sp013-sycophancy-reflector-hook | skill-pack | `aa2a5b8` | 4 files (check-reflection.sh, hooks.json, CLAUDE.md, execution.md) |

All 6 changes completed on 2026-05-09 in a single session. Zero changes rolled back. Zero blocking failures.

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with formal QA (artifact-refiner) | 0 / 6 |
| Changes with self-check QA gate | 4 / 6 |
| Changes QA-skipped (doc/CI-only, <3 files) | 2 / 6 (change-001, change-002) |
| Build/syntax failures | 0 |
| Type-check regressions | 0 |
| JSON validation failures | 0 |

No artifact-refiner was invoked — the refiner infrastructure was not available in this phase. Self-check gates covered:
- `bash -n` syntax validation on all shell scripts
- `python3 -m json.tool` validation on `hooks/hooks.json`
- TypeScript type-check (`npx tsc --noEmit`) on ssr-frontend changes (pre-existing deprecation warning only; no new errors)

**Recommendation for Phase 2**: Initialize `.refiner/` directory and run artifact-refiner on the two hook scripts (change-005, change-006) before the next phase begins.

---

## Surprises and Deviations

### SP-015 direction inversion
The task doc specified making `hooks/hooks.json` a symlink to `.claude-plugin/hooks/hooks.json`. Assessment revealed the opposite: `.claude-plugin/hooks` is already a directory symlink → `../hooks`, making `hooks/hooks.json` the physical canonical. No swap was needed. The change pivoted to documentation + CI guard. **Impact: none on correctness; saves one risky symlink operation.**

### BDD-001 three key forms, not two
The task doc described two key forms (hex vs. slug). Assessment found three: 29 hex, 249 slug-with-`--`, 96 single-part slug. All 29 hex keys are true orphans (zero CID overlap with any slug key across upload-results.json and cucumber reports). Decision: archive hex keys, keep both slug forms. **Impact: implementation scope slightly larger; migration script handles all three forms.**

### ts-node invocation
`npx ts-node --esm` fails on this project's `tsconfig.json`. Correct invocation requires `--project tsconfig.test.json`. Discovered during BDD-001 migration run; corrected without breaking work.

### Assessment assessment: SP-015 verdict was PARTIALLY-ADDRESSED
The assessment correctly flagged the partial state — but the implementation found no corrective action was needed for the symlink itself. The "partially addressed" verdict was accurate: the symlink existed but the CI guard and CLAUDE.md documentation were missing.

---

## Technical Debt Introduced

| Item | Location | Severity | Notes |
|------|----------|----------|-------|
| MCP binary not available in hooks at runtime if not built | `sycophancy-check-reflection.sh` | Low | Gate gracefully degrades (exit 0). User must `cargo build --release` in `skills/imported/sycophancy-correction/` for gate to activate. |
| `quarantine-state.json` runtime artifact | `ssr-frontend/tests/reports/` | Low | Whether to commit is explicitly deferred to team decision (documented in README). |
| `logrotate.d/prometheus-hooks` requires manual install | `shared/config/logrotate.d/` | Low | User-level logrotate with `--state` flag documented as alternative. |
| `FOCUS_OUTPUT` variable scoping in `pk-focus-on-prompt.sh` | `shared/scripts/pk-focus-on-prompt.sh` | Very low | Changed `$FOCUS_OUTPUT` reference to `${FOCUS_OUTPUT:-}` to handle `set -u` correctly after the refactor. |

---

## Lessons

1. **Read plugin.json before assuming symlink direction.** Task docs describing symlink fixes may describe the intended future state, not what exists. A two-minute read of `plugin.json` resolved a potential day of rework.

2. **ts-node has a project-specific invocation pattern.** The `tsconfig.test.json` pattern (`--project tsconfig.test.json`) is not documented in the project README. It is now implicit in execution records but should go in `tests/README.md` in Phase 2.

3. **Shell hook scripts communicating with MCP servers require FIFO pacing.** Sending the MCP initialize + call messages at once causes the initialized notification to arrive before the server is ready. A 200ms sleep between messages is required. Captured in `sycophancy-check-reflection.sh`.

4. **Hex keys are always orphans in this codebase.** Zero CID overlap between hex keys and any other data source. Future cleanup scripts can assume hex → archive with no mapping work.

5. **`|| true` in Stop hooks is a silent failure pattern.** Replacing with `|| hook_log_error "$LINENO"` costs nothing and provides essential debugging surface. This pattern should be applied globally across all new hook scripts from Phase 2 onward.

6. **The binary path for sycophancy-correction has two valid locations.** PATH-installed binary takes priority over `target/release/`. The hook script handles both without user configuration.

---

## Phase 2 Recommended Focus

Based on the execution roadmap in `docs/future-work/04-build-order/execution-roadmap.md`, the Phase 2 candidates (now unblocked by Phase 1) are:

| Task | Description | Unblocked by |
|------|-------------|--------------|
| SP-014 | Fallback SubagentStop matcher verification | SP-006 (log shim now available) |
| BDD-005 | Step definition ownership tagging | BDD-006 (immutable rule now in place) |
| BDD-007 | Test-isolation enforcement gate | BDD-006 |
| SP-007 | Hook chain timeout monitoring | SP-006 |
| XC-004 | `prometheus doctor` health command | SP-006 + SP-013 |

**Highest-leverage first**: SP-014 is fastest (adds a test to the log infrastructure just built). XC-004 has the most user-visible impact but requires more scope.

---

## Phase Verdict

**COMPLETE.** All 6 Phase 1 goals met in a single session. No blocking issues remain. Phase 1 closes with zero unresolved technical debt above Low severity.
