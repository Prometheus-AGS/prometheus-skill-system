# Proposal: change-onboard-001-claude-code-prereq

**Phase:** phase-first-user-onboarding
**Date:** 2026-06-30
**Status:** PENDING

## Summary

Add a one-line note to Step 4 of `docs/QUICK_START.md` informing collaborators
that Claude Code requires an Anthropic account, with a link to sign up.

## Problem

Step 4 instructs the reader to "open Claude Code" but does not mention that
Claude Code requires a subscription or login. A collaborator who does not have
Claude Code encounters a dead end with no guidance — the Quick Start breaks at
the most critical step (actually using the skill pack) without a path forward.

## Change

**File:** `docs/QUICK_START.md`

**Location:** After the code block in `## Step 4 — Open Claude Code`

**Add:**
```markdown
> **Don't have Claude Code yet?** Sign up at https://claude.ai/code — a free
> plan is available for getting started.
```

## Acceptance criteria

- [ ] The note appears under Step 4 in the rendered Quick Start
- [ ] The link points to `https://claude.ai/code`
- [ ] No other content in the file is modified

## QA gate

Documentation-only change. QA gate skipped per execute protocol.
