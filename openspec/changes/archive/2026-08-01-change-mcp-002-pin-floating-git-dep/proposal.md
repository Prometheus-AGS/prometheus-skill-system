# Pin the floating rmcp git dependency

**Change:** `change-mcp-002-pin-floating-git-dep`
**Phase:** uar-host-execution / mcp-2026-07-28-adoption (child)

## Why

See `.kbd-orchestrator/phases/uar-host-execution/children/mcp-2026-07-28-adoption/plan.md`
for full rationale, acceptance criteria, and the adversarial review record.

## Outcome: AUTHORISED and fixed

The authorisation gate was the first task, and **silence was not assumed**. The
user was asked and granted the fix explicitly, **narrowly**: this one line only,
declining the broader "authorise the whole repo" option. Recorded in
[`evidence/authorisation-surreal-memory-server.md`](../../../.kbd-orchestrator/phases/uar-host-execution/children/mcp-2026-07-28-adoption/evidence/authorisation-surreal-memory-server.md).

### The defect, reproduced before fixing

```console
$ cargo check                    # Finished — looks healthy on the stale lock
$ rm Cargo.lock && cargo check   # 2 errors
error[E0432]: unresolved import `rmcp::model::Content`
error[E0639]: cannot create non-exhaustive struct using struct expression
```

One `cargo update` from breaking, while appearing fine.

### After

```console
$ rm Cargo.lock && cargo check --lib
    Finished `dev` profile in 58.71s
```

`a64be231` is what the committed lockfile already resolved to, so **nothing
changes today** — the pin only stops the drift.

### A trap avoided

Deleting `Cargo.lock` and letting cargo regenerate produced a **979-line diff
dropping 31 packages** — an unrelated dependency sweep smuggled into a one-line
fix. Used `cargo update -p rmcp` instead: **11 lines**, only rmcp's source URL
gaining `?rev=`. The acceptance test still passes against that minimal lock, so
the smaller diff costs nothing.

### Deliberately not fixed

This crate's own `rmcp::model::Content` usage — the reason a newer rmcp breaks
it at all. That is a port needing its own decision, and the grant covered the
dependency edge, not the source.
