---
title: Theory and Sources
description: Where this architecture comes from, which claims are measured, which are borrowed, and what would refute it.
---

# Theory and Sources

Four independent lines of evidence converge on the same structure. This page
separates what is documented from what is measured locally from what remains
assumption, because a design whose provenance is unclear cannot be argued with.

## 1. Instruction-following degrades with density

Peer-reviewed benchmarks, independent of any vendor's product claims.

**IFScale** (Jaroslawicz, Whiting, Shah, Maamari, arXiv:2507.11538) evaluates
instruction-following at increasing density across twenty frontier models from
seven providers. Even the best models fall well short of full compliance at the
maximum density tested. Three degradation patterns appear — threshold, linear,
exponential — along with a measured **bias toward earlier instructions**.

**SEQUOR** (arXiv:2605.06353) finds accuracy on a *single* constraint declining as
a conversation lengthens, and multi-constraint accuracy declining several times
faster.

**LongGenBench** and **LIFBench** (arXiv:2411.07037) show prompt-adherence loss in
long contexts generally.

The design consequence: a rule's position and the total instruction count both
affect whether it is followed. A 4,000-word constitution is not four times the
governance of a 1,000-word one. It may be less.

## 2. Vendor guidance changed direction

Anthropic reported removing **over 80%** of Claude Code's system prompt for its
Claude 5 generation models with no measurable loss on internal coding evaluations,
and described six shifts: rules to judgment, examples to interface design, upfront
context to progressive disclosure, repetition to simple tool descriptions, manual
to automatic memory, and simple specs to rich references.

The same source identifies **overconstraining** across system prompt, rules file,
and skills simultaneously, with conflicting instructions inside a single request —
the example given being "leave documentation as appropriate" against "DO NOT add
comments."

The Opus 5 prompting guide instructs removing explicit verification instructions,
because they cause over-verification and waste tokens without quality gain. The
Fable 5 guide adds that skills written for earlier models are often too
prescriptive and can degrade output.

Claude Code's own documentation states the operational form: a bloated rules file
causes Claude to ignore your actual instructions, and the test for each line is
whether removing it would cause a mistake.

**Treat this as directional, not audited.** It is Anthropic's figure, on
Anthropic's evaluations, for Anthropic's models, without a per-category breakdown.
It is strong evidence about those models and weak evidence about anything else.

## 3. Advisory versus deterministic

Anthropic's hooks documentation draws the line directly: rules-file instructions
are advisory, hooks are deterministic and guarantee the action happens.

This is the load-bearing insight behind moving tier discipline, single-writer
builds, the anti-sycophancy gate, and the compaction re-anchor out of prose. Those
are predicates, not judgment calls. A predicate checked by a process does not
compete for attention with 200 other instructions.

Critic isolation follows the same logic one level up: a subagent that structurally
cannot see generation history is a stronger guarantee than an instruction asking a
model to ignore what it already knows.

Independent support for external verification comes from multi-agent verification
research — **BoN-MAV** (Lifshitz, McIlraith, Du, arXiv:2502.20379) reports stronger
scaling from multiple verifiers than from self-consistency, including
weak-to-strong generalization.

Note the tension this resolves. Anthropic says remove verification instructions
from prompts; verification research says external verifiers help. Both hold:
verification belongs in the *loop* — tests, stop conditions, hooks, CI — not as
prose nagging in a rules file.

## 4. Compaction determines durability

Content loaded from disk at startup is re-read after compaction and re-injected.
Content that arrived through the conversation is summarized, and summarization is
lossy. Standing policy is among the first things discarded, because it does not
resemble the task in progress.

Corollary with a measurable cost: `@import` resolves at launch, so an imported file
loads in addition to the importing one. A `CLAUDE.md` containing `@AGENTS.md` loads
`AGENTS.md` twice. A symlink does not.

## Measured locally

These numbers come from running the tooling on real repositories, not from any
published source. They are reproducible with the shipped scripts.

| Measurement | Value | How |
|---|---|---|
| Resident before/after on one repo | 9,393 words across two files → 1,396 managed + 866 carried in one | `wc -w`, `verify.sh` |
| v3 rule IDs resident after migration | 45 → 0 | `grep -cE '^\*\*[A-G]-[0-9]+ ·'` |
| Profile sizes | lean 847 words, mixed 1,396 | `verify.sh` |
| Skill budget on one machine | 2,267 skills, ~163,000 tokens vs ~4,000 budget, ~41× over | `skill-budget.sh` |
| Waypoint schema across the estate | `.phase` is an identity string; `.status` is the lifecycle | `jq` across every waypoint |

That last one produced a real defect. An early `tier-guard.sh` matched `.phase`
against `milestone|release|certify`. No waypoint in the estate could ever satisfy
it, so Tier 3 was blocked unconditionally with no reachable unblock path. The rule
was written from an assumption about a schema; measuring the schema refuted it.

## Assumptions, still unmeasured

Stated as assumptions so they can be attacked.

- **Per-model scaffold need.** The table in [Model Profiles](./model-profiles)
  gives starting assumptions for Kimi, MiniMax, GPT, and local models. Only the
  Claude entries rest on published guidance. The rest is reasoning from general
  principles about smaller models and should be replaced by your own task-set
  measurement.
- **The 1,500-word ceiling.** A judgment about where the resident file stops
  earning its cost, not a measured threshold.
- **The migration map.** Where each v3 rule lands is a judgment about equivalent
  coverage, not a verified claim that behavior is preserved.

## What would refute this

A design that cannot be refuted is not a design.

**Fixed task set, run twice.** Ten representative tasks for your repository, run
under the old file and the new structure, per model in the fleet. If pass rate
regresses at lower token cost, specific deleted rules were load-bearing for your
codebase and the reduction was wrong for you.

**Compaction test.** Force a compaction mid-session and check whether the
invariants still hold. If behavior drifts on capability inversion or tier
discipline, the invariant was not actually in the durable channel and the layering
claim fails.

**Hook audit.** If a hook is installed but not referenced in settings, enforcement
is zero while the prose that asked for the same behavior has been deleted. That
window is strictly worse than either state alone. `verify.sh` checks it; check that
`verify.sh` is being run.

**Budget measurement.** If `skill-budget.sh` still reports many multiples over
budget after gating, the plugin split was insufficient and the router must carry
more.

## The strongest case against

Vendor evidence for aggressive reduction is a vendor's own number on its own
evaluations. "Trust the model's judgment" lands differently in a regulated,
contractually-governed codebase than in a hobby project — implicit judgment is
precisely what a reviewer of governed AI does not want.

A verbose constitution that reliably prevents one capability-inversion breach, or
one sycophantic sign-off on a client deliverable, may be cheaper than the tokens it
costs.

The disciplined answer is not "keep the large file" and not "cut it because a blog
post said so." It is: **delete nothing until measurement shows it is safe to
delete**, and treat the migration as evidence-gated. That is why `verify.sh` fails
a repo that declares `lean` without a measured fleet entry, and why the default
profile keeps the scaffold rather than assuming it away.

## Source index

| Claim | Type |
|---|---|
| 80% system prompt reduction, six context-engineering shifts | vendor, primary |
| Remove verification instructions on Opus 5; Fable 5 over-prescription | vendor, primary |
| Bloated rules files cause instructions to be ignored | vendor, primary |
| Hooks deterministic, rules advisory | vendor, primary |
| Project-root file re-read after compaction; imports load at launch | vendor, primary |
| Claude Code does not natively read `AGENTS.md` | vendor issue tracker, open |
| `skillListingBudgetFraction`, silent description drops, eviction ranking | vendor changelog + issue reports |
| AGENTS.md as an open convention across 20+ tools | open standard, widely adopted |
| Instruction-following degradation with density and turn count | peer-reviewed (IFScale, SEQUOR, LIFBench) |
| Multi-verifier scaling | peer-reviewed (BoN-MAV) |
| Resident word counts, budget ratios, waypoint schema | measured locally, reproducible |
| Per-model scaffold need outside Claude | assumption, unmeasured |

Vendor documentation moves. Where a number here disagrees with current
documentation, the documentation wins — and the local measurements are reproducible
with the shipped scripts rather than taken on faith.
