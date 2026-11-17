# Certification Phase 2: Pilot

**Date:** 2026-11-17
**Plan:** [`docs/plans/2026-11-17-skill-pack-certification-plan.md`](../plans/2026-11-17-skill-pack-certification-plan.md)
**Scope:** five skills, 15 eval cases, Claude Code only, one model.

## Result

**The largest finding is about the pack, not any one skill: skills that trigger every time when loaded alone stop triggering when the full pack is loaded.** `kbd-status` and `prometheus-rust-workspace` fired on 6 of 6 positive runs in isolation and 0 of 6 with all 218 skills present. A diagnostic run showed that no skill at all was invoked in those sessions, so they are not losing to a competitor.

Where a skill does trigger, it earns its place. Every skill that fired improved the graded outcome over the no-skill baseline, by 0.22 to 0.67 on a 0 to 1 scale.

The pilot met its exit conditions: thresholds are proposed from data, and the cost per skill is known.

## Setup

| Item | Value |
|---|---|
| Runner | `claude plugin eval`, Claude Code 2.1.326 |
| Model under test | `claude-sonnet-5-5` (pinned for cost; not the session default) |
| Judge model | runner default (Haiku), three votes per LLM grader |
| Runs | 3 per arm, with-skill and without-skill |
| Cases per skill | 2 outcome cases, 1 near-miss case that must not trigger |
| Isolation | The runner gives each run a temporary home and loads only the plugin under test. No user settings, hooks, memory or other plugins |
| Publishing | `--no-publish`; reports stayed on this machine |

Skills: `vlbv`, `prometheus-rust-workspace`, `kbd-status`, `learn-grade`, `feynman-loop`.

Each case was run in two modes:

- **Isolated:** the skill directory is the only thing loaded.
- **Pack:** the whole generated Claude plugin (218 skills) is loaded from a scratch copy.

## Results

"Fired" counts with-skill runs where the Skill tool was called for the skill under test. Scores are the mean over three runs; Δ is with-skill minus without-skill.

### Outcome cases

| Skill | Case | Isolated fired | Isolated with / without / Δ | Pack fired | Pack with / without / Δ |
|---|---|---|---|---|---|
| `vlbv` | mid-phase-test-request | 0/3 | 0.00 / 0.00 / 0.00 | 0/3 | 0.00 / 0.00 / 0.00 |
| `vlbv` | boundary-verdict-vocabulary | 0/3 | 0.33 / 0.33 / 0.00 | 0/3 | 0.33 / 0.33 / 0.00 |
| `prometheus-rust-workspace` | tokio-task-command-plan | 3/3 | 1.00 / 0.33 / **0.67** | 0/3 | 0.33 / 0.33 / 0.00 |
| `prometheus-rust-workspace` | phase-end-verification-command | 3/3 | 1.00 / 0.67 / **0.33** | 0/3 | 0.67 / 0.67 / 0.00 |
| `kbd-status` | mid-phase-status | 3/3 | 1.00 / 0.40 / **0.60** | 0/3 | 0.40 / 0.40 / 0.00 |
| `kbd-status` | implementation-complete-certification-pending | 3/3 | 1.00 / 0.33 / **0.67** | 0/3 | 0.33 / 0.33 / 0.00 |
| `learn-grade` | flags-seeded-misconception | 3/3 | 0.75 / 0.17 / **0.58** | 3/3 | 0.83 / 0.17 / **0.67** |
| `learn-grade` | passes-correct-explanation | 3/3 | 1.00 / 0.33 / **0.67** | 3/3 | 1.00 / 0.33 / **0.67** |
| `feynman-loop` | self-report-does-not-close-loop | 3/3 | 0.89 / 0.67 / **0.22** | 3/3 | 1.00 / 0.67 / **0.33** |
| `feynman-loop` | states-closure-criteria | 2/3 | 0.67 / 0.00 / **0.67** | 1/3 | 0.33 / 0.00 / **0.33** |

The `learn-grade` rows use the corrected graders (see "A defect in the cases").

### Near-miss cases

All five near-miss cases stayed quiet in both modes: 15 of 15 runs isolated, 15 of 15 in the pack. No false triggering was observed.

### Trigger rate by skill

| Skill | Isolated | Pack |
|---|---|---|
| `vlbv` | 0/6 | 0/6 |
| `prometheus-rust-workspace` | 6/6 | **0/6** |
| `kbd-status` | 6/6 | **0/6** |
| `learn-grade` | 6/6 | 6/6 |
| `feynman-loop` | 5/6 | 4/6 |

## Findings

### 1. Triggering collapses when the full pack is loaded

Two skills went from 6/6 to 0/6. A follow-up run added graders that pass if any skill is invoked. They failed on every run, for `kbd-status`, `prometheus-rust-workspace` and `vlbv` alike: the model called no skill.

The two skills that kept triggering in the pack are the ones whose prompts use the skill's own vocabulary ("grade my explanation", "Feynman loop"). The prompts for the skills that went silent describe the need without naming it ("where are we on this project?").

**Likely cause, not verified:** the skill catalogue exceeds what Claude Code presents in full, so descriptions are shortened or dropped, as `CLAUDE.md` already documents for Codex. `evals/skill-activation/harness-budgets.json` records the pack's description inventory at 44,882 characters and lists the Claude Code budget as unmeasured. Measuring that budget is the next step; this pilot did not do it.

The consequence for certification: **a trigger result from an isolated run says nothing about the shipped pack.** Level C2 has to be measured with the pack loaded.

### 2. `vlbv` does not trigger even alone

0 of 6 in isolation. Its description says what the skill is and carries no "use when" clause, which is what Phase 1's static check flagged for about 150 skills. This is the first behavioural confirmation that the static warning predicts a real failure.

`vlbv` is the pack's verification method and is reached mainly through other skills and hooks, so a direct trigger may matter less for it than for most. That is a judgement for the owner.

### 3. Where skills fire, they change the outcome

Every outcome case with a firing skill showed a positive Δ. The checks that separated the arms were the specific ones: `Implementation: 2/4` and the completion signal for `kbd-status`, deferring Cargo to the boundary for the Rust skill, the 0.7 threshold and 24-hour retention rule for `feynman-loop`, the misconception score for `learn-grade`.

In the pack, `kbd-status` and the Rust skill scored exactly the same with and without the plugin, because the skill never loaded. Outcome and triggering are separate failures and need separate levels, as the plan has them.

### 4. Noise comes from two places only

Within an arm, deterministic graders (regex, tool use) gave identical results on all three runs in every case. The variation that did appear had two sources:

- **Whether the skill fired.** `feynman-loop` states-closure-criteria fired 2 of 3 times isolated, and its four graders passed on exactly those 2 runs.
- **LLM judges.** Split votes ("FAIL PASS FAIL") appeared on the `learn-grade` leads-with-error check and once on a near-miss "answers the request" check.

So three runs are enough to see an outcome difference, and too few to state a trigger rate more finely than thirds.

### 5. A possible weakness in `learn-grade`

The check "leads with the error, not praise" passed on 1 of 3 isolated runs and 2 of 3 pack runs with the skill, against 0 of 3 and 2 of 3 without. The judge votes were split, so this may be judge noise. It is also the behaviour the anti-sycophancy mandate exists to prevent, so it deserves a deterministic check and more runs before anyone relies on it either way.

## A defect in the cases

The first `learn-grade` run scored the skill at 0.42 and 0.33. The skill was behaving correctly. It reports scores as a table ("Misconceptions absent | 0.00 |") and my graders only accepted JSON (`"misconceptions_absent": 0.0`). After widening the patterns the same skill scored 0.75 and 1.00.

A grader that is too strict looks exactly like a failing skill. The run does not keep transcripts, so the cause was found only by reading the HTML report. This became a required step in the process below.

## Thresholds proposed from this data

| Level | Rule | Basis |
|---|---|---|
| **C2 Trigger** | With the full pack loaded: positive cases fire on at least 80% of runs pooled, no positive case at 0, and near-miss cases never fire. At least 5 runs per case | 3 runs only resolve thirds; 15/15 near-miss runs were quiet, so zero tolerance is achievable |
| **C2-isolated** | Same rule with only the skill loaded. Reported, not sufficient | Finding 1 |
| **C3 Outcome** | Every positive case has Δ > 0; mean Δ across cases is at least 0.25; mean with-skill score is at least 0.80; at least one deterministic grader passes with the skill and fails without it | Observed Δ ranged 0.22 to 0.67; the smallest came from a case where two of three graders passed in both arms |
| **Grader hygiene** | A grader that passes in both arms on every run is reported as non-discriminating and does not count towards C3 | Ten of the 34 outcome graders behaved this way in the isolated runs |

Applied to the pilot, as an illustration and not as issued certificates:

| Skill | C2 (pack) | C3 (isolated) |
|---|---|---|
| `learn-grade` | meets | meets |
| `feynman-loop` | 4/6, below 80% | Δ meets; mean with-skill score 0.78, just below |
| `kbd-status` | fails (0/6) | meets |
| `prometheus-rust-workspace` | fails (0/6) | meets |
| `vlbv` | fails (0/6) | not measurable; never fired |

## Cost

Total spend for the pilot was **$10.74** across 13 suite runs, including smoke tests, the re-run after the grader fix and the diagnostic.

| Measure | Isolated | Pack |
|---|---|---|
| With-skill run | $0.03 to $0.07 | $0.08 mean |
| Without-skill run | $0.02 to $0.04 | $0.03 mean |
| One skill, 3 cases, 3 runs per arm | $0.57 to $0.76 | about $1.00 |
| Wall time per run | 6 to 34 s | 10 to 47 s |

The with-skill run costs more in the pack because the 218-skill catalogue is in context on every call.

Rough projection at the same depth (3 cases, 3 runs) on this model: about $140 for all 198 native skills isolated, about $200 in the pack. C2 at five runs per case costs proportionally more. A more capable model will cost more; that was not measured.

## Corrections to the plan

1. **Prior eval work exists.** The plan and the Phase 1 report said the repository had no behavioural evals. That was wrong. `evals/vlbv-protocol/` is a cross-harness runner with per-harness isolation and reports for Claude Code and Kimi; `evals/skill-activation/critical-30.json` holds 30 trigger cases for five KBD skills, checked by `scripts/check-skill-evals.js`. What was true is narrower: no skill directory had its own `evals/` folder. Phase 4 should extend the existing cross-harness runner rather than introduce Harbor alongside it.
2. **Decision 1 is settled by the tools.** The current skill-creator writes cases in the folder layout `claude plugin eval` reads (`prompt.md`, `case.yaml`, `graders/`). There is one format, not two.
3. **Cases live in the skill** (`<skill>/evals/`), and the distribution generator now leaves them out of the built package. Pack-mode runs copy them next to a scratch copy of the generated plugin, because the runner refuses an eval directory inside a loaded `skills/` tree and the repository root is a marketplace, not a plugin.
4. **`claude plugin eval` is available** on this machine. The plan had it as early access and uncertain.

## Process notes for the certification skill

This is the procedure that worked, written so it can become the skill.

1. **Static gate first.** Run the spec validators before spending on model runs; a skill that fails to parse cannot be evaluated.
2. **Read the skill's contract, then write cases.** For each skill: two outcome cases built from its most specific, checkable promises, and one near-miss prompt that shares vocabulary but needs something else.
3. **Prompts must not name the skill.** A prompt that uses the skill's own words measures recall of the name, not the description. Write at least one positive prompt the way a user who has never heard of the skill would.
4. **Prefer deterministic graders.** Regex over the final message or a produced file, tool-use counts, file existence. Use an LLM judge only for what cannot be pattern-matched, and expect split votes.
5. **Always include a `tool_used: Skill` grader** marked with-only. It is the trigger measurement and it explains every Δ of zero.
6. **Smoke-run one case once** before the full run. It confirms the plugin resolves, shows the cost, and catches malformed case files.
7. **Calibrate graders against the first real output.** If a specific grader fails on every with-skill run while the skill fired, read the output before believing the score. Accept the formats the skill legitimately produces.
8. **Run both modes.** Isolated for outcome, pack for triggering.
9. **When a skill does not fire, ask what did.** Add temporary any-skill and named-competitor graders. "Nothing fired" and "a neighbour fired" need different fixes.
10. **Report non-discriminating graders** and remove or sharpen them.
11. **Record the model, the harness version and the content hash** with every result. A result is evidence for that combination only.
12. **Keep the spend bounded** with `--max-cost-usd`, and never publish reports (`--no-publish`).

Runner gotchas to encode:

- The eval directory cannot sit inside a loaded `skills/` tree and its name must start with a letter or digit.
- A case's `plugins:` entry cannot point outside the plugin under test.
- A second `--case` flag replaces the first; use one glob.
- Scaffold scripts run only with `--scaffold`.
- The runner exits 1 when any case is below the threshold. That is a finding, not a tool failure.
- Traces are deleted after the run; the HTML report is the only place to read outputs afterwards.

Reusable pieces added in this phase: `scripts/certification/run-skill-evals.sh` (both modes) and `scripts/certification/summarize-evals.mjs`.

## Limits

- One harness, one model, three runs. Nothing here says how these skills behave on Codex, OpenCode or a different Claude model.
- Fifteen cases written by one author in one sitting, with one known defect already found and fixed. Others may remain.
- The catalogue-budget explanation for finding 1 is a hypothesis.
- No certificate was issued. The table of verdicts illustrates the thresholds.
- The cases were not reviewed by the skill owners.
- Raw results are in the session scratchpad and are not committed.

## Downstream change

`prometheus-entity-management` was the only submodule failing the official validator on its own skills. The same frontmatter change was applied to its 28 source skills (28 of 28 now pass) and opened as [Prometheus-AGS/prometheus-entity-management#80](https://github.com/Prometheus-AGS/prometheus-entity-management/pull/80). That repository's skills gate was run without installed dependencies and reported FAIL; the one failure traced fails identically on its unmodified `main`. The submodule pin in this repository is unchanged.
