---
type: SessionRecord
id: karpathy-session-d106140ee22ad448
title: Karpathy session d106140ee22a
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T14:23:46.246838+00:00
created_at: 2026-11-17T14:23:46.246838+00:00
updated_at: 2026-11-17T14:23:46.246838+00:00
revision: 0
---

## Delta

The `skill-certification` skill is built and registered in the testing plugin. It is uncommitted on branch `feat/skill-certification-skill` in a scratch worktree, which will not survive a reboot.

## What it does

One command certifies a skill and writes `certificate.json` and `report.md`:

```bash
node skills/testing/skill-certification/scripts/certify.mjs skills/<domain>/<skill> --out <dir>
```

- **C0 and C1 run for free.** C0 needs two validators to agree: built-in checks plus `skills-ref` or the repo's strict validator. With only one, the result is `INCOMPLETE`, not a pass.
- **C2 and C3 need eval results.** Pass existing files with `--results`, or add `--run both` to make billed runs bounded by `--max-cost-usd`. The skill text tells the model to ask the operator before spending.
- **C2 is measured with the installed plugin loaded**, and the certificate records which plugins were loaded and the listing size.
- **A failed level becomes an operator decision.** The report lists what failed, a proposed fix, and the impact of fixing, accepting or abandoning. `--accept C2=<reason>` records an exception without raising the level. `--require C1` exits 2 below that level, so it can be used as a gate.

## Does it work

I ran it on the five pilot skills using the stored Phase 2 results, at no cost, and it reproduces the pilot's verdicts:

| Skill | C2 | C3 |
|---|---|---|
| `learn-grade` | incomplete: 3 runs, rule needs 5 | pass |
| `feynman-loop` | fail, 67% fired | fail, mean score 0.78 |
| `kbd-status` | fail, never fired | pass |
| `prometheus-rust-workspace` | fail, never fired | pass |
| `vlbv` | fail, never fired | fail |

All five pass C0 and C1 and hold level C1. The skill certifies itself at C1. Strict validation, domain plugin coverage, the distribution checks and docs sync pass.

One bug surfaced in that run and is fixed: near-miss cases were counted as firing when they had stayed quiet.

## What is not done

- **The `--run` path was never executed**, because it costs money. The runner script it calls is the Phase 2 one with small edits, which is also untested since the edits.
- **The skill has no eval cases of its own**, so its own C2 and C3 are not exercised.
- **Security scanning and third-party linters are not automated.** The certificate lists them as not exercised.
- **C4 (other harnesses) is recorded as not exercised.**
- **A should-stay-quiet case is recognised only by a name starting `near-miss-` or `quiet-`**, or by the `--negative` flag. The `vlbv` case `plain-project-stays-optional` is therefore counted as a positive case.

## Other changes in the branch

- The three reusable scripts moved from `scripts/certification/` into the skill so they ship with it, and no longer need `js-yaml`. A README at the old location points to the new paths.
- The testing plugin goes from 1.6.0 to 1.7.0. Marketplaces, `SKILLS.md`, the packaged payloads and generated docs are regenerated.
- `CLAUDE.md` gains a short section on certification.

Once PR #232 merges, this branch needs `npm run generate:skill-domains` so the new skill is in the domain index. The next agreed step after this is rewiring `pmpo-skill-creator` to call it.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T14:23:36.259344Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
