# Add skill and agent review packet modes

**Change:** `change-arc-003-creation-review-packets`
**Phase:** adversarial-review-for-creation
**Goal:** 3

## Why

build-review-packet.sh supports diff|artifact only. A generated skill tree and a Cargo workspace are neither, and a workspace exceeds any judge's context.

## What

See `.kbd-orchestrator/phases/adversarial-review-for-creation/plan.md` for full rationale,
acceptance criteria, and the adversarial review record.
