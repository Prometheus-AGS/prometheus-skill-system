# Plan — phase-first-user-onboarding

**Date:** 2026-06-30
**Planner:** Claude Code (Sonnet 4.6)
**Phase:** phase-first-user-onboarding
**Assessment:** `.kbd-orchestrator/phases/phase-first-user-onboarding/assessment.md`
**Change backend:** OpenSpec (`openspec/` directory present at project root)
**Analyze stage:** Skipped (no external library research required — both changes are documentation edits)
**Evolver bridge:** None

---

## Context

This phase has two engineering-addressable changes and a human-coordination gate
that no amount of code can close. The two changes patch `docs/QUICK_START.md` to
prevent two known dead-ends that a collaborator would hit before the maintainer
is even involved.

G1–G4 (finding a collaborator, completing the Quick Start, completing the Feynman
loop, capturing outcomes) remain gated on human action. These changes make G2 more
survivable; they do not cause G2 to happen.

**Important context from kbd-analyze question answered this session:**
The production readiness score ceiling achievable through engineering alone is ~95%,
not 100%. The only additional engineering action that moves the score is a two-node
sovereign-sync integration test in CI (adds ~2-3 points). That change is not part
of this phase. 100% as a static certified claim is structurally unreachable — see
`docs/production-readiness-report.md` → "Why 100% is conceptually unreachable."

---

## Changes

### change-onboard-001: Add Claude Code prerequisite note to QUICK_START.md

**Gap addressed:** Assessment G2 gap — Step 4 says "open Claude Code" but does not
tell a collaborator that Claude Code requires an Anthropic account. A collaborator
without existing access hits a dead end with no guidance.

**File:** `docs/QUICK_START.md`

**Change:** Add a one-line note under Step 4 (`## Step 4 — Open Claude Code`):

```
> **Don't have Claude Code yet?** Sign up at https://claude.ai/code — a free plan
> is available.
```

**Agent:** Claude Code (documentation edit, no build required)
**Depends on:** nothing (independent)
**QA gate:** Skip (fewer than 3 files, documentation-only)

---

### change-onboard-002: Add Linux/systemd note to QUICK_START.md troubleshooting

**Gap addressed:** Assessment G3 gap — the troubleshooting section's MCP services
note references `scripts/prometheus-services.sh load`, which uses `launchctl` on
macOS. Linux users are silently left out; systemd is the correct path there.

**File:** `docs/QUICK_START.md`

**Change:** Extend the "MCP services not running" troubleshooting item to include
a Linux/systemd path alongside the existing macOS launchd note.

The updated block should read:

```
**MCP services not running** — On macOS, surface-bridge (port 7890) and sovereign-sync
(port 7892) are launchd services. Start them with:
```bash
bash scripts/prometheus-services.sh load
bash scripts/prometheus-services.sh status
```
On Linux, use systemd:
```bash
systemctl --user start prometheus-surface-bridge prometheus-sovereign-sync
systemctl --user status prometheus-surface-bridge prometheus-sovereign-sync
```
```

**Agent:** Claude Code (documentation edit, no build required)
**Depends on:** nothing (independent — both patches are in the same file but at
different locations and can be applied in either order)
**QA gate:** Skip (documentation-only)

---

## Ordering rationale

Both changes are independent documentation edits to a single file. They can be
applied in any order. Ordering: change-001 first (higher collaborator impact —
a missing Claude Code account is a harder blocker than a macOS/Linux confusion),
change-002 second.

---

## What this plan does NOT close

- G1: No collaborator identified → requires direct personal outreach by the maintainer
- G2: Quick Start completion → requires a collaborator to exist (G1)
- G3: Feynman loop → requires G2
- G4: Production readiness update → requires G1–G3 evidence

These two changes are guardrails, not a substitute for human coordination. The
phase remains open after both changes are applied.

---

## First change to apply

`/kbd-apply change-onboard-001`

---

## Change records

- `.kbd-orchestrator/changes/change-onboard-001-claude-code-prereq/change.md` (migrated from OpenSpec 2026-07-02)
- `.kbd-orchestrator/changes/change-onboard-002-linux-systemd-note/change.md` (migrated from OpenSpec 2026-07-02)
