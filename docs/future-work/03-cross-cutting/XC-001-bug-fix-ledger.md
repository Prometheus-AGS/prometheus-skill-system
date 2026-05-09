---
id: XC-001
title: Bug-fix-ledger quarterly invariant promotion
status: ready
priority: P2
estimated_effort: recurring
agent_role: skill-pack-maintainer
depends_on: []
unblocks: []
related: [SP-001]
created_from_conversation_turn: 1-2, 3-4
---

# XC-001 — Bug-fix-ledger quarterly invariant promotion

## Problem

Per the architectural patterns (`05-references/architectural-patterns.md`), invariants and regression-guards have different lifecycles. Invariants are properties that must always hold; regression-guards are tests for specific bugs that occurred. Mixing them in the same review channel is the bug — invariants get buried under accumulating regression-guards.

Currently, there is no separate ledger for regression-guards. CLAUDE.md files accumulate everything indistinguishable.

## Evidence

Read SSR `CLAUDE.md` and skill-pack `CLAUDE.md` (after SP-001 reconciliation). Some rules are universal (no `any` types). Some are bug-specific ("after fix #4731, validate YYYY-MM-DD parser before MM/DD/YYYY"). They're listed at the same level of importance.

## Why it matters

- **Reviewability.** A 200-line CLAUDE.md with both kinds is hard to read; a reader can't tell which rules are eternal and which are specific historical fixes.
- **Decay.** Regression-guards for bugs from 2 years ago may be obsolete; mixed in with invariants, they don't get pruned.
- **Promotion path.** A regression-guard that has prevented multiple regressions deserves promotion to invariant status. Without separation, this happens informally or not at all.

## Proposed fix

Establish two files at each enforcement scope (project-level and skill-pack-level):

**`CLAUDE.md`** holds *invariants only*. ~20-50 lines. Reviewed annually.

**`BUG_FIX_LEDGER.md`** holds *regression-guards*. Format:

```markdown
## 4731 — Date parser ordering — 2026-03-12

**Bug:** `MM/DD/YYYY` was tried before `YYYY-MM-DD`, so 2024-01-13 was misread as January 13, 2024 only by accident (it parsed valid both ways).
**Fix:** Reverse parser order in `lib/dates/parse.ts`.
**Guard:** `data-iso-date-format` test. Has prevented 2 regressions since.
**Promotion candidate:** No (guard count = 2; threshold = 3).
```

**Quarterly review process.** Once per quarter (`XC-001` recurrence), the maintainer:
1. Lists ledger entries by guard-count.
2. Promotes entries with guard-count >= 3 to invariant status (move to CLAUDE.md, simplify language).
3. Retires entries with no guard activity in 1 year (the bug pattern is no longer relevant).
4. Confirms remaining entries.

This makes invariant promotion a deliberate, reviewed event rather than implicit.

## Trade-offs and risks

- **Cost: ledger maintenance is recurring work.** Bounded — quarterly, ~1 hour per pass. The cost of not having it (CLAUDE.md decay, lost regression-prevention signal) is higher.
- **Risk: developers don't update the ledger.** Mitigation: include "did you add a ledger entry?" in PR templates for bug-fix PRs.
- **Risk: the threshold (3 guards) is arbitrary.** Tune based on operational experience. Worst case: a rule that has prevented exactly 3 regressions is promoted slightly early, no real harm.

## Acceptance criteria

- [ ] `BUG_FIX_LEDGER.md` exists at SSR project root and skill-pack root.
- [ ] Format documented in `docs/processes/bug-fix-ledger.md`.
- [ ] PR template includes ledger update prompt for bug-fix PRs.
- [ ] First quarterly review completed and documented.
- [ ] CLAUDE.md is reduced to invariants-only after first review.

## Implementation steps

1. Create the file structure and README.
2. Backfill from existing CLAUDE.md content (move regression-guards to ledger).
3. Update PR templates.
4. Schedule first quarterly review (calendar reminder).
5. Document the process.

## Dependencies

None functional. Synergy with SP-001 (canonical CLAUDE.md location).

## Open questions

- Should the ledger be per-project or pack-wide? Per-project is more useful (specific bug references); pack-wide is for genuinely-cross-project lessons. Both, with explicit scoping.
