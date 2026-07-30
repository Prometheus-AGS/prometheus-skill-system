# Reflection — adversarial-review-for-creation

**Closed:** 2026-07-30 · **Changes:** 9 / 9 implemented, verified, archived

## Goal achievement — 7/7 MET

Every verdict below was re-checked against the shipped artifacts at reflect time,
not carried forward from the turn that implemented it.

| # | Goal | Verdict | Evidence |
|---|---|---|---|
| 1 | Wire review into `pmpo-skill-creator` Reflect | **MET** | `prompts/reflect.md` Step 12, positioned after `validate-skill.sh` and before the Loop Decision |
| 2 | Wire review into native-agent generation | **MET** | `prompts/generate.md`, after `cargo check`/`npm install`, before the readiness banner |
| 3 | Artifact-mode packets for skill tree + Cargo workspace | **MET** | `build-review-packet.sh --mode skill\|agent`; `reviewer-mandate-{skill,agent}.md` |
| 4 | Enforce `KBD_PRODUCER_MODEL` at both creator entry points | **MET** | `kbd_require_producer_model()`, defined exactly once; suite Group B asserts exit 2 + no artifact for both creators |
| 5 | Blocking on CRITICAL with a bounded cap | **MET** (with a stated boundary — see below) | `review-retry-loop.sh state` → `CAPPED`/exit 4; `validate-skill.sh` → exit 1 |
| 6 | Promote the sycophancy pass to an enforced gate | **MET** | `validate-skill.sh` group 8 shells out to `check-findings-sycophancy.sh`, propagating exit into `FAIL` |
| 7 | Prove the loop end to end | **MET** | Live suite re-run at reflect: 4/4 fixtures sorted correctly, all `verified-distinct`, **27 assertions / 0 failures** |

### Goal 5 — what "blocking" does and does not mean

Both *decision points* are code: `review-retry-loop.sh` returns `CAPPED` (exit 4)
and `validate-skill.sh` exits 1. But the two creators are **prompt files**, not
executables. Nothing in the runtime physically prevents a model from ignoring
`CAPPED` and printing the banner anyway.

This is a real limit, and it is the same limit every prompt-driven skill in this
pack has. It is recorded here rather than papered over: goal 5 is met in the sense
that the gate returns an unambiguous machine-readable refusal and the creators are
instructed to honour it — not in the sense that the refusal is unbypassable.
Closing that gap means moving creator orchestration into a script, which is a
larger change than this phase scoped. See TD-01.

## The headline result

Before this phase, **all 8 stored `findings.json` in the repository recorded
`isolation_mode: harness-native` and `PASS`** — every adversarial review ever run
here was Claude judging Claude, and the pipeline reported success throughout.

The fixture suite now demonstrates the opposite, live and repeatably:

```
flawed-skill → BLOCK    clean-skill → PASS
flawed-agent → BLOCK    clean-agent → PASS
all four: cross_model_check = verified-distinct
```

The judge named the specific planted defects — a script invoked but not shipped,
a read-only intent contradicted by a ticket-writing prompt, a required MCP server
left `enabled = false`, a provider/model mismatch. That is discrimination, not a
coincidental block.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0 / 9** |
| First-pass pass rate | n/a — refiner not run |
| Substitute gates run per change | `npm run validate` (145/145), `bash -n` on every touched script, `openspec verify` |
| Live cross-model assertions | 27 (fixture suite) + 21 (reject-cap test) |

**No `.refiner/artifacts/*arc-*` logs exist.** The artifact-refiner QA gate named
in the kbd-apply contract was not run for any of the nine changes. Recorded as a
process gap (TD-04), not as a pass.

## Technical debt introduced

| ID | Debt | Why it was accepted |
|---|---|---|
| TD-01 | Creator blocking is prompt-instructed, not runtime-enforced | Making it unbypassable requires script-driven creator orchestration — a larger change than this phase scoped |
| TD-02 | **Nothing in this phase is committed.** 150 changed paths, incl. a new submodule and `.gitmodules` | Work was validated continuously; committing is the immediate next action |
| TD-03 | Plugin caches are stale — 6 drift findings | Repo is ahead; `bash scripts/update-skill-pack.sh --force` refreshes. **The producer guard is not live in installed copies until then** |
| TD-04 | artifact-refiner QA gate skipped for all 9 changes | Substituted validate + syntax + openspec verify; not equivalent |
| TD-05 | `scripts/detect-command-conflicts.sh:40` uses bash-4.2 `[[ -v arr[k] ]]`, fails on macOS 3.2 | Pre-existing, latent (parses under bash 5, nothing invokes it); found by a sweep, left out of scope |
| TD-06 | `install-binaries.sh:131` (liter-llm) lacks the `\|\| true` guard | Survives only because its `target/` exists; documented in CLAUDE.md rather than silently patched |
| TD-07 | 10 near-duplicate `.prometheus/knowledge/wiki/*completion*` entries auto-generated this phase | pk ingest noise; `pk lint` dedup is a separate concern |
| TD-08 | `/kbd-execute` was never run — the phase went plan → `/kbd-apply` directly, so no `execution.md` exists | Caught by the reflect stage gate, which refused to pass on a missing execute handoff. Recorded as an explicit skip rather than backfilled. This is the **root cause of TD-04**: the artifact-refiner QA gate lives in the execute contract, so skipping the stage silently skipped the gate for all 9 changes |

## Lessons

**A green test proves nothing until you have watched it go red.** The fixture
suite itself shipped a false green: I named a variable `GROUPS`, which is a bash
**read-only built-in array** of the caller's group IDs. The assignment was silently
discarded, no group matched, zero assertions ran, and it printed
"✅ the gate discriminates". In the tool whose entire purpose is catching false
greens. Fixed by renaming, and by making zero assertions a hard exit 2. Every
subsequent group was mutation-tested before being believed.

**A fixture that flips verdicts is evidence about the fixture first.** `clean-skill`
blocked on ~50% of runs. The tempting reading was judge non-determinism; the
tempting fix was a retry. Both wrong — the judge was correctly finding that the
fixture compared `pg_stat_user_tables.n_live_tup`, a planner *estimate*, so the
procedure could certify a lossy restore as complete. Fixing the fixture made it
4/4. An inversion that gets retried away is an inversion nobody understood.

**Documentation work keeps surfacing live defects.** Writing docs for
`validate-skill.sh` exposed that it had **never run past its first passing check**:
`((PASS++))` as a function's last command returns the pre-increment value, which
under `set -e` is fatal. Plus `head -n -1` (GNU-only, empty frontmatter on macOS)
and a cross-ref grep that flagged qualified paths as broken. 1 check → 20. This
phase depended on that script being the enforced gate; a gate running one check is
not a gate.

**Verify the CLI, don't infer it from the plan.** The plan assumed
audit-then-install. `cowork audit` takes no repository argument — it scans
*installed* skills, and `install` has no `--dry-run`. The honest flow is read
source → install to project scope → audit → verify, with the residual gap stated.

**An apostrophe inside `$( … )` broke a heredoc three times** across three
separate scripts. Cheap to fix, invisible until `bash -n`. Worth a linter.

## Recommended Next Phase

**`ideation-and-decision-tools`**

The assess stage captured a large vision — persona teams with a judge, business-model
vetting, coach/reflector personas, Feynman + Karpathy loops for personal and business
development, delivery across Claude Desktop / Codex / Kimi, and hooks into the
librefang/bossfang orchestrator. None of it was in scope here, and the adversarial
machinery this phase hardened is exactly the substrate it needs: a real cross-model
judge, manifest-level packets, and a bounded, auditable rejection gate.

**Before starting it**, clear TD-02 and TD-03 — commit this phase and refresh the
plugin caches. The producer-model guard is not live in any installed copy until the
caches are refreshed, which means creators running from a cache still cannot make
the judge≠producer guarantee this phase exists to provide.
