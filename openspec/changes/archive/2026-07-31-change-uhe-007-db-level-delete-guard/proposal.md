# Builtin skills cannot be deleted, even bypassing the service

**Change:** `change-uhe-007-db-level-delete-guard`
**Phase:** uar-host-execution
**Goal:** R2

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: guard proven at SQL; the Rust test is written but **cannot be run here**

### What is proven

The bypass route is closed. Running **`postgres.rs:151` verbatim** — the exact
statement `DatabaseStorageProvider::delete_skill` issues, with no `SkillService`
anywhere in the path:

```
DELETE FROM skills WHERE skill_id = 'u1';   -- user    -> DELETE 1
DELETE FROM skills WHERE skill_id = 'b1';   -- builtin -> ERROR: system_skill_immutable
```

Against Postgres 18.4, with the migration applied through the same file a real
deployment applies. A caller reaching the storage provider directly is refused
by the database.

**Task 3 also holds, and better than expected.** `service.rs:383` raises
`system_skill_immutable`, and `api/skills.rs:283` maps that string to HTTP 409.
Because the DB trigger raises the **same string**, a refusal originating in
Postgres surfaces to the API as 409 too — the mapping did not need changing.

### What is NOT proven, and why

`tests/builtin_delete_guard.rs` is written and syntactically valid, and it does
the right thing: it calls `PostgresProvider::delete_skill` directly, asserts the
error names `system_skill_immutable`, asserts the row survives, and asserts a
builtin can still be **disabled**.

**It has never executed.** `cargo check --lib` fails in this repo on a
dependency, before any of my changes:

```
error[E0599]: no associated function or constant named `from_bytes_stream`
              found for struct `SseStream<B>`
error: could not compile `rmcp`
```

Verified pre-existing by checking out `563ecc2` — the merge commit **before** my
first change — where it fails identically. `rmcp` is pinned `=2.2.0` while
`sse-stream` resolved to 0.2.3, which removed the API it calls.

**So task 2 is recorded as PARTIAL, not complete.** The guarantee is
demonstrated at the layer that enforces it, and the test that would prove it
from Rust is committed and ready — but "the test passes" is not a claim I can
make, and marking it done would assert something unverified.

### Follow-up

Fixing the `rmcp`/`sse-stream` mismatch is out of this change's scope (it is a
dependency-resolution problem affecting the whole crate, not the delete guard).
Carried to the phase reflection so it is not lost. Once the crate builds,
`cargo test --test builtin_delete_guard --features postgres-backend` with
`DATABASE_URL` set completes this task.
