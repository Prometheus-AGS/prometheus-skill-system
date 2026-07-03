---
id: change-credibility-010-bdd-tests
title: Add BDD feature files and step definitions for forge validate and enrich
phase: phase-credibility-closure
priority: P2
effort: M
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-C
verdict: ADOPT
library: "@cucumber/cucumber@^11 (npm, MIT)"
scope:
  - tests/features/forge-validate.feature
  - tests/features/forge-enrich.feature
  - tests/steps/forge-steps.ts
  - package.json
---

# change-credibility-010 — Add BDD feature files and step definitions

## Context

The skill-pack has no behavioral tests for the forge-rs commands. BDD feature files give external reviewers a human-readable specification of what the system does and a CI gate that catches regressions.

The tests use an offline fixture approach: they invoke the `forge` CLI binary directly using `child_process.spawn`, with fixture inputs pre-committed under `tests/fixtures/`. No running daemon is required.

## Scope

1. Add `@cucumber/cucumber@^11` and `ts-node` devDependencies (npm, MIT)
2. Create `tests/features/forge-validate.feature`
3. Create `tests/features/forge-enrich.feature`
4. Create `tests/steps/forge-steps.ts` with step definitions
5. Add `cucumber` npm script to `package.json`

## Implementation Notes

`tests/features/forge-validate.feature`:
```gherkin
Feature: forge validate
  As a developer
  I want to validate a source file against a constitution
  So that style violations are caught before review

  Scenario: Clean file with no constitution passes silently
    Given a Rust source file "clean.rs" with valid content
    And no constitution exists
    When I run "forge validate clean.rs --language rust"
    Then the exit code is 0
    And the output contains "Validation complete"

  Scenario: File violating constitution exits 1
    Given a Rust source file "violating.rs" with a known violation
    And a constitution that forbids that pattern
    When I run "forge validate violating.rs --language rust"
    Then the exit code is 1
    And the output contains the violation rule name
```

`tests/features/forge-enrich.feature`:
```gherkin
Feature: forge enrich
  As a developer
  I want to enrich a task description with relevant skills
  So that my agent context is grounded

  Scenario: Enrich produces JSON output with skills key
    Given a task file "task.md" with content "implement a REST API"
    And the project root has a skill directory
    When I run "forge enrich" with task "implement a REST API" for language "rust"
    Then the exit code is 0
    And the output is valid JSON
    And the JSON contains a "skills" array

  Scenario: Enrich with path traversal attempt returns error
    When I run forge enrich with task_path "../../../etc/passwd"
    Then the exit code is non-zero
    And the output contains "outside the project root"
```

`tests/steps/forge-steps.ts` — use `child_process.spawnSync` to invoke the `forge` binary from `tools/forge-rs/target/debug/forge` or `FORGE_BIN` env, with fixture working directories under `tests/fixtures/`.

## Verification

- `npm run cucumber` passes all scenarios
- The path traversal scenario confirms exit code is non-zero
- No running daemon is required
