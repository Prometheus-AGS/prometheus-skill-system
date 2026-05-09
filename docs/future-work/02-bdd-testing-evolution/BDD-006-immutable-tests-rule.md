---
id: BDD-006
title: Immutable-tests CLAUDE.md rule
status: ready
priority: P0
estimated_effort: 0.5d
agent_role: docs-writer
depends_on: []
unblocks: []
related: [BDD-005, BDD-007]
created_from_conversation_turn: 5-6
---

# BDD-006 — Immutable-tests CLAUDE.md rule

This task is the **reframing of the original "auto-update tests when code changes" ask as a category error.** Read this entire document — including the trade-offs section — before objecting to its scope. The proposed work is intentionally narrow.

## Problem

The original ask was: "tests should be automatically updated by code-generation agents using these skills without having to remind the AI tool."

Implementing this literally creates a tautology. If an agent edits production code in operation A and then edits the tests for that code in operation B to make them pass, the tests are no longer a regression check — they are a derivative of the code. The whole point of having a test suite is that the tests are an *independent specification of intent*, written by a human (or a human-supervised process) such that when the implementation drifts from intent, the test fails. An agent that synchronizes both halves loses that property.

Worse: the failure mode is silent. Tests pass; CI is green; the team has high confidence in code that no longer matches its original intent. This is exactly the kind of trust-erosion that destroys team velocity over months.

## Evidence

The category error is structural and can be reasoned about without inspection. To make it concrete, consider:

1. Agent receives task: "remove the buyer-deletion confirmation dialog."
2. Agent edits `BuyerDeleteButton.tsx` to remove the dialog.
3. Test `cancel-buyer-deletion` fails because no dialog appears.
4. **If the agent is allowed to "auto-update tests when code changes":** it edits the step `When I click cancel in the dialog` to `When I click cancel in the row`. Tests pass.
5. The team's contract — that buyer deletion requires confirmation — has been silently changed. There is no record of the decision.

This is the failure mode. The literal implementation of "auto-update tests" produces it.

## Why it matters

Building this rule is the structural counterweight to the impulse to automate test maintenance. Agents will continue to want to update tests — that's not the problem. The problem is the *operation that combines code edit and test edit in one undocumented step*. The rule splits these.

P0 because BDD-005 and BDD-007 only work if this rule is in force. Without it, agents who hit a testid drift error from BDD-005 will respond by rewriting the step to match new code, defeating BDD-005's purpose entirely.

## Proposed fix

Add a rule to the SSR `CLAUDE.md` (and to the canonical skill-pack `CLAUDE.md` per SP-001) that reads:

> **Tests are an independent specification of intent and code-generation agents may not edit them as part of code changes.**
>
> When you (the agent) modify production code:
>
> - You **may not** edit `tests/steps/*.steps.ts`, `tests/support/*.ts`, or `tests/features/*.feature` to make existing tests pass.
> - You **must** add or update `data-testid` attributes such that step definitions still resolve their selectors. If a selector no longer makes sense (e.g. the dialog you removed), surface it as a question to the user — do not rewrite the step.
> - You **may** add new `.feature` files or scenarios to `tests/features/drafts/` (per BDD-007) describing behavior you've added or changed. Drafts are reviewed and promoted by humans; they do not run in the standard suite.
> - You **may** add new step definitions in `tests/steps/` if the new draft scenarios require them, but only as the natural counterpart to a draft scenario, never to silence an existing failing scenario.
>
> If a test that exists today no longer reflects desired behavior, that is a **deliberate change in intent** that requires a separate, human-reviewed pull request to modify the test. Code changes that incidentally invalidate existing tests must surface that fact to the user, not paper over it.

The rule is enforced both by:
- **CLAUDE.md prose** (probabilistic compliance via prompt).
- **Optional PreToolUse hook** that blocks `Edit`/`Write`/`MultiEdit` operations targeting `tests/steps/*.steps.ts` and `tests/features/*.feature` (excluding `tests/features/drafts/`) when the originating session is also modifying source code in `src/`. This is a deterministic backstop. The hook can warn (dev) or block (prod).

## Trade-offs and risks

- **Risk: the rule prevents legitimate test maintenance.** It does not. Test maintenance is a separate, human-initiated PR. The rule blocks *code-edit-and-test-edit-together* by an agent. Maintenance happens when the *human* decides intent changed.
- **Risk: developers find it annoying when reviewing agent PRs.** Mitigation: when the agent surfaces "this code change invalidates scenario X" as a question, it's faster to triage than discovering it later. Net velocity improves.
- **Risk: the optional PreToolUse hook blocks something legitimate.** Mitigation: the hook is opt-in initially; CLAUDE.md prose is the canonical enforcement. The hook is for projects that have observed enough drift to want deterministic enforcement.
- **Risk: the user disagrees with this framing entirely.** This is the most important consideration. The framing rests on the claim that tests-as-independent-spec is preserved by this rule. If you (the reader) believe agents can both edit code and edit tests safely, the rule's foundation is wrong. The conversation that produced this pack discussed this; the final position is that automated test maintenance, when paired with automated code generation, eliminates the regression-detection function. If you have a counter-argument, raise it before implementing.

## Acceptance criteria

- [ ] Rule text added to SSR `CLAUDE.md` (in the BDD section).
- [ ] Rule text added to skill-pack canonical `CLAUDE.md` (per SP-001).
- [ ] Documentation cross-references BDD-005, BDD-007 explaining the trio.
- [ ] **Optional**: PreToolUse hook implementation (defer to BDD-007 + future task if not done here).
- [ ] At least one example PR demonstrates the rule in action: an agent edits a component, surfaces "scenario X now fails because I removed the dialog," and the human reviews.

## Implementation steps

1. Draft the rule text. Get human review *before* committing.
2. Place in SSR `CLAUDE.md` and skill-pack `CLAUDE.md`.
3. Add cross-references.
4. Optional: add the PreToolUse hook in `shared/scripts/protect-tests.sh`.

## Dependencies

None.

## Open questions

- Is there ever a case where a code change should retire a test entirely (e.g. removed feature)? Yes — done as a deliberate, human-initiated PR with the test removal in the same commit as the code removal, both clearly described in the PR. Agents do not do this autonomously.
- Should the rule apply to unit tests as well? The conversation focused on BDD/integration tests. Unit tests are a closer-coupled artifact and the case for "auto-update is fine" is stronger there because unit tests typically describe specific code units rather than user-facing behavior. Recommend extending to unit tests with a softer rule: agents may update unit tests but must mark each updated test with a comment `// updated automatically alongside code change in <commit>` to preserve auditability.
