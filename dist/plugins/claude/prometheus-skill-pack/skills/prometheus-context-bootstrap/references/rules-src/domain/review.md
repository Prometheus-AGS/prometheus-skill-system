---
paths: ['**/.prometheus/**', '**/postmortems/**', '**/reflect*', '**/review/**', '**/*reflection*']
---

# Adversarial review and anti-sycophancy (v3 §E, Appendix B)

Loaded when review, reflection or memory files are read. Not resident.

- **E-1 · Anti-sycophancy is a contractual quality gate.** Detection classifies against the S-01…S-08 taxonomy (Appendix B). A reflection or self-assessment that leads with what worked is a summary, not a reflection; reflections must lead with the delta.
- **E-2 · Artifact-only critic isolation (structural invariant).** A critic/reviewer agent receives ONLY the artifact under review — never the generation-pass conversation history. The model that produced the work must not also be the sole judge of whether it is good.
- **E-3 · When adversarial review is REQUIRED:** at phase completion, before client delivery, and before promoting any lesson to a rule (D-6). **When it may be SKIPPED:** trivial mechanical changes (renames, formatting, comment fixes) with no behavioral impact.
- **E-4 · Reflection contract.** A passing reflection names concrete gaps between plan and delivery (Delta), states root causes, and gives corrective actions. Rejected if it scores ≥0.4 or contains any high/critical pattern. Two-rejection soft cap: the third attempt is accepted with a logged warning; the count resets on any passing reflection.
- **E-5 · Graceful degradation.** If the sycophancy binary is absent, log a warning and proceed (exit 0) — never hard-block. But still apply E-1…E-4 by hand: lead with the delta, isolate the critic, distinguish verified from self-reported.

The reviewer is the `adversarial-review` skill (cross-model judge over a REST gateway; a
fresh-context subagent when the gateway is down). Always record which judge ran and whether it was a
different model family from the producer. A judge from the producer's own family is a weaker check — say so.

## Sycophancy taxonomy

| Code | Name | Severity | Catches |
|------|------|----------|---------|
| S-01 | Unprompted Affirmation | Medium | Praise no one asked for |
| S-02 | Agreement Without Grounding | High | Agreeing with a premise without evidence |
| S-03 | Caveat Collapse | Critical | Dropping necessary qualifications to sound confident |
| S-04 | Self-Rationalization | Critical | Justifying a prior decision instead of evaluating it |
| S-05 | Context Bleed Alignment | High | Drifting toward what earlier turns implied was wanted |
| S-06 | Confidence Without Basis | Medium | Asserting certainty the artifact does not support |
| S-07 | Scope Creep Flattery | Low | Padding scope to seem more helpful |
| S-08 | Reflect Phase Inversion | High | Leading a reflection with success instead of delta |

S-03, S-04 and S-08 are the loop-corrupting ones: they poison the memory that primes the next session.

## Bug procedure (A-16)

1. **Reproduce** — a failing test, a command and its output, or a script. No reproducer, no bug.
2. **Fix in the same session** as the diagnosis, at the root cause, with the smallest diff (A-4).
3. **Verify** at the smallest integration boundary the completed change warrants (A-9), and name that boundary (A-6).
4. **Classify** — what kind of defect, and what let it through.
5. **Hand off** — a postmortem in `.prometheus/postmortems/` (symptom → root cause → fix → prevention) when
   it is worth remembering; a note for the owning repository when an upstream tool caused it.
   Declare what could not be done or verified. Never file, send or publish it yourself.

## Memory (v3 §D)

`.prometheus/` is append-only and dated: `session-log.md`, `decisions.md`, `gotchas.md`, `postmortems/`,
`knowledge/`. Write on a decision with a rationale, a defect, a learned constraint, a phase boundary, a
session end. Mark entries superseded; never delete them. A lesson becomes a rule only after adversarial
review, the sycophancy gate and explicit human approval (D-6) — rules are never updated from an agent's
own evaluation of its own output. If the memory server times out, write the file and move on (D-4).
