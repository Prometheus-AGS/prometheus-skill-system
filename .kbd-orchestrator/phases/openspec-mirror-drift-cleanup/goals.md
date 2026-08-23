# Goals

Phase: `openspec-mirror-drift-cleanup`
Repository: `prometheus-skill-system` (local dir `prometheus-skill-pack`)
Created: 2026-08-23
Origin: surfaced while repairing `preflight-models.sh` (PR #55, `c0d2de1`).
`update-skill-pack.sh` refuses a dirty source tree, so 98 uncommitted files
block every reinstall — including the one that would ship that merged fix.

## The finding that shapes this phase

The working tree is **not one drift**. It is **three unrelated concerns** that
happen to be uncommitted at the same time, and merging them into a single
"clean up the drift" commit would bury two real decisions inside a mechanical one.

| # | Concern | Size | Kind |
|---|---|---|---|
| A | OpenSpec mirror version churn | 70 M | mechanical — `generatedBy` 1.3.1 → 1.10.0 across `.agent`, `.agents`, `.cursor`, `.opencode` |
| B | `.windsurf/skills` removed entirely | 20 D | a decision — the CLI no longer emits it; is Windsurf still supported? |
| C | `prometheus-entity-management` submodule | 1 M | a decision — `1c40eaa..55dc8dc`, real upstream commits (release-gate convergence work) |
| — | session logs + `.devin/` + `.openspec-target` | 3 M, 4 ?? | routine |

**A is not the same defect HMA's c300 fixed.** HMA's `check-skill-contracts.mjs`
enforces `metadata.internal: true`, so the CLI stripping it turned a gate red.
This repo has **no such contract check** (verified: no `internal: true` in any
`scripts/*.mjs`), so nothing here is red — the mirrors are simply stale. That
makes A a commit, not a repair, and it means the c300 normalizer pattern is a
*prevention* question here rather than a *fix* one.

## Goals (candidate — subject to `/kbd-assess`)

1. **Unblock `update-skill-pack.sh`** by resolving all 98 files, so the merged
   `preflight-models.sh` fix (PR #55) can actually reach installed surfaces.
   This is the phase's reason to exist; everything else serves it.

2. **Commit concern A as its own change**, with the `generatedBy` version bump
   stated in the message so the next reader knows an external tool wrote it and
   why the diff is large and uninteresting.

3. **Decide concern B.** `.windsurf/skills` is fully gone (0 dirs on disk, 1 at
   HEAD) while `.windsurf/workflows` survives. Either Windsurf is still a
   supported harness and the skills must be regenerated, or it is not and the
   deletion is correct and should be recorded as such.

4. **Decide concern C.** The submodule advances four upstream commits of
   release-gate work. Adopt the pointer deliberately, or pin it back — do not
   let it ride along inside a drift commit.

5. **Decide whether this repo wants the c300 prevention pattern.** A normalizer
   plus a per-harness completeness assertion stops `openspec update` from
   silently dropping a whole tree. With no contract check here, a tree could
   vanish and nothing would notice — which is exactly how `.windsurf/skills`
   reached 0 without anyone deciding it should.

6. **Then reinstall and verify** the live installed `preflight-models.sh`
   contains `resolver_missing`, and that a run through `~/.claude/skills/...`
   reports `status: ok` without `CLAUDE_PLUGIN_ROOT` exported.

## Non-goals

- Re-litigating the `preflight-models.sh` fix itself (merged as `c0d2de1`).
- Touching the HMA repo, which is clean at `1d26aa6` and unaffected.
- Any work in `prometheus-companion`.

## Method (binding)

- **Verify each premise against the artifact it names before encoding it.** The
  HMA phase recorded this lesson three times; here it already paid off — the
  "same drift as c300" framing was wrong, and the difference (no contract check)
  changes what goals 2 and 5 mean.
- `/kbd-assess` runs with **adversarial review**, which is genuinely cross-model
  now: judge `k3` vs producer `claude-opus-5`, `verified-distinct`, provided
  `CLAUDE_PLUGIN_ROOT` is exported until goal 6 lands.
- **One commit per concern.** A, B, and C do not share a commit.
- Never `git checkout`/`reset` the working tree wholesale — the 98 files include
  a submodule pointer and session logs that are not disposable.
- Local-only validation. No CI runs started or cited.
