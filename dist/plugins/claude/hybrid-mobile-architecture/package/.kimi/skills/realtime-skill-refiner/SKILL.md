---
name: realtime-skill-refiner
description: Reactively detect, triage, refine, verify, and ship a fix for a bug in an installed skill package, driven by a real observed failure rather than a proactive review. Use when a skill produces wrong answers, fires on the wrong prompts, fails to fire at all, or is flagged by a log monitor or sycophancy check. Triggers on skill bug, skill failing, fix skill, patch skill, refine skill, realtime correction, skill regression, wrong skill fired, skill did not fire, sycophancy, bug ticket.
---
<!-- TJ-ARCH-MOB-001 compliant -->

# Realtime Skill Refiner

Proactive refinement happens when an author reviews a skill they just wrote.
This is the reactive counterpart: the same refinement algorithm, triggered by a
failure that already happened in production.

Run with `scripts/refiner-loop.mjs --skill <name>`.

## The loop

```
Detect  →  Triage  →  Refine  →  Verify  →  Ship
            HALT
```

**Triage halts for human approval.** The loop stops there and will not continue
without an explicit decision. A producer does not grade its own work: the same
agent that will write the fix must not also be the one that decides the bug is
real and the diagnosis correct. Resume with `--approve <ticket>`, or close it
with `--reject <ticket>`.

## Stage 1 — Detect

Three signal sources, each recording where the evidence came from:

| Source | Signal |
|---|---|
| Log monitor | A service log line matches a known failure pattern |
| Sycophancy check | A skill's output is flagged as agreeable rather than correct |
| Operator | A bug is reported by hand against a named skill |

Detection records a ticket: which package, which skill, the source, the verbatim
evidence, and the timestamp. It does not diagnose. A ticket with a paraphrased
symptom and no raw evidence cannot be triaged later.

## Stage 2 — Triage (halts)

Gather evidence and decide scope. Produce, for human review:

- The exact failing prompt or input, verbatim.
- Which skill fired, and which should have.
- Whether the fault is the description (activation), the body (guidance), or the
  frontmatter contract (validation).
- The blast radius: which other skills share the trigger terms being changed.

Then stop. The output of triage is a proposal, not a change.

Widening one skill's description to catch a missed prompt routinely steals
prompts from a neighbor. Naming the neighbors here is what prevents fixing one
activation bug by creating another.

## Stage 3 — Refine

Only after approval. Apply the smallest change that addresses the approved
diagnosis:

- **Activation fault** → adjust the description's trigger terms. Keep the
  description within its length cap.
- **Guidance fault** → correct the body. Do not expand scope while in there.
- **Contract fault** → fix the frontmatter so it satisfies the validator.

One ticket, one skill, one focused diff. A refinement touching several skills
means triage under-scoped the problem; go back rather than proceeding.

## Stage 4 — Verify

Run the gates, then re-execute the failure if a reproduction was recorded:

```bash
node scripts/refiner-loop.mjs --verify TICKET
```

Verify runs four gates — `check-skill-contracts.mjs`,
`generate-skill-metadata.mjs`, `generate-skill-evals.mjs`, and
`sync-harness-skills.mjs --check` — then reports whether skill/eval changes are
present (informational, **not** a gate: both outcomes pass), then the replay.

**Replay.** A ticket opened with `--replay-argv '["node","path/to/regression.mjs"]'` records an argument array that
*reproduced* the failure. After a real fix that command must **succeed**:

- exit 0 → the failure no longer reproduces → the ticket verifies
- non-zero → the failure is still there → Verify fails and the ticket returns
  to `approved`, no matter how green the gates are

The replay runs directly in the project root, without shell parsing. A ticket
still requests executable code, so inspect its command and arguments before
verification. Old tickets containing shell replay strings need a reviewed
conversion to an argument array; the tool refuses to evaluate them.

A transcript-only ticket may be recorded and triaged, but cannot become
verified or shipped without a runnable regression. Missing replay returns a
nonzero outcome and retains an unverified ticket. Metadata gates alone do not
prove the reported failure is fixed.

If the same fix fails verification twice, stop and report the discrepancy
rather than trying a third variation.

> **On eval cases.** An earlier version of this skill said Verify should "add
> an eval case covering it". It cannot, and neither can you by hand:
> `evals/builder-skills.jsonl` is **generated** — `generate-skill-evals.mjs`
> templates exactly three cases per skill from that skill's own description, so
> a hand-added line is erased on the next regeneration. The case shape
> (`{prompt, expectedSkills}`) asserts skill *activation*, which most refiner
> tickets are not about, and no runner executes cases at all —
> `check-skill-contracts.mjs` only counts them. `--replay-argv` is the regression
> guard this loop actually has.

## Stage 5 — Ship

Commit the skill change together with its regenerated artifacts. Mark the ticket
shipped, keeping the evidence and the replay command attached. Re-register the
package so the harness picks up the change.

## Anti-patterns

- Skipping the Triage halt because the bug "is obvious".
- Editing a generated mirror instead of the canonical skill.
- Broadening a description without checking which neighbors lose prompts.
- Shipping a ticket whose replay was never recorded when the evidence could
  have been reduced to a runnable command.
- Closing a ticket with no record of the evidence that opened it.
