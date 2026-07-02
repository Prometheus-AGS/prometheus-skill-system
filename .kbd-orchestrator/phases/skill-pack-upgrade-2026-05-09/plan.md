# Phase Plan — skill-pack-upgrade-2026-05-09
## Phase 1: Quick Wins (Day 0 to Day 2)

**Generated:** 2026-05-09  
**Change backend:** OpenSpec (`openspec/changes/`)  
**Assessment source:** `.kbd-orchestrator/phases/assess/skill-pack-upgrade-2026-05-09-assessment.md`  
**Roadmap source:** `docs/future-work/04-build-order/execution-roadmap.md`

---

## Ordered Change List

Changes are ordered by the slot allocation from the assessment's Phase 1 execution order. Slots 1 and 2 are parallel; slots 3 and 4 are sequential.

| # | Change ID | Source Task | Priority | Effort | Agent Role | Slot | Repo |
|---|-----------|-------------|----------|--------|------------|------|------|
| 1 | `change-001-bdd006-immutable-tests-rule` | BDD-006 | P0 | 0.5d | docs-writer | 1 (parallel) | skill-pack + ssr-frontend |
| 2 | `change-002-sp015-hooks-json-canonical` | SP-015 | P2 | 0.5d | skill-pack-maintainer | 1 (parallel) | skill-pack |
| 3 | `change-003-bdd001-manifest-dual-key-cleanup` | BDD-001 | P0 | 0.5d | bdd-engineer | 2 (parallel) | ssr-frontend |
| 4 | `change-004-bdd002-flake-quarantine` | BDD-002 | P0 | 1d | bdd-engineer | 2 (parallel) | ssr-frontend |
| 5 | `change-005-sp006-stop-hook-observability` | SP-006 | P0 | 1d | hooks-engineer | 3 | skill-pack |
| 6 | `change-006-sp013-sycophancy-reflector-hook` | SP-013 | P0 | 1-2d | hooks-engineer | 4 | skill-pack |

---

## Change Summaries

### change-001-bdd006-immutable-tests-rule (Slot 1)
**Agent:** docs-writer  
**Source task:** `docs/future-work/02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md`  
**Proposal:** `.kbd-orchestrator/changes/change-001-bdd006-immutable-tests-rule/change.md` (migrated from OpenSpec 2026-07-02)

Add the immutable-tests behavioral rule to both CLAUDE.md files. Code-gen agents may not edit `tests/steps/*.steps.ts`, `tests/support/*.ts`, or `tests/features/*.feature` to make existing tests pass. Optional: add `shared/scripts/protect-tests.sh` PreToolUse guard (warn mode).

**Files to touch:**
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/CLAUDE.md` — add rule to BDD section
- `CLAUDE.md` (skill-pack root) — add abbreviated rule with cross-reference
- `shared/scripts/protect-tests.sh` (new, optional)

**Why first:** Fastest task (0.5d). Must land before any BDD agent picks up BDD-005/007 in Phase 2. Pure doc edit — zero blast radius.

---

### change-002-sp015-hooks-json-canonical (Slot 1)
**Agent:** skill-pack-maintainer  
**Source task:** `docs/future-work/01-skill-pack-fixes/SP-015-hooks-json-symlink.md`  
**Proposal:** `.kbd-orchestrator/changes/change-002-sp015-hooks-json-canonical/change.md` (migrated from OpenSpec 2026-07-02)

**Assessment surprise:** The task doc direction is inverted. `.claude-plugin/hooks` is already a directory symlink → `../hooks`. Executor must read `plugin.json` first to confirm which path is authoritative for the Claude Code runtime, then either document the existing setup or swap canonicity as needed. Add CI check regardless.

**Files to touch:**
- `.claude-plugin/plugin.json` — read to determine canonical path
- `.github/workflows/validate.yml` — add one-line CI symlink check
- `hooks/hooks.json` or `.claude-plugin/hooks` — one becomes explicit symlink (direction TBD)

**Why first:** Land before any hook-modification change touches `hooks.json`. Once CI guard is in place, subsequent hook changes can't silently break the symlink.

---

### change-003-bdd001-manifest-dual-key-cleanup (Slot 2)
**Agent:** bdd-engineer  
**Source task:** `docs/future-work/02-bdd-testing-evolution/BDD-001-manifest-dual-key-cleanup.md`  
**Proposal:** `openspec/changes/change-003-bdd001-manifest-dual-key-cleanup/proposal.md`

**Assessment discovery:** 374 total manifest entries across THREE key forms (not two as the task doc states): 29 hex-form, 249 slug-with-`--`, 96 single-part-slug. The migration script must handle all three. Executor must run dry-run mode and review before applying.

**Files to touch:**
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/docs/videos-manifest.json` — normalized
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/docs/videos-manifest-legacy.json` — new, archives unmappable hex entries
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/scripts/migrate-videos-manifest.ts` — new migration script
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/scripts/validate-video-coverage.ts` — add hex-key rejection rule

---

### change-004-bdd002-flake-quarantine (Slot 2)
**Agent:** bdd-engineer  
**Source task:** `docs/future-work/02-bdd-testing-evolution/BDD-002-flake-quarantine.md`  
**Proposal:** `openspec/changes/change-004-bdd002-flake-quarantine/proposal.md`

Add `@quarantine` tag + retry policy (up to 3 retries) + state machine (`quarantine-state.json`) + report section. Non-quarantined scenarios keep current fail-fast behavior. Can run in parallel with change-003 — different file scope.

**Files to touch:**
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/scripts/run-video-proof.ts`
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/scripts/generate-video-run-report.ts`
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/tests/reports/quarantine-state.json` (new)
- `/Users/gqadonis/Projects/sansaba/ssr-frontend/tests/README.md`

---

### change-005-sp006-stop-hook-observability (Slot 3)
**Agent:** hooks-engineer  
**Source task:** `docs/future-work/01-skill-pack-fixes/SP-006-stop-hook-observability.md`  
**Proposal:** `.kbd-orchestrator/changes/change-005-sp006-stop-hook-observability/change.md` (migrated from OpenSpec 2026-07-02)

Create `shared/scripts/lib/hook-log.sh` with `hook_log_start`, `hook_log_end`, `hook_log_error` functions using `flock`-serialized JSONL writes to `~/.prometheus/hooks.log`. Wire all hook scripts. Add logrotate config.

**Must land before change-006** — the sycophancy gate (SP-013) logs decisions via this shim.

**Files to touch:**
- `shared/scripts/lib/hook-log.sh` (new)
- `shared/scripts/forge-reflect-on-stop.sh`
- `shared/scripts/pk-focus-on-prompt.sh`
- `shared/scripts/guard-direct-deploy.sh`
- `shared/scripts/validate-gitops-write.sh`
- `shared/scripts/subagent-checkpoint-fallback.sh`
- `shared/config/logrotate.d/prometheus-hooks` (new)

---

### change-006-sp013-sycophancy-reflector-hook (Slot 4)
**Agent:** hooks-engineer  
**Source task:** `docs/future-work/01-skill-pack-fixes/SP-013-sycophancy-reflector-hook.md`  
**Proposal:** `.kbd-orchestrator/changes/change-006-sp013-sycophancy-reflector-hook/change.md` (migrated from OpenSpec 2026-07-02)

Write `shared/scripts/sycophancy-check-reflection.sh` that reads the reflection artifact only (never the generation history), invokes `sycophancy-correction` at configurable strictness, and rejects sycophantic reflections with actionable feedback. Wire into the `reflector` SubagentStop matcher. 2-rejection soft cap prevents rejection loops.

**Depends on change-005** (hook-log shim must exist for SP-006 logging requirement).

**Files to touch:**
- `shared/scripts/sycophancy-check-reflection.sh` (new)
- `hooks/hooks.json` — add command to `reflector` SubagentStop matcher
- `.prometheus/reflect-rejections.txt` (runtime, not committed)
- `CLAUDE.md` (skill-pack root) — document the gate and `PROMETHEUS_REFLECT_STRICTNESS`

---

## Parallelism Map

```
Day 0 ─── Slot 1 (parallel) ──────────────────────────────────────────────
           change-001-bdd006 (docs-writer)
           change-002-sp015  (skill-pack-maintainer)

Day 1 ─── Slot 2 (parallel, after Slot 1 commits) ───────────────────────
           change-003-bdd001 (bdd-engineer, ssr-frontend)
           change-004-bdd002 (bdd-engineer, ssr-frontend)

Day 1 ─── Slot 3 (can start concurrent with Slot 2) ─────────────────────
           change-005-sp006  (hooks-engineer, skill-pack)

Day 2 ─── Slot 4 (after Slot 3 commits) ─────────────────────────────────
           change-006-sp013  (hooks-engineer, skill-pack)
```

Single-agent linear order: `change-001` → `change-002` → `change-005` → `change-006` → `change-003` → `change-004`

---

## Acceptance Gate for Phase Completion

Phase 1 is complete when all six changes are `archived` in OpenSpec and:

1. `ssr-frontend/CLAUDE.md` contains the immutable-tests rule text.
2. `skill-pack CLAUDE.md` contains abbreviated rule + cross-reference.
3. CI check on `hooks.json` symlink passes in `.github/workflows/validate.yml`.
4. `docs/videos-manifest.json` has zero hex-form keys (`openspec validate` or manual check).
5. `run-video-proof.ts` recognizes `@quarantine` tag; `quarantine-state.json` is created on first quarantined run.
6. `~/.prometheus/hooks.log` contains JSONL events after running a Claude Code session.
7. A synthetic sycophantic reflection artifact is rejected by the sycophancy gate at `strict` strictness; a balanced reflection passes.

---

## What This Plan Does NOT Cover

- Phase 2–6 tasks from the execution roadmap.
- Surreal-memory hydration from STATUS.md.
- Any task not in the Phase 1 slot table above.
