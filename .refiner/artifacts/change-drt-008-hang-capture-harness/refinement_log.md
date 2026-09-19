# Refinement Log — change-drt-008-hang-capture-harness

Gate: /refine-validate (KBD-wired; constraints sourced from .kbd-orchestrator/constraints.md per references/integrations/artifact-refiner.md — never defined independently)
Date: 2026-09-13 · Validator: glm-5.3 (opencode session)

## Constraint evaluation

| Constraint | Verdict | Evidence |
|---|---|---|
| C-01 generated artifacts in sync | **PASS (deferral contract honored)** | Change adds files under `skills/research/deep-research/tests/hang/` (script + HANG-CAPTURE.md; capture artifacts not committed). Reconciliation owner named in persisted evidence: spec.md Constraints + plan.md change 4 + execution.md all identify change-drt-011-phase-close-distribution-regen as the C-01 owner. No generator inputs touched (`shared/launchagents/**`, `shared/systemd/**`, `.claude-plugin/*`, `.mcp.json`, `hooks/hooks.json`, `scripts/build-codex-plugin.js` all untouched by this change). No distribution certification claimed. |
| C-02 no committed secrets | **PASS** | `grep -inE 'api[_-]?key|token|password|secret'` over both produced files: no hits (scan output empty). |
| C-03 docs with surface changes | **PASS (N/A)** | No Codex plugin surface (manifest/marketplace/MCP/hooks/install) altered. |
| C-04 generators idempotent | **PASS (N/A)** | No generator edited. |
| C-05 bash 3.2 compatibility | **PASS** | No `mapfile`, no `declare -A` (grep clean); `/bin/bash -n` parse passes. Honest note: the campaigns executed under bash 5; full execution under `/bin/bash` 3.2 was not re-run — construct absence plus parse is the evidence, stated as such. |

## Artifact checks

- `run-hang-capture.sh`: executable, `bash -n` + `/bin/bash -n` clean, `--runs/--timeout/--scenario/--out/--stop-on-hang` flags present, capture-before-teardown (ps tree + sample + output volumes) implemented, scoped PID-only cleanup (no name-based kill).
- `HANG-CAPTURE.md`: non-empty; failure rate present (bounded 0/37); blocked-command section present ("none captured — no run hung"); `method: uncommitted-only` dating line; uncommitted-tree boundary stated.
- Pre-existing files untouched by this change: `driver-contract.sh` content unchanged (its `A` in `git status` is the phase's pre-existing staged tree from 2026-09-09, not this change).
- No orphaned driver processes after campaigns (`pgrep` clean).

## Result

**ALL PASS.** Blocking constraints: 5/5 PASS (two N/A by scope, recorded). Proceed to adversarial-review diff mode.

## Round 2 — revisions after adversarial-review diff BLOCK (round 1)

Round-1 findings (harness-native reviewer; packet had been rebuilt after the untracked-files/scoped-diff defect was found and fixed): 2 CRITICAL + 2 WARNING + 1 SUGGESTION, all verified legitimate and fixed:

1. CRITICAL (teardown depth): kill_recorded_tree killed only direct children and early-returned when the root was already dead — grandchildren could orphan. FIXED: cleanup now kills the full recorded tree (root + per-tick descendant snapshot refreshed every poll second, so reparented children remain reachable), TERMs all, re-walks a surviving root, KILLs survivors, warns on any survivor. Verified by a live smoke run: clean teardown log + pgrep clean.
2. CRITICAL (evidence): per-run tables lived only in uncommitted captures. FIXED: all 37+1 per-run rows inlined into HANG-CAPTURE.md as three tsv blocks.
3. WARNING (termination semantics): spec said "K or first hang", default continued. FIXED by amending spec.md + tasks to the plan-stage resolution: all K runs by default, capture-and-continue on hang, --stop-on-hang opt-in.
4. WARNING (false comment): lstart PID-recycling defense claimed but unimplemented. FIXED: comment removed; behavior now documented as recorded-list teardown (the per-tick snapshot is the actual mechanism).
5. SUGGESTION (tree artifact): rooted tree now materialized as $label-tree.txt (ps rows joined to root+descendants with command lines).

Re-verified after revision: bash -n + /bin/bash -n PASS; task-1 verify string PASS; live smoke run clean (12/12, 14 s), no orphaned processes. **ALL PASS — proceed to adversarial-review diff round 2.**

## Round 3 — revisions after diff round 2 BLOCK

Round-2 findings (1 CRITICAL + 2 WARNING + 2 SUGGESTION), all verified legitimate, all fixed:
1. CRITICAL (claimed-but-unimplemented perturbation marking): now implemented — RESULT="clean-perturbed" for any clean run following a hang (PERTURBED flag).
2. WARNING (N arithmetic): corrected in HANG-CAPTURE.md — N = ceil(54.46) = 55 (was ≈53, which missed the 99% bound).
3. WARNING (start_utc recorded completion time): script now captures START_TS before launch; historical tables carry an explicit caveat note (true starts = value − duration; campaign dir names hold true starts).
4. SUGGESTION (stale last.pid after ~27s of sample calls): snapshot refreshed immediately after capture_hang, before teardown.
5. SUGGESTION (nondeterministic 8-sample cap): descendants sorted (sort -n, lowest pid first) before the cap.

Re-verified: bash -n + /bin/bash -n, executable, task-1 verify string PASS. **ALL PASS — proceed to diff review round 3.**

## Round 4 — revisions after diff round 3 BLOCK

Round-3 findings (1 CRITICAL + 3 SUGGESTION), all legitimate, all fixed:
1. CRITICAL (replace-vs-merge regression): post-capture snapshot refresh now MERGES the pre-capture recorded list with the live walk (sort -un) — a root dying during capture can no longer erase recorded PIDs.
2. SUGGESTION (hang duration inflated): hang rows now record DUR at the timeout instant (excludes ~27s capture + teardown time); clean/failed rows unchanged.
3. SUGGESTION (scratch-reset delegation undocumented): HANG-CAPTURE.md now states scratch freshness is delegated to the driver's per-invocation mktemp and roots persist intentionally.
4. SUGGESTION (failed runs labeled clean): RESULT now distinguishes failed / failed-perturbed from clean / clean-perturbed.

Re-verified: bash -n + /bin/bash -n, executable, task-1 verify string PASS. **ALL PASS — proceed to diff review round 4.**
