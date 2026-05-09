---
id: BDD-015
title: Feedback record to draft-scenario emitter
status: planned
priority: P1
estimated_effort: 3-5d
agent_role: bdd-engineer
depends_on: [BDD-007]
unblocks: []
related: [BDD-014]
created_from_conversation_turn: 5-6
---

# BDD-015 — Feedback record to draft-scenario emitter

## Problem

Feedback records get triaged with `intent` (bug/ux/feature/question), `confidence`, `userFacingSummary`, etc. When intent is `bug` with high confidence, this is a candidate for coverage — there's a behavior the user expected that isn't enforced by tests. Currently, the path from "user reports bug" to "test scenario protecting against regression" is entirely manual.

## Evidence

`supabase/functions/analyze-feedback/index.ts` produces structured triage. The output is stored. No automation consumes it for test generation.

## Why it matters

Feedback records are early signal of regression risk. Each high-confidence bug report represents a behavior the user expected. If no test exercises that behavior, the bug can recur. Auto-emitting a *draft* scenario (per BDD-007's directory) creates a low-friction path from feedback → coverage.

Not auto-running the draft — that would violate BDD-006. Auto-creating the draft for human review is the right level of automation.

## Proposed fix

A small post-triage hook in the feedback Edge Function (or a separate cron job querying recent triages):

1. **Query criteria.** Triage records with `intent: 'bug'`, `confidence > 0.7`, `status: 'triaged'` (not yet acted upon).
2. **Generate draft.** For each, emit a `.feature` file in `tests/features/drafts/from-feedback/<slug>.feature` containing:
   - Header comment with the feedback record ID and timestamp.
   - `@draft @from-feedback @change-<linked-change-id-if-any>` tags.
   - Scenario synthesized from `userFacingSummary` + page-context: `Given I am on <page>`, `When ...`, `Then I should see <expected behavior from summary>`.
3. **Mark feedback as drafted.** Update `feedback_records.status` to `drafted` so re-runs don't duplicate.
4. **Notify reviewer.** Slack/email/GitHub-issue with link to the draft for human review.

Drafts that humans review and find good get promoted via `pnpm test:bdd:promote-draft <slug>` (per BDD-007).

## Trade-offs and risks

- **Risk: low-quality drafts overwhelm reviewers.** Mitigation: the 0.7 confidence threshold filters most. Periodic review of acceptance rates tunes the threshold.
- **Risk: drafts are syntactically broken (missing testids, etc.).** Mitigation: validate-testid-coverage (BDD-005) is the natural backstop. The `pnpm test:bdd:drafts` profile catches structural issues without blocking the live suite.
- **Risk: drafts leak sensitive info from the feedback record.** Mitigation: redaction reused from feedback-project-adapter applies before draft emit.
- **Risk: drafts pile up uncreviewed.** Mitigation: BDD-007's stale-after-30-days flagging covers this.

## Acceptance criteria

- [ ] A trigger (post-triage in Edge Function or scheduled job) emits drafts for qualifying feedback records.
- [ ] Drafts land in `tests/features/drafts/from-feedback/<slug>.feature`.
- [ ] Each draft references the originating `feedback_records.id`.
- [ ] `feedback_records.status` updates to `drafted` after emission.
- [ ] Notification fires to a configured channel (Slack/email/issue).
- [ ] Drafts pass `validate-testid-coverage` (BDD-005).
- [ ] Smoke test: a synthetic high-confidence bug report produces a coherent draft.

## Implementation steps

1. Identify the trigger location (post-triage hook in Edge Function vs separate worker).
2. Implement the synthesis step (build scenario text from triage fields).
3. Apply redaction.
4. Write the feature file.
5. Update feedback record status.
6. Implement notification.
7. Test end-to-end.

## Dependencies

BDD-007 (drafts directory must exist).

## Open questions

- Should the synthesis use an LLM call to produce more natural Gherkin? Probably yes; the structured fields don't always read fluently. Use a cheap model with strict-output mode.
- What about UX feedback and feature requests? Bugs are the clearest case for draft emission. UX is sometimes appropriate (a "user can't find this") becomes a navigation scenario. Feature requests are rarely tests-yet — they're scope changes. Default: bugs only; consider extension after operational experience.
