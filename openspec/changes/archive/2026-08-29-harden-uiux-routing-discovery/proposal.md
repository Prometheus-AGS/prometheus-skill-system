## Why

The injected UI/UX workflow can point Impeccable at a future path that does not exist and mandates an `ux-designer` catalog entry that is neither installed nor verifiably supplied by Anthropic. Agents need deterministic existing-target resolution and a truthful capability fallback before UI work begins.

## What Changes

- Require an existing incumbent implementation target before bounded Impeccable context loading.
- Treat proposed future paths as design destinations rather than context roots.
- Consult `ux-designer` only when it is present in the active catalog.
- Route missing UX-review capability to installed UI/UX Pro Max plus `frontend-design` and require the fallback to be recorded.
- Update the roster, injector documentation, and focused managed-fence tests.

## Capabilities

### New Capabilities

- `uiux-agent-routing`: Existing-target-first, capability-aware routing for UI/UX skills and Impeccable context.

### Modified Capabilities

None.

## Impact

Affects the `uiux-routing` injected rules pack and its documentation/tests. The UAR source tree is read-only evidence for this change and is not modified.
