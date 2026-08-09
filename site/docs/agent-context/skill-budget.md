---
title: Skill Budget
description: Why installed skills stop firing, how to measure the real number, and why raising the budget does not fix it.
---

# Skill Budget

The symptom is specific and confusing: a skill exists, it tested fine, and it does
not fire.

The cause is not the skill. Claude Code reserves a fraction of the context window
for **all skill descriptions**, governed by `skillListingBudgetFraction` (default
`0.01`, one percent). Past that budget, descriptions are dropped silently. The
skill keeps its name in the listing; its description vanishes; auto-triggering
stops.

Eviction ranks by usage recency and frequency, roughly
`usageCount × 0.5^(days/7)`. A newly installed skill has never been used, scores
zero, and is dropped first — a catch-22 where the skill can never auto-trigger
because it has never auto-triggered.

## Measure it; do not assume it

The measurement trap is the denominator. A repo-local count reports headroom that
does not exist, because the budget spans **every scope the harness loads**.

```bash
bash "$SK/scripts/skill-budget.sh" --path .
bash "$SK/scripts/skill-budget.sh" --path . --json
```

Measured on one estate, 2026-08-09:

| Scope | Skills | Description chars |
|---|---|---|
| repo `.claude/skills` | 56 | 13,078 |
| user `~/.claude/skills` | 916 | 251,125 |
| plugins `~/.claude/plugins` | 1,295 | 388,816 |
| **total** | **2,267** | **653,019 (~163,000 tokens)** |

Against a ~4,000-token budget (0.02 × 200k) that is **~41× over**.

Counting only the 56 repo-local skills gives ~3,300 tokens and looks like tenfold
headroom. It is the same machine. The difference is entirely the denominator.

```
OVER BUDGET by ~40.8x.
Descriptions past the budget are dropped silently.
Raising skillListingBudgetFraction does not fix a multiple this large.
```

Exit 0 within budget, 1 over, 2 could not measure.

## Parse frontmatter properly

`description: >` is a **YAML folded block scalar**. The text continues on the
indented lines below it.

```yaml
---
name: my-skill
description: >
  This is the real description. It continues
  across several lines and is not empty.
---
```

`grep -m1 '^description:'` returns `>` and reports a one-character description.
Any audit built on that will invent dead skills that are fine and miss ones that
are genuinely broken.

This is not hypothetical: a measurement pass flagged three healthy skills as having
empty descriptions, and the same bug affects at least one skills-index generator in
the wild, which is why some catalogs render a bare `>` in the description column.

`skill-budget.sh` uses PyYAML, with a folding fallback when it is unavailable, and
reports SKIP rather than PASS when it cannot parse.

## Raising the fraction is not the fix

At 41× over, moving `skillListingBudgetFraction` from `0.02` to `0.03` changes
nothing that matters. It also costs context that the actual work needs.

The bootstrap writes `0.02` because the `0.01` default is too tight for any
non-trivial profile, not because `0.02` is sufficient. It buys headroom for a
modest profile. It does not solve sprawl.

## What actually works

**Gate the long tail behind plugins.** A plugin bundles skills, commands, hooks,
and MCP servers as one installable unit. Only enabled plugins contribute
descriptions to the budget. Split a large pack into domain plugins — rust-core,
ts-web, flutter, memory, governance — and enable the one the current work needs.

**Keep one always-on router.** A small meta-skill that reads intent, git state,
and the waypoint phase, then dispatches **by name**. Name invocation works even
when a description was evicted, which inverts the problem: instead of N
descriptions competing for auto-trigger, one router decides. Its own description
must stay short.

**Tighten descriptions.** Aim for ~100-150 characters with trigger keywords
front-loaded. A description written for a human reader costs budget without
improving matching.

**Prune.** `/skills` disables what is not in use. A skill installed and never
invoked is pure budget cost.

**Seed new skills.** Invoke a newly installed skill by name once so it has a usage
score and stops being first in line for eviction.

## Verify reports it as WARN, not FAIL

```
WARN  skill budget measured   2267 skills, ~163254 tok vs ~4000 — 40.8x OVER
                              (machine-wide; repo contributes 13078 chars)
WARN  skills can auto-trigger 9 with empty descriptions (machine-wide)
```

The budget is a property of the machine, not the repository. Reporting it as FAIL
would leave every repo permanently red, and a gate that always fails stops being
read — the same decorative failure the SKIP rule exists to prevent, arriving by a
different route.

WARN does not change the exit code, so `verify.sh` still works as a CI gate. The
finding is real; it is simply not one this repository can fix on its own.

The line reports what the repo contributes, so you can tell repo sprawl from
machine sprawl.

## Diagnosing a skill that will not fire

1. `bash "$SK/scripts/skill-budget.sh" --path .` — is the profile over budget?
2. Is the skill's description empty or malformed? Check with a YAML parser, not
   grep.
3. Is the skill newly installed? Invoke it by name once to seed its usage score.
4. Is it disabled? `/skills`.
5. Is its description written for a human rather than for matching? Front-load the
   trigger keywords.
6. If the profile is many multiples over budget, stop debugging the individual
   skill. It is a sprawl problem, and only gating fixes it.
