---
id: change-int-006-cowork-submodule
title: cowork-skills git submodule registration
phase: cowork-integration
priority: P1
effort: S
wave: 5
agent: general-purpose
status: done
gap_id: G-05-cowork
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - tools/cowork-skills (new submodule gitlink)
  - .gitmodules (url correction: SSH → HTTPS)
  - docs/SUBMODULES.md (already updated in change-int-001)
---

# change-int-006 — cowork-skills git submodule registration

## Context

The .gitmodules file already contained a tools/cowork-skills entry (written
directly in a prior change) but the submodule was never properly registered
in the git index via `git submodule add`. This change runs the registration,
which clones the repo at the correct HEAD and creates the gitlink commit entry.

The install_cowork() function in scripts/install-binaries.sh was already added
(change-cowork-012 / install-binaries.sh section 8) and correctly expects the
binary at tools/cowork-skills/cli/target/release/cowork.

## Strategy

1. Run git submodule add (done — submodule now at 53e6b31, v0.1.5)
2. Verify docs/SUBMODULES.md already has the cowork-skills entry
3. Commit the submodule gitlink and updated .gitmodules
4. Update KBD orchestrator

## Scope

1. git submodule add completed (tools/cowork-skills at 53e6b31)
2. Verify docs/SUBMODULES.md cowork-skills entry
3. Commit
4. Update KBD orchestrator
