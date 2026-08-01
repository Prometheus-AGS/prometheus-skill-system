# Every builtin skill reaches the database on every provider

**Change:** `change-uhe-008-builtin-db-registration`
**Phase:** uar-host-execution
**Goal:** R1

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: R1 **PARTIAL** — 2 of 3 providers verified, the third named

### The defect, measured before it was fixed

`SkillRegistry::register` gated persistence on **both** a database and a
`VectorMatcher`:

```rust
if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
```

A host with a database but no embedder matched nothing and persisted nothing —
**silently**, because a tuple pattern that fails to match is not an error. The
embedded case has no embedding service by definition, so **the platform R1
exists for was exactly the one that never reached the database.**

Test written first, and it failed:

```
builtin rows in the database (0) != skills the loader discovered (3)
```

The doc comment on `register_builtins` said "these skills are not persisted via
storage providers", which reads as an intentional design note. It described a
bug.

### The fix

Persistence and embedding are now independent: persist whenever a database
exists; attach an embedding when one can be produced. Both `None` (no matcher)
and `Err` (matcher failed) degrade to persisting **without** an embedding rather
than to dropping the skill — an embedding is an enrichment for vector search, not
a precondition for the skill existing.

### Provider results — equality, not "some rows exist"

| Provider | Result | Evidence |
|---|---|---|
| **memory** | ✅ VERIFIED | 3 rows == 3 discovered; plus an exact-set test so over-registration cannot satisfy it |
| **surreal** (embedded, **default backend**) | ✅ VERIFIED | 3 rows == 3 discovered against file-backed SurrealKV, **no server** |
| **postgres** | ⛔ **BLOCKED** | prerequisite named below |

**Revert-verified:** with the old gating restored, the surreal test fails
`0 != 3`. The test catches the defect rather than passing by construction.

### Two would-be shortcuts that measurement refused

1. **"Surreal needs a live server → BLOCKED."** It does not. UAR compiles
   `kv-surrealkv`, so the embedded engine runs in-process. Recording BLOCKED
   would have left **R1's own platform unproven** behind a tidy status. It also
   is not `"memory"` → `mem://`: that engine is not compiled in and fails with
   `Unsupported scheme: memory`. The embedded store must be file-backed.
2. **"Postgres has no server → BLOCKED."** One **is** live on `127.0.0.1:5432`
   and accepts connections. The real blocker is narrower and was found only by
   attempting the install.

### Postgres: the actual prerequisite

`migrations/20251225000000_init_uar.sql:2` requires `CREATE EXTENSION vector`.

- `brew install pgvector` **succeeded** — but ships only `share/postgresql@17`
  and `share/postgresql@18`.
- The running server is **16.14**. A PG17-built `.so` in a PG16 install is an
  ABI mismatch, not a fix.
- Only `postgresql@14/@15/@16` are installed locally.

**To unblock:** `pgvector` built for PostgreSQL 16, or a PostgreSQL 17+ server
with the bottled extension. Standing up a new major version and migrating is out
of scope for this change.

### Correction made mid-change

I assumed `memory` was the embedded case. `Cargo.toml` says
`default = ["minimal"]` → `surreal-backend`, "SurrealKV-backed, fully embedded —
no external services required", while `in-memory-backend` is off by default and
documented "do not use when state must survive process restart". **`surreal` is
the embedded path**, and the test file records the correction rather than quietly
adopting it.

**R1 is reported PARTIAL, not MET** — per the acceptance criteria, one unverified
provider means the goal is not met, and the gap is named rather than dropped.

## The fix nearly introduced a second silent data-loss bug

Caught by asking who else calls the function I changed, rather than stopping at
a green test.

`SkillRegistry::register_all` routes through `register`, and **`register_all` is
what `initialize()` and `refresh()` call on skills they just read from a
provider via `list_skills()`**. Making `register` persist therefore made the
*load* path write back.

That is not harmless. `save_skill` upserts:

```sql
ON CONFLICT (skill_id) DO UPDATE SET … embedding = EXCLUDED.embedding
```

So a host with a database but **no embedder** — the case this whole change
exists to support — would overwrite every stored embedding with an empty vector
**on every restart**, silently degrading vector search on a database that was
previously healthy. The repair for one silent data-loss bug would have shipped
another.

**Fix:** load and save are now separate operations. `index_only` / `index_all`
index without persisting, and both load paths (`initialize`, `refresh`) use
them. `register` / `register_all` persist, and are for skills genuinely entering
the system.

### The first regression test I wrote was worthless

I initially asserted row counts against `InMemoryProvider`. That provider's
`save_skill` signature is `_embedding: &[f32]` — it **discards the embedding
entirely**, so it cannot observe a clobber. The test would have passed whether
or not the bug was present, while looking like protection.

Replaced with a counting double that asserts the load path issues **zero**
`save_skill` calls — the behaviour that actually differs.

**And the replacement was vacuous too.** It called `initialize()` on a service
with **no providers attached**. `initialize` iterates `self.providers`; with an
empty list, `list_skills()` is never called, the load path never runs, and the
zero-writes assertion holds **even with the bug present**.

Two vacuous tests in a row for the same underlying reason: I asserted on a
*result* without checking the code path that produces it had executed. The
version that ships attaches a real `SkillStorageProvider` yielding a skill, and
adds a second assertion that the skill actually reached the registry — so "zero
writes" cannot be satisfied by "nothing was read".

## Environment hazard hit during verification

Two test runs appeared to "compile" for ~10 minutes each while producing a
**0-byte** output file. They were not compiling. `ps` showed the cargo process at
**0.0% CPU with zero `rustc` workers**, holding
`/Volumes/my-passport/cargo-build/…/debug/.cargo-build-lock`.

`~/.cargo/config.toml` routes intermediate artifacts to a shared build root, so
**cargo invocations from unrelated projects serialise against each other**. Four
`librefang` processes had been idle at 0.0% CPU since 03:36 and 03:47 — roughly
45 minutes — holding the lock this run needed.

**Diagnostic that settles it in one command:** a stalled build has *zero* `rustc`
children. Elapsed time and an empty log look identical to slow progress; process
state does not.

**Also learned the expensive way:** `pkill -f 'cargo test …'` matched **my own
run** as well as the strays, killing it (exit 144). Kill by explicit PID after
listing them, never by pattern, when your own process matches the same pattern.

## POSTGRES UNBLOCKED — and it found two real defects

The BLOCKED verdict was not bureaucratic completeness. Unblocking the third
provider surfaced **two bugs the other two providers structurally could not
detect.**

### How the block was resolved

Reused the **PG18 + pgvector image already proven in `flint-forge`**
(`docker/postgres/Dockerfile`) rather than re-solving Homebrew's packaging.
Verified live: PostgreSQL **18.4**, pgvector **0.8.5**.

That image had already solved a trap: it pins `PGVECTOR_REF=v0.8.5` because
**0.8.0 does not compile on PG18** — pgvector called `vacuum_delay_point()` with
no arguments while PG18 changed the signature. Its own comment records this.
Reusing a solved problem beat re-deriving it.

### Defect 1 — the embedder-optional fix was wrong on Postgres

The column is `vector(384)`. Persisting an **empty** vector when no embedder
exists is rejected outright:

```
ERROR:  vector must have at least 1 dimension
```

`SkillRegistry::register` *logs* persist failures without propagating them, so
the skill vanished silently: **0 rows in Postgres while memory and SurrealDB
held all 3.**

Neither passing provider could catch this. `InMemoryProvider::save_skill` takes
`_embedding` and **discards it**; SurrealDB does not enforce vector dimensions.
The bug was only reachable on the provider that validates.

**Fixed:** empty slice → SQL `NULL`, the correct representation of "not embedded
yet". The column is already nullable, and vector search simply does not match the
row until something backfills it.

### Defect 2 — pre-existing: a trigger broke EVERY skills insert

Independent of this change:

```
ERROR:  record "rec" has no field "id"
CONTEXT:  PL/pgSQL assignment "row_id := rec.id::text"
```

`uar_notify_entity_change()` assumes every table has an `id` column. `skills` is
keyed by `skill_id`. The trigger fires AFTER INSERT, so **the entire statement
aborted** — no skill could ever reach Postgres, with or without this change.

**Fixed** in migration `20260801000000_notify_entity_change_key_agnostic.sql`:
resolve the row key generically (`id` → `%_id` → the catalogue's real primary
key, read via `to_jsonb(rec)` because PL/pgSQL cannot do dynamic field access).

It also gains an exception handler, which is the more important change:
**a notification is a side channel. Losing a notify is a degraded feature;
losing the row is data loss.** The trigger must never abort the write.

### The lesson

Two providers passing is not two-thirds of the evidence — it is evidence from the
two providers that *cannot* fail this way. A third backend was the only thing
that could surface either bug, and both were silent: one logged-and-swallowed,
one aborting a statement nobody was watching.

## R1 is now MET — all three providers verified

```
test result: ok. 5 passed; 0 failed; 0 ignored
```

| Provider | Result |
|---|---|
| memory | ✅ 3 == 3 |
| surreal (embedded, default backend) | ✅ 3 == 3 |
| **postgres** | ✅ **3 == 3** on PG18.4 + pgvector 0.8.5 |

Confirmed in the database directly, not only via the assertion:

```
uhe008-pg-alpha|builtin|t
uhe008-pg-beta |builtin|t
uhe008-pg-gamma|builtin|t
```

`origin=builtin`, `embedding IS NULL` — the NULL is correct, not a shortfall: no
embedder was configured, and NULL is what "not embedded yet" means.

**R1 upgraded from PARTIAL to MET.** Nothing is left named-but-unverified.
