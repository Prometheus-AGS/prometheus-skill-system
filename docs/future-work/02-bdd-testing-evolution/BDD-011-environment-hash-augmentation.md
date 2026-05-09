---
id: BDD-011
title: Environment hash augmentation
status: planned
priority: P1
estimated_effort: 1d
agent_role: bdd-engineer
depends_on: [BDD-010]
unblocks: []
related: []
created_from_conversation_turn: 5-6
---

# BDD-011 — Environment hash augmentation

## Problem

BDD-010 hashes the source-file closure of each scenario's impact set. This is necessary but not sufficient. Several runtime factors affect test correctness without showing up in source files:

- `prisma/schema.prisma` changes invalidate every DB-touching scenario.
- `tests/.env`, `.env.local` changes alter runtime configuration.
- `package.json` (dependency version bumps) affects behavior.
- `prisma/migrations/` changes the live DB state.
- `cucumber.js` profile changes affect test setup.

If any of these change without source files changing, BDD-010's pure-source-hash will declare scenarios "unchanged" and skip them — a correctness gap.

## Evidence

Reason about the failure mode: a Prisma schema migration that adds a column changes which scenarios pass without changing any `.ts` file. BDD-010's hash misses it; cached pass status persists incorrectly.

## Why it matters

The selective-execution gain in BDD-010 is real, but its correctness depends on the impact-set capturing the right things. Environmental factors are exactly the kind of "everyone forgets to include" that creates silent skipped-but-actually-broken scenarios.

## Proposed fix

Extend the impact-set hash to include environmental factors:

```
impact_set_hash = sha256(
  sorted(file_path:content_hash for file in scenario.exercises_files)
  + ":"
  + sha256(prisma/schema.prisma content)
  + ":"
  + sha256(tests/.env content if it exists)
  + ":"
  + sha256(.env.local content if it exists)
  + ":"
  + sha256(package.json content)
  + ":"
  + sha256(cucumber.js content)
  + ":"
  + sha256(sorted(file_paths in prisma/migrations/))
)
```

Each environmental factor is configurable: a `.bdd-impact-config.json` file at the project root lists factors with their hash strategy. Defaults cover the common cases above.

## Trade-offs and risks

- **Risk: over-invalidation.** Bumping a dev-only dependency in `package.json` invalidates every scenario's hash. Mitigation: use `package.json`'s `dependencies` and `devDependencies` separately; the test impact only needs `dependencies` + a small set of test-relevant `devDependencies` (Playwright, Cucumber). Configure precisely.
- **Risk: hash thrash from generated files.** Mitigation: include only hand-edited files; exclude generated lockfiles (`pnpm-lock.yaml`) from the hash by default.
- **Risk: developer adds an env factor and forgets to add it to the config.** Mitigation: a "smell" check that warns when the impact-set has had 0% miss rate for 2 weeks (suggests over-broad hashing) or 100% miss rate (suggests under-broad hashing).

## Acceptance criteria

- [ ] `.bdd-impact-config.json` schema documented and example committed.
- [ ] Hash computation includes env factors.
- [ ] When `prisma/schema.prisma` changes, all scenarios needing DB access re-run.
- [ ] When `tests/.env` changes, all scenarios re-run.
- [ ] When `pnpm-lock.yaml` changes (default-excluded), no re-run.
- [ ] When `package.json` `dependencies` changes, all scenarios re-run.
- [ ] When `package.json` `devDependencies` changes outside the relevant set, no re-run.
- [ ] Documentation explains the configurability.

## Implementation steps

1. Define the config schema.
2. Update the hash computation in `run-video-proof.ts` to honor it.
3. Commit a default config.
4. Test by intentionally changing each factor and observing re-run behavior.
5. Document.

## Dependencies

BDD-010 (must exist to be augmented).

## Open questions

- Should branches have different env hashes (e.g. `main` vs feature branch)? Probably yes, since merge-base diff is the meaningful comparison.
- Is there a way to detect environmental factors automatically? Possibly via "files referenced by the test runner during a typical run" — but error-prone. Stick with configuration-as-data.
