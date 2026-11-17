---
name: bdd-lifecycle-loop
license: MIT
description: >
  Codifies the VLBV define → author → execute (once, at the value-phase
  boundary) → triage → maintain workflow for BDD suites. Includes outside-in
  scenario authoring, the stochastic profile for nondeterministic claims (no
  retry-until-green), an immutable-tests guard, and a visual-baseline refresh
  workflow with paper trail. Use when standing up a BDD suite, growing one past
  100 scenarios, or when nondeterminism starts eroding trust in results.
metadata:
  version: '2.0.0'
  author: prometheus-skill-pack
  category: testing
  tags: "testing, bdd, cucumber, lifecycle, vlbv, stochastic, immutable-tests"
---

# BDD Lifecycle Loop

A repeatable workflow for the life of a BDD suite under VLBV, the pack's single
verification method. Pairs with `bdd-cucumber-js` and `bdd-cucumber-rs` (which
cover *how to author*), `value-scenario-verify` (which *executes*) and
`bdd-video-proof` (which covers *how to certify*). Boundaries are defined once in
`skills/process/kbd-process-orchestrator/references/boundaries.md`.

## Definition, authoring and execution are three different acts

| Act | When | Skill |
|---|---|---|
| **Define** scenarios (actor, outcome, source, oracle, negative control) | Before generation, from the specification; adopted into the phase contract | `value-scenario-discovery` |
| **Author** features, real assertions and steps | After the complete agreed production work (static inspection only) | `bdd-cucumber-js` / `bdd-cucumber-rs` `author-only` |
| **Execute** against the real path | Only at B4 (the boundary packet, once) and an authorized B5 repair | `value-scenario-verify` |

Defining a scenario does not authorize running anything. Do not run a scenario
against an intentionally incomplete codebase, per change, per task or per edit. A
scenario that has never run is `NOT_EXERCISED` — never "red", never "green". A
different execution cadence exists only as a signed `KBD-METHOD` deviation for one
phase, disclosed in its closure packet. No step here modifies a protected scenario
without its existing signed approval.

## The loop

```
      ┌──────────────┐
      │  1. Author   │  Features → real assertions → step definitions (no runs)
      └──────┬───────┘
             ↓
      ┌──────────────┐
      │  2. Execute  │  B4 step 3: the boundary packet runs each scenario once
      └──────┬───────┘
             ↓
      ┌──────────────┐
      │  3. Triage   │  PASS / FAIL / BLOCKED / NOT_EXERCISED — never rerun to green
      └──────┬───────┘
             ↓
      ┌──────────────┐
      │  4. Maintain │  Refresh baselines, prune stale scenarios, review
      │              │  stochastic declarations, enforce immutable tests
      └──────┬───────┘
             ↓ (next value phase)
```

## Phase 1: Author (outside-in)

Work outside-in: begin from the observable user outcome, not from step code that
mirrors the implementation. Write the features, assertions and oracle mapping
from the adopted scenario IDs, and stop after static inspection.

Each assertion must be real: it observes the production operation's externally
meaningful result against an independent source of expected truth. A step that
only returns, or asserts a process exit code, is not an assertion. Each scenario's
negative control must make that assertion fail when the behavior is absent.

### Feature file conventions

- One feature per file. One behavior per scenario.
- Declarative language: "signs in with valid credentials", not "clicks button X and types Y"
- Use `Background` for shared preconditions across scenarios in a file
- Use `Scenario Outline` for data variations
- Tag every scenario:
  - Contract identity: the adopted scenario ID (for example `@SCN-AUTH-001`)
  - Layer: `@api`, `@ui`, `@system`
  - Ownership: `@team:auth`, `@owner:alice`
- Reference `data-testid` selectors when the scenario is `@ui`

## Phase 2: Execute (B4 step 3)

The adopted boundary packet lists the exact commands; `value-scenario-verify`
invokes each once. The cucumber runner writes `cucumber.json` beside any
human-readable report; the packet's receipts and `boundary-results.json` read it.

```bash
# cucumber-js (as a boundary-packet entry)
npx cucumber-js --format json:tests/reports/cucumber.json

# cucumber-rs (as a boundary-packet entry)
cargo test --test features -- --format=json > tests/reports/cucumber.json
```

## Phase 3: Triage

Every scenario ends with exactly one of the four verdicts. Nothing is rerun to
obtain a different one.

| Observation | Verdict and action |
|-------------|--------|
| Assertion caught a genuine behavior difference | FAIL. Keep the receipt. The owner may authorize a bounded B5 repair; the rerun covers only affected scenarios and consumes `repairAttempts` |
| Timeout, race or environment fault the contract did not anticipate | BLOCKED (not FAIL). Record the cause; fixing the fixture or environment is a B5 repair or a contract amendment |
| Runner, fixture, permission or isolation unavailable | BLOCKED |
| Adopted scenario never ran | NOT_EXERCISED; it cannot pass |
| Result differs between identical runs | Not a verdict: the scenario is nondeterministic and needs a declared stochastic profile (below). Until then it is BLOCKED |

A later green receipt never overwrites an earlier failure.

### Nondeterministic claims: the stochastic profile

There are no `@flaky` tags, retry flags or flake budgets: retrying until green
hides failures and changes the denominator. A behavior that is genuinely
nondeterministic (model output, timing-dependent UI, distributed timing) is
declared **before any result exists**, in the scenario's `stochastic` block of the
adopted contract (schema v2):

```json
"stochastic": {
  "samples": 20,
  "grader": "fraction of samples whose observation meets the oracle",
  "stoppingRule": "run all 20; PASS iff >= 18 meet the oracle; no early stop"
}
```

The boundary packet then runs the declared number of samples, once, and the verdict
follows the stopping rule. Adoption lint rejects a scenario marked nondeterministic
that has no `stochastic` block.

## Phase 4: Maintain

- **Stochastic declarations.** Review each `stochastic` block when its scenario's
  amendment history or the closure packet shows drift. Changing samples, grader or
  stopping rule is a signed contract amendment, never an edit after results.
- **Pruning.** Retire scenarios through a contract amendment so history stays.

### Immutable tests

The **immutable-tests rule** is enforced once, at final local certification.
Agent-time Bash, Python, Edit and Write calls remain unrestricted. The
method-independent verifier compares the certified base and candidate Git
commits and detects protected modifications, deletions, renames and mode
changes. Intentional changes require an SSH-signed canonical approval manifest;
see [references/immutable-tests.md](references/immutable-tests.md).

### Visual baseline refresh

Playwright's `--update-snapshots` overwrites baselines silently. Coupled with the
immutable-tests rule, baseline changes need a paper trail. See
[references/visual-baseline-refresh.md](references/visual-baseline-refresh.md)
for the branch-and-review workflow.

## See also

- [value-scenario-discovery](../value-scenario-discovery/SKILL.md) — define scenarios before generation
- [value-scenario-verify](../value-scenario-verify/SKILL.md) — execute at the value-phase boundary
- [vlbv](../vlbv/SKILL.md) — the method entry skill
- [bdd-cucumber-js](../bdd-cucumber-js/SKILL.md) — how to author TS scenarios
- [bdd-cucumber-rs](../bdd-cucumber-rs/SKILL.md) — how to author Rust scenarios
- [bdd-video-proof](../bdd-video-proof/SKILL.md) — certification bundles
- [references/immutable-tests.md](references/immutable-tests.md)
- [references/visual-baseline-refresh.md](references/visual-baseline-refresh.md)
