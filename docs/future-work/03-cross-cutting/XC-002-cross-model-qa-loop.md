---
id: XC-002
title: Cross-model QA loop (Codex/GPT review)
status: ready
priority: P1
estimated_effort: 2d
agent_role: hooks-engineer
depends_on: []
unblocks: []
related: [SP-013]
created_from_conversation_turn: 1-2
---

# XC-002 — Cross-model QA loop

## Problem

The PMPO Reflect phase has a critic agent. With SP-013 in place, the critic uses sycophancy-correction. But the critic is still an LLM — and likely the same model family as the generator. **Same-family critics share blind spots.** A pattern that the generator missed because of training-data gaps is one the critic likely also misses for the same reasons.

A second-opinion pass from a *different* model family catches a class of errors that intra-family review structurally cannot.

## Evidence

Reason about it: if Claude generates code with a subtle pattern that doesn't appear in Claude's training distribution as broken, Claude reviewing Claude won't catch it. GPT-class models or Codex have different training distributions; their blind spots don't exactly overlap.

This isn't "GPT is better." It's "diverse models catch diverse errors."

## Why it matters

For high-stakes artifacts (production code, architecture decisions, security-relevant content), a same-family critic is insufficient. Cross-model review is the structural countermeasure.

## Proposed fix

Add an optional `cross-model-review` Stop-chain script (or SubagentStop hook) that:

1. **Triggers selectively.** Not every artifact needs cross-model review. Configurable filter: "if the artifact is in `src/` and the change touched > 50 LOC" or "if the change touched security-relevant paths" or "if the user explicitly tagged the request `--cross-review`."
2. **Picks a different model family.** Calls Codex (GPT-class) via litellm. If GPT-class is unavailable, falls back to Gemini or another distinct family.
3. **Asks the structured prompt:** "Review this artifact. Identify: factual errors, structural issues, missing edge cases, security concerns. For each, indicate severity. Do not rewrite — only identify."
4. **Surfaces the response** in a sidebar or comment, not as enforcement. Author decides what to act on.
5. **Logs the review** to `~/.prometheus/cross-review/<session>/<artifact>.md`.

The output is advisory, not blocking. The structural value comes from a second-pair-of-eyes-with-different-eyes, not from rule enforcement.

## Trade-offs and risks

- **Cost: extra LLM call per qualifying artifact.** Bounded by selective triggering.
- **Risk: developers ignore the review.** Mitigation: high-value reviews are surfaced prominently; low-value ones are quietly logged.
- **Risk: cross-family review identifies false issues.** Acceptable — it's advisory. The signal-to-noise ratio improves with calibration.
- **Risk: requires an additional API key/cost line item.** Plan accordingly.

## Acceptance criteria

- [ ] Script `shared/scripts/cross-model-review.sh` exists.
- [ ] Configurable trigger filters.
- [ ] Calls a non-Claude model via litellm.
- [ ] Surfaces output in a structured form (markdown).
- [ ] Logs to `~/.prometheus/cross-review/`.
- [ ] Test: a synthetic artifact passes through and produces output.
- [ ] Documentation: when to use, when not to, how to interpret advisory feedback.

## Implementation steps

1. Identify litellm config for the secondary model.
2. Write the script + configurable trigger.
3. Define the review prompt.
4. Implement the trigger evaluation.
5. Test on a real artifact.
6. Document.

## Dependencies

None hard. Synergy with SP-013 (sycophancy correction) — both are structural reflection-quality measures.

## Open questions

- Should the cross-model review be a SubagentStop matcher, or a separate Stop-chain script? The latter is simpler; the former is more tightly scoped to specific subagents. Default: Stop-chain.
- Should there be more than one cross-family reviewer (e.g. Codex *and* Gemini for highest-stakes)? Yes for high-stakes; default to one for cost.
