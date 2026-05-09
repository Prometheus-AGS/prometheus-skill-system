# KBD Assessment — Phase 1 Skill-Pack Upgrade
## skill-pack-upgrade-2026-05-09

**Date:** 2026-05-09  
**Session ID:** f265e820-bad7-483c-9960-836e7a2574d8  
**Assessor:** Claude Sonnet 4.6 (claude-sonnet-4-6)  
**Prior input:** `docs/future-work/` — architectural review output generated 2026-05-09  
**Phase assessed:** Phase 1 — Quick wins (Day 0 to Day 2) per `04-build-order/execution-roadmap.md`

---

## Summary Verdict Table

| Task ID | Title | Verdict | One-line finding |
|---------|-------|---------|-----------------|
| SP-013 | Sycophancy correction in SubagentStop(reflector) hook | **CONFIRMED-GAP** | `forge-reflect-on-stop.sh` and the reflector `SubagentStop` hook contain zero invocations of `sycophancy-correction`. |
| SP-015 | hooks.json symlink fix | **PARTIALLY-ADDRESSED** | `.claude-plugin/hooks` is a symlink to `../hooks`; however `hooks/hooks.json` is still a **regular file**, not a symlink. The fix proposed in the task doc (making `hooks/hooks.json` a symlink to the `.claude-plugin/hooks/hooks.json`) is still needed. |
| SP-006 | Stop hook observability log | **CONFIRMED-GAP** | `~/.prometheus/hooks.log` does not exist. No hook script in `shared/scripts/` writes to any log file. `|| true` error-swallowing is pervasive. |
| BDD-001 | Manifest dual-key cleanup | **CONFIRMED-GAP** | `docs/videos-manifest.json` in `ssr-frontend` contains **29 hex-form keys** alongside 345 slug-form keys (374 total). Hex-form keys are exactly the legacy dual-keying described in the task doc. Path is accessible. |
| BDD-002 | Flake quarantine / retry mechanism | **CONFIRMED-GAP** | `scripts/run-video-proof.ts` (412 lines) contains zero occurrences of `quarantine`, `retry`, `retries`, or `failFast`. No quarantine tag handling or state file exists. |
| BDD-006 | Immutable-tests CLAUDE.md rule | **CONFIRMED-GAP** | Neither `ssr-frontend/CLAUDE.md` nor `prometheus-skill-pack/CLAUDE.md` contains the immutable-tests rule language ("may not edit", "independent specification", "code-gen agents"). `ssr-frontend/CLAUDE.md` mentions `tests/steps/*.steps.ts` in a directory-listing context only (line 207), with no behavioral restriction. |

---

## Evidence per Task

### SP-013 — Sycophancy correction in SubagentStop(reflector) hook

**Gap confirmed.**

File inspected: `shared/scripts/forge-reflect-on-stop.sh`

```bash
#!/usr/bin/env bash
# forge-reflect-on-stop.sh — Stop hook: runs forge reflect if forge and an iterations dir exist.
forge reflect 2>&1 || true
if command -v pk &>/dev/null; then
  pk ingest 2>&1 || true
fi
exit 0
```

Neither `sycophancy-correction` nor any equivalent MCP tool is invoked. The script calls `forge reflect` and optionally `pk ingest`. Nothing in the Stop chain checks the reflection artifact for sycophancy patterns.

`hooks/hooks.json` SubagentStop section (lines 110–130): the `reflector` matcher invokes three scripts:
- `log-reflection.sh`
- `state-checkpoint.sh`
- `workflow-dispatch.sh`

None of these call `sycophancy-correction`. The task doc's description of the gap is exact.

---

### SP-015 — hooks.json symlink fix

**Partially addressed — but not in the direction the task doc specifies.**

Observed state:
- `.claude-plugin/hooks` → symlink → `../hooks` (the entire `hooks/` directory is symlinked)
- `hooks/hooks.json` → **regular file** (confirmed via `readlink`, returned empty)

The task doc proposes: make `hooks/hooks.json` a symlink to `../.claude-plugin/hooks/hooks.json`.

The actual state is the inverse: `.claude-plugin/hooks` is a symlink that resolves through to the regular file at `hooks/hooks.json`. This means:
- Both paths resolve to the same physical file — no drift risk currently.
- The symlink chain is directory-level, not file-level.
- The task doc's proposed fix (file-level symlink in `hooks/`) is still logically useful as an explicit canonical declaration, but the practical drift risk it was designed to prevent is already handled by the directory-level symlink.

**Classification: PARTIALLY-ADDRESSED** — drift risk is mitigated by the directory symlink, but the task acceptance criteria (`hooks/hooks.json` is a symlink) is not met. The task executor should re-evaluate whether the fix is still needed given the directory-level symlink, and update the task doc if the approach changes.

---

### SP-006 — Stop hook observability log

**Gap confirmed.**

Checks performed:
1. `grep -r "hooks.log" shared/scripts/` → zero matches
2. `grep -r "hooks.log" hooks/` → zero matches  
3. `grep -r "hooks.log" . --include="*.sh"` → zero matches across the entire repo
4. `ls ~/.prometheus/` → does not exist (home directory has no `.prometheus/`)
5. Local `.prometheus/` at `prometheus-skill-pack/.prometheus/` contains only `traces/` and `wiki/` subdirectories; no `hooks.log`

The shared hook library (`shared/scripts/lib/hook-log.sh`) described in the task doc does not exist. All hook scripts use `|| true` with no failure capture.

---

### BDD-001 — Manifest dual-key cleanup

**Gap confirmed. Path accessible.**

File: `/Users/gqadonis/Projects/sansaba/ssr-frontend/docs/videos-manifest.json`

Key distribution across the `videos` object (374 total entries):
- **Hex-form keys (32-char lowercase hex):** 29  
  Examples: `025e00ae15af902f51290a864ddbc670`, `10b156dddd023d4e5cb375841d87f239`
- **Slug-form with `--` separator:** 249  
  Example: `acquisition-buyers-invoicing--add-a-buyer-to-an-acquisition`
- **Slug-form without `--` separator:** 96  
  Example: `acquisition-edit-page-loads-with-all-tabs`

The 29 hex-form keys are the dual-keying described in BDD-001. Whether they overlap with slug equivalents (same scenario represented twice) was not checked at this depth — that is the migration script's job (BDD-001 implementation step 1). The hex keys are confirmed present.

The file is accessible from this session.

---

### BDD-002 — Flake quarantine / retry mechanism

**Gap confirmed.**

File: `/Users/gqadonis/Projects/sansaba/ssr-frontend/scripts/run-video-proof.ts` (412 lines)

Search results:
- `quarantine` → 0 occurrences
- `retry` / `retries` → 0 occurrences  
- `failFast` / `fail_fast` → 0 occurrences

No `tests/reports/quarantine-state.json` file was found. The `@quarantine` tag is not defined anywhere in the test runner. The task doc's characterization of the gap is exact.

---

### BDD-006 — Immutable-tests CLAUDE.md rule

**Gap confirmed.**

Files checked:
1. `/Users/gqadonis/Projects/sansaba/ssr-frontend/CLAUDE.md`
   - Line 207 mentions `tests/steps/*.steps.ts` in a directory-map context ("Steps: `tests/steps/*.steps.ts`")
   - Search for: `immutable`, `may not edit`, `independent spec`, `code-gen`, `auto-update` → 0 matches
2. `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/CLAUDE.md`
   - Search for same terms → 0 matches

Neither file contains the behavioral restriction text described in BDD-006. The ssr-frontend `CLAUDE.md` describes BDD conventions (tags, selectors, feature-per-file) but does not address agent edit permissions.

---

## Blockers and Surprises

### Surprise 1 — SP-015 direction inversion

The task doc proposes `hooks/hooks.json → symlink → ../.claude-plugin/hooks/hooks.json`, treating `.claude-plugin/hooks/hooks.json` as canonical. The actual on-disk state is the reverse: `.claude-plugin/hooks` is a **directory symlink** pointing to `../hooks`, making `hooks/hooks.json` the physical file and `.claude-plugin/hooks/hooks.json` the derived path.

**Impact on execution order:** The task remains worth confirming/documenting, but the executor should re-read the `plugin.json` to verify which location the Claude Code runtime actually reads from — that determination resolves which path should be canonical. This is low-risk to execution order but avoids a wasted half-day reverting a correct symlink.

### Surprise 2 — docs/future-work/ was not present at session start

The `docs/future-work/` directory was absent when the session started; the user confirmed it was added and the assessment was retried. No impact on findings. Noted for session continuity documentation only.

### Surprise 3 — BDD-001 manifest has three key forms, not two

The task doc describes hex vs. slug dual-keying as a binary situation. The actual manifest has a third form: single-part slugs without the `--` separator (`acquisition-edit-page-loads-with-all-tabs`). These 96 entries appear to be older scenarios before the `feature--scenario` slug convention solidified. The migration script (BDD-001 implementation step 2) must handle this third form. **This is not a blocker but the migration script author must be aware.**

### No blockers to Phase 1 execution order

All six tasks remain executable in the order the roadmap specifies. No circular dependencies, missing files, or inaccessible paths were found that would block starting work.

---

## Recommended Phase 1 Execution Order

The roadmap order is sound. No adjustments required based on findings. Suggested slot allocation:

| Slot | Task | Rationale |
|------|------|-----------|
| 1 (parallel) | BDD-006 | Fastest (0.5d). Must land before any BDD agent picks up BDD-005/007 in Phase 2. Pure doc edit. |
| 1 (parallel) | SP-015 | 0.5d. Land before any hook-modification task runs. Executor should read `plugin.json` first to confirm canonical path (see Surprise 1). |
| 2 (parallel) | BDD-001 | 0.5d. No dependencies. Executor must handle the three-key-form complexity (see Surprise 3). |
| 2 (parallel) | BDD-002 | 1d. No dependencies. Executor works in `ssr-frontend/scripts/`. |
| 3 | SP-013 | 1-2d. Can start in parallel with BDD-001/002 but benefits from SP-006 being in-flight so the logging shim is available to wire immediately after. Roadmap recommends landing SP-006 first; that is the correct call. |
| 4 | SP-006 | 1d. Start as soon as SP-015 commits. SP-013 implementation step 4 (rejection logging) depends on SP-006's shim existing. |

If only one agent is available: SP-015 → SP-006 → SP-013 → BDD-006 → BDD-001 → BDD-002.

---

## Scope — What This Assessment Does NOT Cover

This assessment is bounded to Phase 1 tasks only. The following are explicitly out of scope:

- **Phase 2–6 tasks** (BDD-005, SP-008, SP-001, BDD-007, SP-016, and all later phases). Their state was not inspected.
- **`prometheus-knowledge` crate** — not inspected for any of the assessed tasks.
- **`document-generation-agent`** — not inspected.
- **Cross-cutting tasks** (XC-001 through XC-005) — out of Phase 1 scope.
- **All SP-002 through SP-012, SP-014, SP-016 through SP-021** — not assessed; their dependencies may or may not be met.
- **Surreal-memory hydration** (`00-meta/memory-bootstrap.md`) — not executed; STATUS.md remains the live source of truth.
- **Implementation quality of existing hook scripts** — hooks were read for presence/absence of specific patterns only, not for correctness of existing behavior.

---

## Next Step

Present this artifact to the human for review. Upon approval, mark this assessment `in-progress` → `done` is not applicable (assessment is not a task in STATUS.md). The human should select a task from the Phase 1 execution order above and spawn an appropriate agent with the relevant task doc.
