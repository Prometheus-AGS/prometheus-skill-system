# Execution — phase-first-user-onboarding

**Date:** 2026-06-30
**Backend:** native-tool (Claude Code direct edit — documentation-only changes)
**QA gate:** Skipped for both changes (documentation-only, fewer than 3 files each)

## Changes

### change-onboard-001-claude-code-prereq — DONE

Added a blockquote note under `## Step 4 — Open Claude Code` in `docs/QUICK_START.md`:

```
> **Don't have Claude Code yet?** Sign up at https://claude.ai/code — a free plan
> is available for getting started.
```

This prevents a dead end for collaborators who encounter "claude: command not found"
at Step 4 with no guidance on how to acquire Claude Code.

### change-onboard-002-linux-systemd-note — DONE

Extended the "MCP services not running" troubleshooting paragraph in
`docs/QUICK_START.md` to include Linux/systemd commands alongside the existing
macOS/launchd instructions. The macOS text is preserved unchanged.

## Outcome

`docs/QUICK_START.md` now covers:
- The Claude Code account requirement (previously silent dead end)
- macOS launchd AND Linux systemd paths for MCP service management

## What these changes do not close

G1–G4 remain entirely gated on human action. These changes reduce friction for
a collaborator the maintainer finds; they do not find one.
