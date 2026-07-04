---
id: change-push-003-docs-smoke-test
title: Document smooth update flow + smoke test cowork v0.2.0
phase: phase-cowork-push-and-release
priority: P1
effort: S
wave: 3
agent: general-purpose
status: done
gap_id: G-04
verdict: BUILD
scope:
  - prometheus-skill-pack worktree (claude/charming-diffie-309eef branch)
  - skills/process/cowork-management/references/COMMANDS.md
  - smoke test verification (cowork --version, cowork pack status, cowork toolchain status)
---

# change-push-003 — Document smooth update flow + smoke test

## Context

The assessment identified a documentation gap: `cowork pack update` only
re-installs skill symlinks, not the cowork binary itself. Users must know
the two-step full-update sequence. This change adds that documentation and
records a smoke test of the installed v0.2.0 binary.

## Strategy

1. Add `## Updating the Skill Pack` section to
   `skills/process/cowork-management/references/COMMANDS.md`
2. Build cowork v0.2.0 from source (tools/cowork-skills/cli after pointer advance)
   and install to ~/.local/bin/cowork
3. Run smoke tests:
   - `cowork --version` → expect `cowork 0.2.0`
   - `cowork pack status` → expect version table + skill counts
   - `cowork toolchain status` → expect toolchain health table
4. Commit the documentation change
5. Update KBD orchestrator to 3/3

## Scope

1. Add ## Updating the Skill Pack section to COMMANDS.md
2. Build and install cowork v0.2.0 locally
3. Record smoke test results
4. Commit documentation
5. Update KBD orchestrator to 3/3 and phase complete
