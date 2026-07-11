---
type: Reference
id: gitignore-audit-for-ci-cross-model-qa-hardening-phase
title: .gitignore Audit for CI Cross-Model QA Hardening Phase
description: "The `.gitignore` audit for `phase-ci-cross-model-qa-and-hardening` was completed, committed, and merged to `main`."
tags:
- gitignore
- ci
- cross-model-qa
- hardening
- repository-hygiene
- runtime-artifacts
links:
- ci-cross-model-qa-and-hardening-phase-closeout
sources:
- stdin
timestamp: 2026-07-03T20:34:14.056290+00:00
created_at: 2026-07-03T20:34:14.056290+00:00
updated_at: 2026-07-03T20:34:14.056290+00:00
revision: 0
---

## Session Outcome

The `.gitignore` audit for `phase-ci-cross-model-qa-and-hardening` was completed, committed, and merged to `main`.

- **PR:** #29
- **Merged HEAD:** `2b7faf8`
- **Working tree status after session:** no tracked modifications
- **Context:** follow-on hardening cleanup for the phase tracked in [CI Cross-Model QA and Hardening Phase Closeout](/ci-cross-model-qa-and-hardening-phase-closeout.md).

## Ignore Gaps Fixed

Previously untracked runtime/generated artifacts were added to `.gitignore` so they cannot be accidentally committed:

| Pattern / path | Purpose | Status |
|---|---|---|
| `/data/memory.db/` | `surreal-memory` / forge embedded RocksDB default at `./data/memory.db` | ignored |
| `site/.docusaurus/` | generated Docusaurus cache; build output and `node_modules` were already ignored | ignored |
| `*.log.lock` | hook-log advisory lock files | ignored |
| `*.jsonl.lock` | JSONL hook-log advisory lock files | ignored |

## Verification Results

- Removed approximately **62 runtime/generated files** from the untracked set: `260 → 198`.
- Final sweep confirmed **zero untracked runtime junk** matching:
  - `.db`
  - `.docusaurus`
  - `/target/`
  - `node_modules`
  - `.log`
  - `.lock`
- Verified that no tracked file is newly ignored by the added rules.
- Confirmed `Cargo.lock` remains committable, matching the existing policy represented by six already-tracked substrate/tools lockfiles.

## Deliberate Non-Commit Decision

The session intentionally did **not** run `git add -A` or blindly commit all untracked files. The remaining ~198 untracked files were pre-existing prior-session or other-tool work visible as `??` before this audit began, including:

- `substrate/sovereign-client/Cargo.lock` and crate source
- `substrate/sovereign-sync/Cargo.lock` and crate source
- documentation files
- OpenSpec specs
- other phases' KBD records
- `site/` documentation source
- `skills/learn/sync-*`

Rationale:

- The files span unrelated concerns and were not created or reviewed during this session.
- Blindly committing them would violate the rule against sweeping in large unreviewed changes.
- Before the ignore audit, a blind commit risked adding a **312 MB site tree** and a **runtime database** to `main`.

## Follow-Up Policy

If the remaining pre-existing untracked work should be committed, it should be split into focused, reviewable PRs by group, such as:

- sovereign sync/client crates
- documentation site source
- Feynman docs
- credibility/OpenSpec specs
- KBD records from other phases

No further action is required for the `.gitignore` audit itself.

# Citations

1. stdin