---
id: BDD-014
title: Feedback aggregation in docs site
status: planned
priority: P1
estimated_effort: 3-5d
agent_role: bdd-engineer
depends_on: [BDD-013]
unblocks: []
related: [BDD-015]
created_from_conversation_turn: 5-6
---

# BDD-014 — Feedback aggregation in docs site

## Problem

The feedback engine accumulates `feedback_records` in Supabase with structured triage results: `intent`, `area`, `severity`, `userFacingSummary`, `devFacingSummary`, etc. None of this surfaces back into the docs site at `docs/site/`. A user looking at the "Acquisitions" area page has no idea that 12 questions, 3 known bugs, and 5 feature requests have been filed against it.

The data is in Supabase. The docs build is static. They never meet.

## Evidence

1. Inspect `feedback_records` schema in `supabase/migrations/20260326000000_feedback_platform.sql`.
2. Inspect `generate-bdd-docs.ts`. No Supabase reads.

## Why it matters

The docs site becomes more valuable when it reflects user signals. "Documentation + known issues + asked questions + pending features" in one place beats "documentation alone" because users searching for context find their pain points in the same place they're documented.

Triggering: this also closes the intended loop in the original "use cases ↔ tests ↔ docs" ask. Feedback records are the user-side voice; bringing them into docs is bidirectional in the right way (data flows users → triage → docs build).

## Proposed fix

Extend `generate-bdd-docs.ts` (or a sibling script) to read `feedback_records` at docs-build time and surface aggregations:

**Per-area page sidebar block.** For each functional area, show:
- Open questions (intent: `question`) — top 5 by recency.
- Known issues (intent: `bug`, status: `confirmed` or `in_progress`).
- Pending feature requests (intent: `feature_request`, status: `triaged`).

**Linked details.** Each entry deeplinks into a feedback record viewer (read-only) with the full triage result.

**Pull mechanism.** During `pnpm docs:generate`, a small Supabase read fetches the latest feedback aggregations (anonymized). Cached for 5 minutes if rate-limited. Anonymization strips email, name, IP from any displayed text.

**Per-feature page block.** When a feature page renders, show feedback records that match by area + scenario name. Heuristic-based but valuable.

## Trade-offs and risks

- **Risk: privacy.** Feedback records contain user-submitted text and (potentially) screenshots. Mitigation: docs-site rendering shows triage results (`userFacingSummary` is the canonical safe-for-display form) but never raw screenshots or unredacted bodies.
- **Risk: feedback dominates the docs site visually.** Mitigation: sidebar block, not main flow. Collapsed by default.
- **Risk: stale during long doc-rebuild gaps.** Acceptable; rebuild cadence is daily anyway.
- **Risk: leaks confidential info.** Project adapter's redaction (already in `feedback-project-adapter`) is reused at render time.

## Acceptance criteria

- [ ] Docs-generate reads feedback_records via Supabase client.
- [ ] Per-area page shows open questions / known issues / pending feature requests.
- [ ] Per-feature page shows matching feedback records.
- [ ] Anonymization applied; redaction reused.
- [ ] Cache or rate-limit handling for Supabase reads.
- [ ] No breaking changes to existing pages.

## Implementation steps

1. Add Supabase client setup to docs-generate.
2. Define query filters per intent / area / status.
3. Build the sidebar block component (HTML).
4. Build the per-feature matching heuristic.
5. Apply anonymization at render.
6. Test on real (or fixture) feedback data.

## Dependencies

BDD-013 (the area/change tagging contract is the linking key for per-feature matching).

## Open questions

- Does this need a "submit new feedback from this docs page" link? Probably yes — the docs site is the natural place for a user to file feedback while reading. Hooks into the existing feedback engine (no new API).
- What's the right Supabase access pattern (service-role vs anon-with-RLS)? Anon-with-RLS plus anonymization is the safer default.
