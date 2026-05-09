---
id: SP-018
title: End-to-end pipeline smoke test (Layer 1 → Layer 4)
status: planned
priority: P1
estimated_effort: 2-3d
agent_role: hooks-engineer
depends_on: [SP-006, SP-012]
unblocks: []
related: [XC-004]
created_from_conversation_turn: 3-4
---

# SP-018 — End-to-end pipeline smoke test

## Problem

The 4-layer pipeline (ZeeSpec → PMPO → OpenSpec → forge-rs) is documented and (after SP-012) enforced. But there is no automated smoke test that exercises the full pipeline end-to-end on a synthetic case. Each layer's tests cover that layer; nothing tests the integration.

## Evidence

Look for any test that:
1. Submits a synthetic broad-change prompt.
2. Requires ZeeSpec, PMPO, OpenSpec artifacts to exist.
3. Validates the eventual code-edit happens.
4. Validates the reflector runs and the reflection passes sycophancy correction.

It doesn't exist.

## Why it matters

The pipeline is only as strong as its weakest link in integration. Each layer working in isolation is insufficient. Without a smoke test, regressions in one layer's contract with the next are caught only when a real session triggers them.

This task is the "is the system actually working?" check.

## Proposed fix

Build a synthetic test harness in `scripts/test-pipeline-e2e.sh` that:

1. Creates a temporary scratch project (under `/tmp/prom-pipeline-test-<ts>/`).
2. Initializes it as a Claude Code-aware project (minimal `.claude/`, `.kbd-orchestrator/`, etc.).
3. Submits a synthetic broad-change prompt (via either a scripted Claude Code invocation or by faking the hook events).
4. Asserts the artifacts at each layer:
   - ZeeSpec entry exists.
   - PMPO `.kbd-orchestrator/phases/<phase>/assessment.md` exists.
   - OpenSpec `openspec/changes/<change-id>/proposal.md` exists.
   - The actual code edit was made.
   - The reflector ran and the reflection passed sycophancy-correction.
5. Cleans up the scratch project.

The test runs in CI on every PR to the skill-pack and prometheus-knowledge.

## Trade-offs and risks

- **Risk: faking Claude Code events is fragile.** Real Claude Code is preferred. Mitigation: if real Claude Code isn't available in CI, use a scripted invocation that hits the same hook code paths via the bash-level entry points. The test asserts each layer's artifact appears, not the agent's internal state.
- **Cost: each test run takes ~30-60s.** Acceptable for a CI smoke test.
- **Risk: synthetic prompt is contrived; real prompts hit edge cases the test doesn't.** Mitigation: this is a smoke test, not full coverage. Real-session quality is the integration test.

## Acceptance criteria

- [ ] `scripts/test-pipeline-e2e.sh` runs end-to-end and exits 0 on a healthy pipeline.
- [ ] It runs in CI on PR to either skill-pack or prometheus-knowledge.
- [ ] Asserts all 4 layer artifacts are produced.
- [ ] Asserts the reflector ran and passed sycophancy-correction.
- [ ] Includes a `--negative` mode that intentionally breaks one layer (e.g. removes the OpenSpec scaffolding) and verifies the test catches it.

## Implementation steps

1. Write the scratch-project initializer.
2. Write the prompt submission step (start with bash-level fake; upgrade later if Claude Code can be invoked headlessly).
3. Write the artifact-assertion checks.
4. Write the cleanup.
5. Add CI invocation.
6. Test the negative mode.

## Dependencies

SP-006 (need hook log for assertions about whether hooks fired), SP-012 (pipeline enforcement; the smoke test exercises it).

## Open questions

- Can Claude Code be invoked headlessly in CI? If yes, the test is much higher-fidelity. If no, the bash-level fake is the path.
- Should the smoke test produce a coverage report (which scripts ran, which were skipped)? Useful for SP-007 verification too.
