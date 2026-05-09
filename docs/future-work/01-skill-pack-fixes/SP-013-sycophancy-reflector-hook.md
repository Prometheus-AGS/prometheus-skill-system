---
id: SP-013
title: Sycophancy correction in SubagentStop(reflector) hook
status: ready
priority: P0
estimated_effort: 1-2d
agent_role: hooks-engineer
depends_on: []
unblocks: []
related: [SP-012, SP-018]
created_from_conversation_turn: 3-4
---

# SP-013 — Sycophancy correction in SubagentStop(reflector) hook

**This is the highest-leverage fix in the entire pack.** The work is small (1-2 days), the cost is near-zero, and the structural effect is that the PMPO Reflect phase becomes resistant to a class of failure mode that currently slips through.

## Problem

The `sycophancy-correction` skill exists in the pack as a submodule. It exposes detection at strictness levels (loose, standard, strict, adversarial) and 8 named patterns (S-01 through S-08). It is currently invoked **manually** when a user remembers to run it.

The PMPO Reflect phase produces an artifact ("the reflection") that is supposed to honestly evaluate the just-completed work. In practice, the reflector agent has access to the entire generation-pass conversation history, which biases it toward agreeing with the conclusions that were already reached. **The critic agent, by reading the generation history, becomes a participant in the same reasoning rather than an independent reviewer.**

The fix is structural: invoke `sycophancy-correction` automatically at the SubagentStop boundary specifically for the reflector subagent, with the artifact-only (no generation history) input pattern.

## Evidence

1. Read `skills/sycophancy-correction/SKILL.md` and the `agents/` configurations.
2. Read `shared/scripts/forge-reflect-on-stop.sh` — the current Stop-chain script for the reflector.
3. Note: nothing in that script invokes sycophancy-correction. The reflection is accepted as-is.
4. Sample reflections from past sessions (if any are persisted). Look for patterns S-03 (completion without trade-offs) and S-08 (premature closure). These will appear.

## Why it matters

The Reflect phase is supposed to be the system's self-criticism step. If the critic is biased toward agreement, the system can't catch its own failures. Every other quality gate downstream (OpenSpec review, broad-change recording, etc.) inherits the bias.

This is the *single-step intervention* that produces the largest improvement in self-evaluation quality. It costs ~1 day to wire correctly. It has been done manually in this very session (Travis ran sycophancy-correction at adversarial strictness against my turn-2 response, surfaced a critical S-03, and the conversation improved as a result). The point is to make this automatic, not session-specific.

## Proposed fix

A `SubagentStop` hook with matcher specifically targeting the reflector subagent. The hook:

1. **Reads only the artifact**, not the generation conversation. The artifact is whatever the reflector subagent just produced (typically a reflection.md file or equivalent).
2. **Calls sycophancy-correction** at `strict` strictness by default (configurable per-environment to `adversarial` for prod).
3. **If the score is below threshold** (e.g. >0.3, configurable), or if any **critical pattern** is flagged (severity in `[high, critical]`), rejects the artifact and returns to the user with a message: "Reflection rejected by sycophancy-correction. Patterns detected: [...]. Re-run /kbd-reflect with the correction guidance below."
4. **Logs the result** to `~/.prometheus/hooks.log` (per SP-006).

The critical structural property: **the critic must NEVER see the generation-pass conversation history.** Only the artifact. This is the entire reason for doing it at the SubagentStop boundary rather than inside the reflector subagent itself — the boundary is the only place where the input can be controlled to artifact-only.

## Trade-offs and risks

- **Risk: false positives reject good reflections.** Adversarial strictness is aggressive and will sometimes flag genuine completions as S-03. Mitigation: default strictness is `strict`, not `adversarial`. Adversarial is opt-in for prod environments where the cost of missing a sycophancy is high.
- **Risk: rejection loop.** A user re-runs the reflector after rejection and gets rejected again. Mitigation: rejection includes specific guidance (the patterns flagged plus rewrite suggestions from the correction skill). After two consecutive rejections, the third allows through with a logged warning instead of a hard block.
- **Risk: latency.** Each reflection now incurs a sycophancy-correction call (~2-5s). Acceptable — it's at session-end timing.
- **Risk: the critic skill's own biases.** Sycophancy-correction is itself an LLM call; its outputs are not infallible. Mitigation: the patterns are well-defined; the skill's score is auditable.

## Acceptance criteria

- [ ] `SubagentStop` hook with matcher for the reflector subagent invokes sycophancy-correction on the artifact only.
- [ ] Generation-pass conversation history is verifiably *not* passed to the critic (review the data flow in the hook script).
- [ ] Default strictness is `strict`. Configurable via `PROMETHEUS_REFLECT_STRICTNESS=loose|standard|strict|adversarial`.
- [ ] Below-threshold or critical-pattern rejections produce actionable feedback.
- [ ] After two consecutive rejections, the third allows through with logged warning.
- [ ] All decisions logged via SP-006.
- [ ] Test: a synthetic sycophantic reflection is rejected at strict strictness; a genuine balanced reflection passes.

## Implementation steps

1. Locate the reflector subagent's matcher in the existing SubagentStop config.
2. Write `shared/scripts/sycophancy-check-reflection.sh` that:
   - Reads the artifact path from the hook event.
   - Reads the artifact (and *only* the artifact).
   - Invokes sycophancy-correction with the artifact text and configured strictness.
   - Parses the result, decides accept/reject.
   - Logs.
   - Returns the appropriate exit code (zero = accept, non-zero = reject with stderr message).
3. Wire the script as the SubagentStop matcher action for the reflector.
4. Add the consecutive-rejection counter (state file at `.prometheus/reflect-rejections.txt` per session).
5. Test with synthetic sycophantic and balanced artifacts.
6. Document in skill-pack `CLAUDE.md` so users know the gate exists.

## Dependencies

None functional. SP-006 (hook log) strongly recommended for audit.

## Open questions

- Is there exactly one reflector subagent matcher, or multiple? Verify against current `hooks.json`. If multiple, gate all of them.
- Should the rejection feedback include suggested rewrites from sycophancy-correction's `correct_sycophancy` mode? Yes — strongly recommended. The user gets actionable guidance rather than just "you failed."
- Should this also apply to non-reflector subagents that produce evaluations or summaries? Possibly, but expand cautiously. Start with reflector only; observe; expand to other evaluating subagents if useful.
