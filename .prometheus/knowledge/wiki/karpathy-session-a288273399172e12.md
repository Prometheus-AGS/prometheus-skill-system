---
type: SessionRecord
id: karpathy-session-a288273399172e12
title: Karpathy session a28827339917
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T10:07:03.927168+00:00
created_at: 2026-11-16T10:07:03.927168+00:00
updated_at: 2026-11-16T10:07:03.927168+00:00
revision: 0
---

## Delta

[install-skills-flat.sh](scripts/install-skills-flat.sh) is fixed: `learner-model` and `surface-bridge` are now installed from wherever cargo actually writes them, not from the hardcoded in-repo `target/release/` path.

- **Shared helper:** the lookup now lives in a new file, [scripts/cargo-release-dir.sh](scripts/cargo-release-dir.sh), which `install-skills-flat.sh` sources.
- **Why not `scripts/lib/`:** everything under `scripts/lib/` is hashed into the hook bundle id. Putting the helper there made the four generated hook files stale, so I moved it up one level; both drift checks pass again.
- **Checked:** the script parses under bash 3.2, and the helper resolves both crates to `/Volumes/my-passport/cargo/cargo-target/release` and survives `set -e` when cargo can't answer. I have not run `install-skills-flat.sh` end to end, since it reinstalls skills to every platform.

One piece is still pending. [install-binaries.sh](scripts/install-binaries.sh) carries its own inline copy of the same helper, and I can't edit it while its verification run is still executing. When that run finishes I'll switch it to the shared file, re-check it, and report the installed versions.

Nothing is committed; the three changed files are uncommitted on `main`.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T10:07:01.255599Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- scripts/install-binaries.sh
- scripts/install-skills-flat.sh
- scripts/cargo-release-dir.sh
