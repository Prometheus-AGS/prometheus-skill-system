---
name: bdd-lifecycle-loop
version: '1.1.0'
license: MIT
description: >
  Codifies the define → author → run → triage → maintain workflow for BDD
  test suites, with execution only at the authorized phase boundary.
  Includes outside-in scenario authoring, flake-budget enforcement wrapping
  cucumber's --retry-tag-filter primitive, an immutable-tests guard, and a
  visual-baseline refresh workflow with paper trail. Use when
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

## Definition, authoring and execution are three different acts

| Act | When | Skill |
|---|---|---|
| **Define** scenarios (actor, outcome, source, oracle, false positive) | Before generation, from the specification | `value-scenario-discovery` |
| **Author** features, real assertions and steps | Per host policy; where production is implemented first, only after the complete agreed production work | `bdd-cucumber-js` / `bdd-cucumber-rs` `author-only` |
| **Execute** against the real path | Only at the separately authorized complete value-phase boundary | `value-scenario-verify` |

Defining a scenario does not authorize running anything. Where the repository
requires implementation first (for example Prometheus: complete the whole
agreed production batch, then run one integration gate), the phases below
describe the loop *after* that boundary; do not run Phases 2-3 against an
intentionally incomplete codebase, per change, per task or per edit. A
selected method or host policy always outranks this loop's defaults, and no
step here repeals an operator rule or modifies a protected scenario without its
existing signed approval.

## The loop

```
      ┌──────────────┐
      │  1. Author   │  Features → real assertions → step definitions
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

Work outside-in: begin from the observable user outcome, not from step code
that mirrors the implementation. Whether the scenario is expected to fail
first depends on the declared timing policy:

- **Continuous-testing projects** may write the feature, observe it fail for
  the right reason, then implement the smallest slice and watch it pass.
- **Production-first or selected-VLBV projects** (including this repository)
  write the features, assertions and oracle mapping from the adopted
  scenario IDs, but do **not** run them while production is incomplete. A
  scenario that has never run is `NOT_EXERCISED`, never "red" and never
  "green". Use `author-only` and stop after static inspection.

In either policy each assertion must be real: it observes the production
operation's externally meaningful result against an independent source of
expected truth. A step that only returns, or asserts a process exit code, is
not an assertion.

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

Run only at the authorized boundary (see the table above); under production-first
policy that is the single complete-phase gate, never a per-change loop.

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

After the complete production implementation, run locally as part of the
repository's final integration gate:

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/testing/bdd-lifecycle-loop/scripts/flake-budget.sh" \
  tests/features \
  .bdd-flake-budget.json
```

Exits non-zero when:
- More than `max_flaky_scenarios` are tagged `@flaky`, OR
- Any `@flaky` scenario has been tagged longer than `max_flaky_age_days`
  (unless listed in `grace_scenarios`)

A failing local gate requires a recorded disposition: fix the production
cause or propose an approved scenario/grace-budget change with a linked ticket.
Protected scenarios cannot be deleted or weakened to obtain a pass; their
changes require the repository's signed approval. Hosted CI is not validation
evidence for Prometheus repositories.

### Immutable tests

The **immutable-tests rule** is enforced once, at final local certification.
Agent-time Bash, Python, Edit, and Write calls remain unrestricted. The
method-independent verifier compares the certified base and candidate Git
commits and detects protected modifications, deletions, renames, and mode
changes. Intentional changes require an SSH-signed canonical approval
manifest; see [references/immutable-tests.md](references/immutable-tests.md).

### Visual baseline refresh

Playwright's `--update-snapshots` overwrites baselines silently. Coupled
with the immutable-tests rule, baseline changes need a paper trail. See
[references/visual-baseline-refresh.md](references/visual-baseline-refresh.md)
for the branch-and-review workflow.

## See also

- [value-scenario-discovery](../value-scenario-discovery/SKILL.md) — define scenarios before generation
- [value-scenario-verify](../value-scenario-verify/SKILL.md) — execute at the authorized phase boundary
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
- `scripts/verify-protected-tests.mjs` — canonical local verifier
