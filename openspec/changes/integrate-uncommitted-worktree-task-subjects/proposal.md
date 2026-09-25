# Proposal

## Why

The operator requested integration of uncommitted work in the mini and full skill-pack worktrees. The only dirty secondary worktree contains a qualified-task patch using `change::task`; current main already supports slash and single-colon forms.

## What Changes

- Preserve the original worktree patch in Git and integrate its missing double-colon syntax into full-pack main.
- Retain existing separators, ambiguous-task rejection, and unrelated main-checkout changes.
- Record the worktree inventory and local CLI integration result.

## Capabilities

### New Capabilities
- `kbd-qualified-task-subjects`: compatible task selection with change-qualified subjects.

### Modified Capabilities
None.

## Impact

Only the full-pack CLI task guard parser and its process-level integration coverage. No dependencies, runtime schema, services, UI, or security policy changes. Mini secondary worktrees are clean. Already committed branch work is reported separately from the requested uncommitted work.
