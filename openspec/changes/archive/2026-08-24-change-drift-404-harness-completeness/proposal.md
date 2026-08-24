## Why

`.windsurf/skills` reached zero files and nothing noticed. No gate in this repository
asserts that a declared harness still has a tree, so an external generator can drop one
silently — which is how this phase's 98-file tree came to include a 20-file deletion nobody
decided on. HMA's c300 built exactly this assertion after the same class of silent loss.

## What Changes

- Add a per-harness completeness assertion, deriving the harness set from
  `skill-system.json` rather than hardcoding it.
- Answer, with evidence, whether this repo needs a NORMALIZER as well. Unlike HMA it
  enforces no `internal: true` invariant, so there may be nothing to re-apply — if so,
  record that and ship the assertion alone rather than inventing an invariant.

## Impact

- Depends on c400-c403: the assertion must encode the SETTLED harness set, and c401 may
  change it.
- C-04 (idempotent), C-05 (bash 3.2 if shell), C-03 (document the check).
