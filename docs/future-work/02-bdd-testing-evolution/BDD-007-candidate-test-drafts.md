---
id: BDD-007
title: Candidate test drafts (tests/features/drafts/)
status: ready
priority: P1
estimated_effort: 1d
agent_role: bdd-engineer
depends_on: []
unblocks: [BDD-015]
related: [BDD-005, BDD-006]
created_from_conversation_turn: 5-6
---

# BDD-007 — Candidate test drafts

## Problem

BDD-006 forbids code-gen agents from editing existing tests. But agents *should* be able to contribute to test coverage when they add new behavior. Without an outlet, agents either bypass the rule (bad) or contribute nothing (worse — tests grow only as fast as humans can write them).

## Evidence

The conversation noted this gap explicitly. The pattern needed: a directory where agents can drop draft `.feature` files describing behavior they've added, which humans review and promote to the live suite.

## Why it matters

Tests grow with code only when there is a low-friction path for adding them. The drafts directory is that path.

## Proposed fix

Create `tests/features/drafts/` as a recognized directory:

1. **Excluded from the standard suite.** `cucumber.js` profiles in SSR exclude `tests/features/drafts/**` from the default and `ui` profiles.
2. **Tagged `@draft`.** Cucumber tag is a reminder for humans reading the file.
3. **Promoted to live by humans.** The promotion is `git mv tests/features/drafts/<file>.feature tests/features/ui/<file>.feature` plus removing the `@draft` tag and stripping any `# DRAFT` comments.
4. **Convention for agents:** drafts include a `# DRAFT — added by agent in commit <sha>` comment line at top, plus the originating user-story or change-id reference.

A new `pnpm test:bdd:drafts` profile *does* run drafts, separately, so agents and humans can see if a draft works end-to-end before promotion. Drafts that pass `test:bdd:drafts` for 3 consecutive runs are flagged "ready for promotion" by a small script that scans drafts and the test history.

A complementary `pnpm test:bdd:promote-draft <slug>` command does the promotion mechanically (move file, strip `@draft` and comment, regenerate docs).

## Trade-offs and risks

- **Risk: drafts accumulate without ever being promoted.** Mitigation: monthly reminder script that lists drafts older than 30 days as "stale candidates"; a quarterly review prompts the team to promote, edit, or delete.
- **Risk: drafts conflict with existing scenarios.** Mitigation: the validate-testid-coverage check (BDD-005) still runs on drafts to surface conflicts early.
- **Risk: agents include drafts that are simply wrong.** Mitigation: human review is required before promotion. The drafts dir is the *queue*, not the deliverable.

## Acceptance criteria

- [ ] `tests/features/drafts/` exists and is recognized.
- [ ] Standard test profiles exclude drafts.
- [ ] `pnpm test:bdd:drafts` profile runs drafts.
- [ ] `pnpm test:bdd:promote-draft <slug>` does the promotion mechanically.
- [ ] Validate-testid-coverage covers drafts.
- [ ] CLAUDE.md updated with the convention (per BDD-006).
- [ ] Sample draft committed showing the format.

## Implementation steps

1. Create the directory; commit a README in it explaining the convention.
2. Update `cucumber.js` profiles to exclude drafts from default and ui profiles.
3. Add `test:bdd:drafts` profile.
4. Implement `test:bdd:promote-draft` script.
5. Update CLAUDE.md and SKILL.md with the convention.
6. Commit a sample draft.

## Dependencies

None functional. Synergy with BDD-005 (covers drafts) and BDD-006 (the rule that creates the need).

## Open questions

- Should the agent be allowed to modify its own previously-drafted tests? Probably yes, with a `# UPDATED BY AGENT IN <sha>` comment. Drafts are not under the BDD-006 rule because they aren't promoted.
- Should drafts have an expiration policy? Recommend: stale-after-30-days flag plus quarterly cleanup. Don't auto-delete.
