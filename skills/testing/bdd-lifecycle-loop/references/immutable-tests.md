# The Immutable-Tests Rule

**Code-generation agents may not edit existing test files to make failing
tests pass.** They may create new tests (in `tests/features/drafts/`) and
new step definitions, but they may not modify existing `tests/steps/*`,
`tests/support/*`, or non-draft `tests/features/*` files.

This is the operative form of [BDD-006](../../../docs/future-work/02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md).

## Why

The original ask — "tests should be automatically updated by
code-generation agents" — is a category error. Tests express what the
system *should do*. Production code expresses what it *does*. When those
disagree, only one side gets to move.

If agents can rewrite tests, tests become a rubber stamp. A failing test
means the agent will make the assertion pass, not that the agent will
implement the missing behavior. Over time, tests describe whatever the
current implementation happens to do — which is worthless.

## Two enforcement layers

### 1. PreToolUse hook (agent-time)

`shared/scripts/protect-tests.sh` matches `Edit` and `Write` tool calls
against a protected-path list and blocks them.

Configure in the target project's `.claude/hooks.json`:

```json
{
  "PreToolUse": [
    {
      "matcher": "Edit|Write",
      "command": "bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/protect-tests.sh"
    }
  ]
}
```

The hook receives the tool call JSON on stdin, checks
`tool_input.file_path` against the protected paths, and exits 2 (blocking)
when there's a match.

### 2. CI gate (PR-time)

`scripts/test-file-diff-guard.sh` runs in CI and fails PRs that touch
protected paths without a `test-change-approved` label. Example GitHub
Actions step:

```yaml
- name: Immutable-tests guard
  run: |
    bash "${CLAUDE_PLUGIN_ROOT}/skills/testing/bdd-lifecycle-loop/scripts/test-file-diff-guard.sh" \
      origin/${{ github.base_ref }} HEAD
```

## What agents are allowed to do

- **Author new features under `tests/features/drafts/`.** These are
  candidate scenarios — reviewed manually, then promoted out of `drafts/`.
- **Author new step definitions** in NEW step files (not existing ones).
- **Extend production code** to make existing tests pass.
- **Refuse a task** when a failing test appears to encode a bad
  requirement, and surface the disagreement to the human.

## What agents may NOT do

- Edit `tests/steps/*.ts` / `tests/steps/*.rs`
- Edit non-draft `tests/features/**/*.feature`
- Edit `tests/support/**/*`
- Delete a test to make a suite "pass"
- Change an assertion's expected value

## Overriding the rule (human review only)

Two escape hatches:

1. `test-change-approved` label on the PR (visible in the review UI —
   requires a human to add it)
2. `BDD_ALLOW_TEST_EDITS=1` environment variable (local runs only; not
   set in CI)

Both are audit-visible. Silent bypasses are impossible.

## Related

- [BDD-005: testid drift detection](../../../docs/future-work/02-bdd-testing-evolution/BDD-005-testid-drift-detection.md) — pairs with this rule
- [BDD-007: candidate test drafts](../../../docs/future-work/02-bdd-testing-evolution/BDD-007-candidate-test-drafts.md) — the `drafts/` workflow
- `shared/scripts/protect-tests.sh` — PreToolUse reference implementation
