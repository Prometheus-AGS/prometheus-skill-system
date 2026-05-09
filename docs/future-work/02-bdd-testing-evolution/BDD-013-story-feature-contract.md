---
id: BDD-013
title: User-story to feature contract (OpenSpec change-id tagging)
status: planned
priority: P1
estimated_effort: 1w
agent_role: docs-writer
depends_on: [BDD-008]
unblocks: [BDD-014]
related: []
created_from_conversation_turn: 5-6
---

# BDD-013 — User-story to feature contract

## Problem

User stories live in `docs/user-stories/`. Feature files live in `tests/features/`. The docs site at `docs/site/` is generated *from feature files* by `generate-bdd-docs.ts`. There is no enforced relationship between the user-story documents and the feature files. A user story can exist without any feature exercising it; a feature can exist without any user story justifying it.

This invites the "the docs say feature X is done, but no test ever exercised it" surprise.

The original ask was for *bidirectional sync*. Bidirectional is unstable: two authoritative sources require resolution rules for every conflict, and the rules don't generalize. The recommended path is *one direction* with the others as generated outputs.

## Evidence

Inspect `docs/user-stories/`. Inspect `tests/features/`. There is no link between them. The OpenSpec change records under `openspec/changes/<change-id>/proposal.md` *do* describe behavior changes; they're the natural integration point.

## Why it matters

- **Coverage transparency.** "Which scenarios prove change-XYZ is implemented?" should be answerable by query.
- **Living documentation.** Users see "this feature was specified in change-XYZ, validated by N scenarios, last verified at TIMESTAMP" as a per-feature block in the docs site.
- **Triage of stale features.** Features whose linked OpenSpec change is closed-without-implementation are flagged for cleanup.

## Proposed fix

Adopt a single direction: **tests are tagged with OpenSpec change-IDs**. The feature file (or scenario) declares which change(s) it implements:

```gherkin
@ui @video @change-005 @change-007
Feature: Acquisition edit workflow
  ...
  @scenario-id-12345
  Scenario: Save and re-edit preserves all fields
    ...
```

The docs generator (`generate-bdd-docs.ts`) reads these tags and produces:

1. **Per-feature block** showing which OpenSpec changes the feature traces back to, with deeplinks.
2. **Per-change reverse view** (`docs/site/change/<change-id>.html`) showing all features and scenarios that exercise the change. Pulled from the change's `proposal.md` plus the scenarios tagged with the ID.
3. **Coverage report** (a CI artifact) listing OpenSpec changes with no exercising scenarios — flagged for review.

User stories are *generated* from this combined data: each `docs/user-stories/<id>.md` becomes a synthesis of its OpenSpec change record + the scenarios that exercise it. Stories aren't a parallel input; they're a curated rendering.

## Trade-offs and risks

- **Risk: existing user stories are deeply detailed and shouldn't be auto-generated.** Mitigation: existing stories stay as-is; the generated stories live alongside (e.g. in `docs/site/story/<id>.html`). The generated form supplements, doesn't replace, the human-written form. Over time, manual stories age out; generated form becomes canonical.
- **Risk: tag bloat.** Some scenarios serve multiple changes; the tag list gets long. Mitigation: tag is metadata; visual rendering can group/abbreviate.
- **Cost: requires retroactive tagging of existing 250+ scenarios.** Realistic ask: tag only scenarios that map to recent (last 6 months) OpenSpec changes; older scenarios are uncategorized and tracked for gradual coverage.
- **Risk: developers forget to tag new scenarios.** Mitigation: a CI check warns when a new scenario is added without any `@change-*` tag. Warning, not block, initially.

## Acceptance criteria

- [ ] Convention documented: scenarios reference OpenSpec changes via `@change-<id>` tags.
- [ ] `generate-bdd-docs.ts` reads tags and produces per-feature blocks linking to changes.
- [ ] Per-change reverse view at `docs/site/change/<change-id>.html` exists.
- [ ] Coverage report flags untagged scenarios and changes-without-scenarios.
- [ ] Generated user-story pages at `docs/site/story/<id>.html` synthesize change + scenario data.
- [ ] CI check warns on untagged new scenarios.

## Implementation steps

1. Document the tagging convention in `tests/README.md` and skill-pack `CLAUDE.md`.
2. Update `generate-bdd-docs.ts` to extract tag references.
3. Generate per-change reverse pages.
4. Generate user-story pages from synthesis.
5. Add the coverage report.
6. Add the CI warn check.
7. Retroactively tag a sample of recent scenarios.

## Dependencies

BDD-008 (codegraph helps populate tag data and validate tag references).

## Open questions

- Should scenario-level `@change-<id>` tags propagate up to feature-level for rendering convenience? Probably yes.
- What if an OpenSpec change spans multiple repos? Multi-repo aggregation is a future concern; per-repo for now.
