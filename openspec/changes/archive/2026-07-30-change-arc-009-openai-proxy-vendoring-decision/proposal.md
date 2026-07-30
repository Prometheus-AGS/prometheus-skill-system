# Decide whether to vendor openai-proxy

**Change:** `change-arc-009-openai-proxy-vendoring-decision`
**Phase:** adversarial-review-for-creation
**Goal:** new

## Why

openai-proxy is a referenced sibling, not a submodule. It is what kbd-judge resolves to; its absence silently degrades every review to harness-native.

## What

See `.kbd-orchestrator/phases/adversarial-review-for-creation/plan.md` for full rationale,
acceptance criteria, and the adversarial review record.
