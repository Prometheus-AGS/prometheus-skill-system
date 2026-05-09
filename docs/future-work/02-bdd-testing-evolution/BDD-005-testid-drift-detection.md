---
id: BDD-005
title: testid drift detection (validate-testid-coverage.ts)
status: ready
priority: P0
estimated_effort: 1d
agent_role: bdd-engineer
depends_on: []
unblocks: []
related: [BDD-006, BDD-007]
created_from_conversation_turn: 5-6
---

# BDD-005 — testid drift detection

## Problem

Step definitions reference `data-testid` selectors, e.g. `page.locator('[data-testid="acquisition-edit-form"]')`. When a developer (or code-gen agent) renames or removes a testid in a React component, the step continues to compile and the test fails at runtime — only when the scenario runs. With selective execution (BDD-010), some scenarios may not run for days. Drift is invisible until the next full pipeline run.

## Evidence

Run a build of SSR. Note that no static check runs to validate "every testid referenced by a step exists in some component."

## Why it matters

This task is one leg of the trio that replaces the "auto-update tests" category-error ask:
- BDD-005 (this): detect when a testid disappears.
- BDD-006: forbid code-gen agents from "fixing" tests by editing step defs.
- BDD-007: allow agents to add candidate test drafts.

Together these give the value Travis was after (faster feedback when code changes) without creating the auto-update tautology.

## Proposed fix

A static analyzer `scripts/validate-testid-coverage.ts`:

1. Walks all `tests/steps/*.steps.ts` files. Extracts every string passed to `page.locator('[data-testid="..."]')`, `getByTestId('...')`, and equivalents. Build a set: `referenced_testids`.
2. Walks all `src/**/*.tsx` files. Extracts every JSX `data-testid="..."` attribute. Build a set: `defined_testids`.
3. Computes:
   - `referenced - defined` → testids referenced by steps but not produced by any component. **Build-fails.**
   - `defined - referenced` → testids in components that no step exercises. **Build-warns** (it's a coverage gap, not necessarily a problem).
4. Reports both lists.

Wire as a `pnpm` script:
```json
"validate:testids": "tsx scripts/validate-testid-coverage.ts"
```
And as a CI check that runs on every PR.

## Trade-offs and risks

- **Risk: dynamic testid construction.** Some components compute testids: `data-testid={\`row-${id}\`}`. The static analyzer can't always resolve these. Mitigation: pattern-match on string concatenation; for templated patterns, allow the developer to declare the pattern in `tests/testid-patterns.json` and the validator tolerates pattern matches.
- **Risk: false-positive build fails block work.** Mitigation: provide an `--allow` list `tests/testid-allowlist.json` for known unresolved cases, with required justification comment.
- **Cost: AST walk over the whole src/ on every PR.** Sub-second on SSR's size.

## Acceptance criteria

- [ ] `scripts/validate-testid-coverage.ts` walks steps and src/ correctly.
- [ ] Reports referenced-but-undefined → exit 1.
- [ ] Reports defined-but-unreferenced → exit 0 with warning.
- [ ] Pattern declarations in `tests/testid-patterns.json` are honored.
- [ ] Allowlist mechanism with justification.
- [ ] CI check runs on every PR.
- [ ] Smoke test: introduce a fake removal of a testid; CI fails. Restore; CI passes.

## Implementation steps

1. Choose the AST walker — recommend ts-morph for both steps and src parsing (consistency with BDD-008).
2. Write the extractor for step files: regex on `getByTestId|locator.*data-testid` plus the dynamic-construction patterns.
3. Write the extractor for src/: JSX visitor for `data-testid`.
4. Compute the diff.
5. Add the pattern + allowlist support.
6. Wire into CI.
7. Document in `tests/README.md`.

## Dependencies

None. Independent of BDD-008's larger code-graph work, though once BDD-008 lands the testid extraction is a query against the codegraph rather than a separate walker.

## Open questions

- Should this evolve into a real-time check (PreToolUse hook on Edit/Write to .tsx files that affect testid attributes)? Useful but adds complexity. Start with CI-only.
- Should it also check that every testid used in `[data-testid="..."]` selectors in *components* (e.g. for component composition) is a real testid? Probably yes; same walker.
