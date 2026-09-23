## Why

The cross-platform installed-skill verifier discovers `SKILL.md` files below
test fixture directories even though the production installers deliberately
exclude those trees. MiniMax correctly omitted two adversarial-review fixtures,
but verification reported them as missing installed skills and blocked release
closeout.

## What Changes

- Apply the installers' existing `tests` and `fixtures` directory exclusion to
  installed-skill verification.
- Add an isolated regression that runs the production verifier against a
  temporary repository and MiniMax home.

## Capabilities

### Modified Capabilities

- `installed-surface-verification`: require verifier discovery to match the
  production installer contract for non-product fixture trees.

## Impact

This changes only installed-skill discovery and its deterministic test. It does
not change any installed skill payload or include test fixtures in a tool
catalog.
