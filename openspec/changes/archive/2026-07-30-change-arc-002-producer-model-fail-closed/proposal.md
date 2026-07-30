# Fail closed when KBD_PRODUCER_MODEL is unset

**Change:** `change-arc-002-producer-model-fail-closed`
**Phase:** adversarial-review-for-creation
**Goal:** 4

## Why

A synthesized default would fabricate a verified-distinct result that never happened — the exact failure class this phase exists to eliminate.

## What

See `.kbd-orchestrator/phases/adversarial-review-for-creation/plan.md` for full rationale,
acceptance criteria, and the adversarial review record.
