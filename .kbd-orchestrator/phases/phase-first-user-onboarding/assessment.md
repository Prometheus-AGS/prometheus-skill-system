# Assessment — phase-first-user-onboarding

**Date:** 2026-06-30  
**Assessor:** Claude Code (Sonnet 4.6)  
**Previous phase:** phase-external-validation (92% readiness, reflect_complete — 0/5 goals MET)  
**Binding constraint inherited:** BG-2 — no external collaborator

---

## Context

This phase exists because phase-external-validation correctly built the infrastructure
for external validation but could not produce it — all five goals required a second
human who did not participate. The phase-external-validation reflection identified
three root causes:

- L1: Phases with external-validation goals need a named collaborator before they start
- L2: "Removing barriers" and "achieving outcomes" need separate success criteria
- L3: Passive issue creation has zero reach without an established audience

This assessment maps the current state against each of those lessons and identifies
what must be true before the four goals of this phase can close.

---

## Current state — audience and reach

### GitHub repo metrics (as of 2026-06-30)

| Metric | Value |
|---|---|
| Unique visitors (recent period) | 4 |
| Total page views | 11 |
| Contributors | 1 (GQAdonis only) |
| Collaborators | 1 (GQAdonis only) |
| Issue #14 comments | 0 |
| Stars / forks | 0 |

**Implication:** The repo has essentially no organic audience. Issue #14's call for
feedback reaches exactly zero external people unless the maintainer actively sends it
to someone. The L3 lesson from the previous phase is confirmed by the traffic data.

### Infrastructure readiness (inherited from phase-external-validation)

| Artifact | Status | Collaborator-ready? |
|---|---|---|
| `docs/QUICK_START.md` | Present, 128 lines | Yes — covers clone → install → `/learn-goal` |
| `tests/sycophancy-corpus/` | 6 fixtures + verdicts | Yes — self-contained with README |
| `docs/SOVEREIGN_SYNC_TESTING.md` | Present | Yes — Docker + two-host paths |
| `docs/guide/19-installation.md` | Present | Requires Rust/MCP familiarity |
| GitHub Issue #14 | Open, 0 comments | Needs active outreach to reach anyone |

The Quick Start is the right entry point for a collaborator. It covers the learn loop
path (G2 and G3). The full installation guide is available for collaborators who want
more detail.

### What a collaborator needs

A collaborator who can participate in G2 (Quick Start) and G3 (Feynman loop) needs:
- macOS or Linux (Windows is untested — `install-skills-flat.sh` uses bash)
- Rust 1.75+ (for building the substrate binaries)
- Node.js 18+ and Git
- A Claude Code license (the learn skills run in Claude Code)
- ~15–30 minutes for the full Quick Start + first Feynman loop

The Claude Code license requirement is the highest barrier: Claude Code is not free,
and asking someone to install it for a validation exercise requires either existing
access or a specific reason to acquire it. This is a real friction point that the
Quick Start does not address.

---

## Goal-by-goal gap analysis

### G1 — Identify and confirm a collaborator

**Gap:** No collaborator has been identified. The GitHub Issue is open but has 0 comments.
Active outreach is required.

**Concrete outreach targets the maintainer could contact:**
- Colleagues or teammates who already use Claude Code
- Members of AI/agent tooling communities (Discord servers, Slack workspaces)
- Anyone who has expressed interest in AI-assisted development workflows

**What makes outreach more likely to succeed:**
- Send the Quick Start link directly, not the repo root
- Offer to be present in real time during their session (reduces the risk of
  them getting stuck and giving up)
- Explicitly say what you need from them: "run 3 commands and tell me if it works"

**Code can do:** nothing. This is a human action.

### G2 — Collaborator completes the Quick Start

**Gap:** No collaborator has been identified (G1 not met).

**Infrastructure readiness:** GOOD. The Quick Start covers the path. One gap identified:
the Quick Start says "open Claude Code" but does not mention that Claude Code requires
a subscription/login. If the collaborator does not have Claude Code, they will hit a
wall at Step 4 that the guide does not address.

**Suggested fix to Quick Start:** add a one-line note under Step 4:
"Claude Code requires an Anthropic account — sign up at [claude.ai/code](https://claude.ai/code) if you haven't already."

### G3 — Collaborator completes the Feynman loop

**Gap:** G1 not met. Beyond that, the Feynman loop itself is ready — 12 learn skills
installed, surface-bridge running, sycophancy gate wired. No blocking code issues found.

**One risk:** if the collaborator is on Linux, launchd is macOS-only. The install script
attempts systemd on Linux. The Quick Start does not mention this split. The troubleshooting
section covers it (`prometheus-services.sh load`), but only for macOS users.

### G4 — Capture outcomes in production readiness report

**Gap:** G1, G2, G3 not met. G4 is the written record — it can only be done after the
others produce evidence.

---

## What this phase can do right now

Two things are actionable without a collaborator:

1. **Fix the Quick Start's Claude Code prerequisite gap** — a one-line addition that
   prevents a dead end at Step 4 for anyone who doesn't already have Claude Code.

2. **Add a Linux launchd note** — the Quick Start is macOS-centric; add a note for
   Linux users pointing to the systemd path in the full installation guide.

Beyond those two small fixes, this phase is entirely gated on human action.

---

## Assessment verdict

**The phase is valid but gated.** G1 is the only goal the maintainer can initiate,
and G1 requires direct personal outreach — not infrastructure. G2, G3, and G4 follow
from G1.

**Recommended changes:** 2 (Quick Start patch for Claude Code prerequisite + Linux
note). Both are small. The rest of the phase is human coordination.

**The honest position:** this phase will not close in a coding session. It closes
when the maintainer sends a message to a real person and that person says yes.

**Assessment complete.**
