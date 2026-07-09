---
name: bdd-lifecycle-loop
version: '1.0.0'
license: MIT
description: >
  Codifies the create → run → triage → maintain workflow for BDD test
  suites. Includes outside-in scenario authoring, flake-budget enforcement
  wrapping cucumber's --retry-tag-filter primitive, an immutable-tests CI
  guard, and a visual-baseline refresh workflow with paper trail. Use when
  standing up a new BDD suite, growing an existing suite past 100
  scenarios, or when flakiness starts eroding trust in CI.
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [testing, bdd, cucumber, lifecycle, flake-budget, immutable-tests, ci]
---

# BDD Lifecycle Loop

A repeatable workflow for the four phases of a BDD suite's life. Pairs
with `bdd-cucumber-js` and `bdd-cucumber-rs` (which cover *how to author*)
and `bdd-video-proof` (which covers *how to certify*).

## The loop

```
      ┌──────────────┐
      │  1. Author   │  Write failing feature → step definitions
      └──────┬───────┘
             ↓
      ┌──────────────┐
      │  2. Run      │  cucumber-js / cargo test → cucumber.json
      └──────┬───────┘
             ↓
      ┌──────────────┐
      │  3. Triage   │  Fail? Real bug or flake?
      │              │  Tag @flaky, open ticket, retry
      └──────┬───────┘
             ↓
      ┌──────────────┐
      │  4. Maintain │  Refresh baselines, prune stale scenarios,
      │              │  enforce flake budget, enforce immutable tests
      └──────┬───────┘
             ↓ (loop back to 1)
```

Skip any phase and the loop breaks:
- Skip **Author outside-in** → step definitions bake in implementation details, tests become brittle
- Skip **Triage** → flakes accumulate, CI signal degrades
- Skip **Maintain** → visual baselines drift, scenarios rot, budget explodes

## Phase 1: Author (outside-in)

Write the **failing scenario first**. Do NOT let the AI or human write
the step definitions from the feature file, run once, and call it done —
that's inside-out. Instead:

1. Draft the `.feature` file from the user story or acceptance criterion
2. Run cucumber — it MUST fail with "step undefined" for every step
3. Add the first missing step; the assertion inside must still fail (feature not implemented)
4. Implement the smallest slice of production code that turns that assertion green
5. Repeat for each remaining step

**Never** write production code without a failing scenario driving it.

### Feature file conventions

- One feature per file. One behavior per scenario.
- Declarative language: "signs in with valid credentials", not "clicks button X and types Y"
- Use `Background` for shared preconditions across scenarios in a file
- Use `Scenario Outline` for data variations
- Tag every scenario:
  - Layer: `@api`, `@ui`, `@system`
  - Speed/stability: `@smoke`, `@slow`, `@flaky`
  - Ownership: `@team:auth`, `@owner:alice`
- Reference `data-testid` selectors when the scenario is `@ui`

## Phase 2: Run

The cucumber runner writes `cucumber.json` (machine-readable) alongside
whatever human-readable report you configured (`html`, `progress-bar`,
etc.). The lifecycle loop's tooling reads `cucumber.json` — do not skip it.

```bash
# cucumber-js
npx cucumber-js --format json:tests/reports/cucumber.json

# cucumber-rs
cargo test --test features -- --format=json > tests/reports/cucumber.json
```

Feed `cucumber.json` into:
- `bdd-video-proof` — bundle for certification
- `scripts/flake-budget.sh` — enforce the flake budget

## Phase 3: Triage

When a scenario fails, the triage rule is:

| Failure looks like | Action |
|--------------------|--------|
| Real regression (assertion caught a genuine change in behavior) | Open a bug; do NOT re-run; do NOT tag `@flaky` |
| Timeout under load | Increase timeout; investigate throughput regression |
| Race between test and system | Add a wait/expect; do NOT sleep |
| Passes on rerun; no timing/data reason found | Tag `@flaky`, open a ticket, put on the budget |
| Third-party service down | Skip via tag, page the vendor's on-call |

**Do NOT** re-tag a scenario `@flaky` to make CI green. That's how a
suite degrades to a rubber stamp.

Retry the flake-only subset with cucumber's built-in filter:

```bash
# cucumber-js
npx cucumber-js --retry 2 --retry-tag-filter "@flaky"

# cucumber-rs — no built-in retry filter; wrap with scripts/flake-budget.sh
```

## Phase 4: Maintain

### Flake budget

The flake budget is a **hard cap on the number of `@flaky` scenarios and
their age**. Wraps cucumber's `--retry-tag-filter @flaky` primitive with an
enforcement script.

`.bdd-flake-budget.json` at the project root:

```json
{
  "max_flaky_scenarios": 5,
  "max_flaky_age_days": 14,
  "grace_scenarios": []
}
```

Run in CI:

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/testing/bdd-lifecycle-loop/scripts/flake-budget.sh" \
  tests/features \
  .bdd-flake-budget.json
```

Exits non-zero when:
- More than `max_flaky_scenarios` are tagged `@flaky`, OR
- Any `@flaky` scenario has been tagged longer than `max_flaky_age_days`
  (unless listed in `grace_scenarios`)

CI failure forces a decision: fix the flake, delete the scenario, or add
it to `grace_scenarios` with a linked ticket. Silent flake accumulation is
impossible.

### Immutable tests

The **immutable-tests rule** is enforced two ways:

1. **PreToolUse hook** (agent-time): `shared/scripts/protect-tests.sh`
   blocks code-generation agents from editing `tests/steps/**`,
   `tests/features/**`, or `tests/support/**` files — see
   [references/immutable-tests.md](references/immutable-tests.md).
2. **CI gate** (PR-time): `scripts/test-file-diff-guard.sh` fails PRs
   that modify protected test files without a `test-change-approved`
   label.

Both together mean tests never move to accommodate a change in production
code — production code moves to satisfy tests. New behavior requires new
tests, not edited old ones.

### Visual baseline refresh

Playwright's `--update-snapshots` overwrites baselines silently. Coupled
with the immutable-tests rule, baseline changes need a paper trail. See
[references/visual-baseline-refresh.md](references/visual-baseline-refresh.md)
for the branch-and-review workflow.

## See also

- [bdd-cucumber-js](../bdd-cucumber-js/SKILL.md) — how to author TS scenarios
- [bdd-cucumber-rs](../bdd-cucumber-rs/SKILL.md) — how to author Rust scenarios
- [bdd-video-proof](../bdd-video-proof/SKILL.md) — certification bundles
- [references/immutable-tests.md](references/immutable-tests.md)
- [references/visual-baseline-refresh.md](references/visual-baseline-refresh.md)
- `docs/future-work/02-bdd-testing-evolution/BDD-002-flake-quarantine.md`
- `docs/future-work/02-bdd-testing-evolution/BDD-005-testid-drift-detection.md`
- `docs/future-work/02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md`
- `docs/future-work/02-bdd-testing-evolution/BDD-007-candidate-test-drafts.md`
- [`STATUS.md`](../../../docs/future-work/02-bdd-testing-evolution/STATUS.md) — BDD-* implementation matrix
- `shared/scripts/protect-tests.sh` — PreToolUse hook reference implementation
