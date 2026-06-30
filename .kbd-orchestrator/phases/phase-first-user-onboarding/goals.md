# Goals — phase-first-user-onboarding

**Context:** phase-external-validation closed with 0/5 goals MET. All five goals
required a second human (external user) — none participated. The binding constraint
was BG-2: no identified external collaborator. This phase addresses that directly.

The distinction from phase-external-validation is narrow but critical:
- phase-external-validation built the *infrastructure* for external validation
- phase-first-user-onboarding finds a *specific person* and walks through the loop with them

This phase succeeds or fails on a single question: did a second human run the system?

## Goals

- [ ] **G1 — Identify and confirm a collaborator:** Name a specific person who has agreed
  to attempt onboarding. The person must be outside the maintainer's own Claude Code
  session — a colleague, community member, or beta tester. Document their name/handle
  and the agreed date. A GitHub issue comment from someone saying "I'll try this" is
  sufficient confirmation.

- [ ] **G2 — Collaborator completes the Quick Start:** The identified person follows
  `docs/QUICK_START.md` from clone to a running `/learn-goal` session. They report
  back (GitHub Issue #14 comment, email, or any documented channel) what worked and
  what broke. If the Quick Start fails them, fix the guide and re-attempt — that still
  counts as a validation cycle.

- [ ] **G3 — Collaborator completes the Feynman loop:** The same or different person
  runs `/learn-goal → /feynman-loop → /learn-grade` end-to-end on a real topic of
  their choice, without the maintainer driving the session. A session log or issue
  comment describing the outcome is the evidence.

- [ ] **G4 — Capture the outcome in the production readiness report:** Update
  `docs/production-readiness-report.md` with the G1–G3 evidence. Remove the
  PENDING placeholders and replace them with actual outcomes. Run the sycophancy
  gate on the updated claims.

## Definition of Done

G1 through G3 require at least one non-maintainer human to participate. This phase
cannot close without that participation. G4 is the written record.

All four goals MET = sufficient evidence to justify a readiness claim above 92%.
The exact new percentage depends on what issues the collaborator discovers.

## What this phase does NOT require

- A second external user (one is enough for initial validation)
- A perfect onboarding experience (friction is expected; documenting it is the work)
- Any code changes to forge-rs, sovereign-sync, or the learn domain
  (unless the collaborator hits a blocking bug)

## What the maintainer can do to make this phase succeed

- Actively reach out to 2–3 people who might be willing to try the system
- Share `docs/QUICK_START.md` directly rather than linking to the repo root
- Be available to answer questions in real time during the collaborator's session
- Treat every friction point the collaborator hits as data, not failure
