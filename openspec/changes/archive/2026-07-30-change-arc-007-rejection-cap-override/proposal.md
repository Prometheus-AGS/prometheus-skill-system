# Make the sycophancy-screen cap user-overridable

**Change:** `change-arc-007-rejection-cap-override`
**Phase:** adversarial-review-for-creation
**Goal:** extends 5

## Why

check-findings-sycophancy.sh:48 hardcodes 2 and is not env-overridable. The code decides for the user.

## What

See `.kbd-orchestrator/phases/adversarial-review-for-creation/plan.md` for full rationale,
acceptance criteria, and the adversarial review record.
