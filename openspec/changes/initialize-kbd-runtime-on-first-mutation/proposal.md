## Why

A registered KBD project can have a valid immutable identity and replica entry but no first signed runtime event. In that state every typed mutation fails with `runtime has not been initialized`, while status recommends an unrelated migration command whose own safety comments document prior loss of projection-only work.

## What Changes

- Initialize an empty registered runtime automatically when the first typed mutation needs canonical state, using the existing legacy-aware initialization path.
- Keep read-only status non-mutating and replace the speculative `migrate --apply` recommendation with accurate automatic-initialization guidance.
- Include the canonical runtime path in initialization failures so operators can identify the affected state.
- Add process-level CLI coverage proving successful first mutation and non-zero exit for a rejected typed command.
- Preserve registration semantics, legacy projections, signing authority, local fallback behavior, and migration safety.

## Capabilities

### New Capabilities

- `kbd-runtime-initialization`: Defines the boundary between project registration and the first signed runtime event, automatic first-mutation initialization, actionable status, and CLI failure semantics.

### Modified Capabilities

None.

## Impact

The change affects the safe legacy-import state refresh in `substrate/kbd-runtime/src/lib.rs`, `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs`, focused runtime/CLI tests, and the KBD runtime/CLI OpenSpec contract. It adds no dependency, daemon route, registry, or journal format change.
