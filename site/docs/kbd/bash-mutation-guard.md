---
id: bash-mutation-guard
title: Tool Guards
sidebar_label: Tool Guards
---

# Tool Guards

:::info The Bash mutation fence was removed
Earlier versions gated `Bash`, `Write`, `Edit`, and `MultiEdit` behind a
pre-mutation fence that checked KBD project identity, control-plane
reachability, and lifecycle state. That fence and the protected-test
`PreToolUse` hook no longer exist. **Bash, Python, Write, and Edit are not
gated.**
:::

## What still guards a tool call

| Matcher | Script | What it blocks |
|---|---|---|
| `Write\|Edit\|MultiEdit` | *(none)* | Nothing |
| `Bash` | *(none)* | Nothing |
| `Python` | *(none)* | Nothing |

Protected BDD integrity is checked at final local certification by
`scripts/verify-protected-tests.mjs`. It compares committed Git states, so it
detects content changes, deletion, rename, or mode changes regardless of which
tool caused them. Intentional changes require an SSH-signed canonical approval
manifest. Missing approval does not block creative work; it fails certification.

## Why the fence was removed

The fence attempted to arbitrate contention between several agents by blocking
the operator's own tools. It protected against a failure mode that was not
occurring while imposing one that was.

Every gate failed closed. A stopped daemon, an uninitialized runtime, or a
phase that had merely *finished* would deny `ls`, `git status`, and
`cargo test`. Because the fence covered all of `Bash` rather than only mutating
commands, the denial also removed the diagnostics each error message
recommended, and the recovery commands documented for that state. Finishing a
phase — an ordinary, successful outcome — left the runtime in `completed` and
disabled the shell with no in-band way out.

The scope guards (`scope-guard.sh`, `check-child-scope.sh`) compounded it by
flagging writes outside a change's declared `scoped_paths`. Editing a submodule
or a sibling project that the current change depends on is ordinary work, and it
was flagged every time.

## What the KBD adapter still does

`shared/scripts/kbd-harness-adapter.sh` runs on four observational events:

| Event | Behavior |
|---|---|
| `session_start` | Renders the bounded re-anchor block (committed revision, lifecycle, active path, next work) |
| `post_compact` | Same re-anchor after context compaction |
| `prompt` | Queues a deferred event for the daemon |
| `stop` | Queues a deferred event; advisory only |

All four exit `0` unconditionally. None can refuse a tool call. The generated
adapters for Codex, OpenCode, and Kimi carry the same event set — verified by
`scripts/check-harness-adapters.js` and by `prometheus doctor`'s
`hooks.harness-adapters` check.

## What is unchanged

Phases, `progress.json`, `current-waypoint.json`, `position-reminder.txt`, and
reflections all still work. They record position so a session — or a different
harness entirely — can resume with one read. That bookkeeping never blocked
anything, and it is the part that carries its weight.

Lifecycle state remains committed to the event journal and visible through
[`prometheus kbd status`](./operator-controls) and the
[REST API](/docs/sovereign-sync/rest-api). Command concurrency is enforced at
the journal transaction boundary, not by intercepting shell tools.
