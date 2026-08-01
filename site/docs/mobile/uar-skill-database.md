---
id: uar-skill-database
title: Skills in the UAR Database
sidebar_label: UAR Skill Database
---

# Skills in the UAR Database

The Universal Agent Runtime keeps a skill database that the admin UI, the REST
API, and embedded hosts all read. This page documents how pack skills get there,
the guarantees around them, and two silent-data-loss bugs found and fixed in the
process — both of which are instructive beyond this codebase.

## Requirements

| # | Requirement |
|---|---|
| R1 | Skills install and are recognised in the UAR skill database **on every platform**, embedded or not |
| R2 | Pack skills **can never be deleted** — only turned off or excluded |
| R3 | The UAR UI shows these skills and can administer them |
| R4 | UAR exposes skill installation/query REST APIs and an embedded SDK, including optional DB registration for dynamically created skills |
| R5 | Know when skills need updating, initiate it from the GitHub repository, and support updating **for mobile use** |

## How builtin skills reach the database

At startup UAR discovers pack skills from the active skill-pack root and registers
them:

```
discover_builtin_skills()  →  SkillService::register_builtins()
                           →  SkillRegistry::register()
                           →  PersistenceLayer::save_skill()
```

Provenance is read from the pack's `SKILLS.md` frontmatter — version, commit, and
skill count — so a stale pack is visible rather than silently assumed current.

:::info Why provenance matters
UAR's pin on this pack was once **359 commits stale** (161 skills vs 220), with
nothing in the system able to detect it. The provenance reader exists so that
drift is observable instead of inferred.
:::

## The embedded-platform bug (R1)

R1 says "every platform, embedded or not." The embedded platform was the one it
failed on.

`SkillRegistry::register` gated persistence on **both** a database and an
embedder:

```rust
// BEFORE — the bug
if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
    // …persist…
}
```

A host with a database but **no `VectorMatcher`** matched nothing and persisted
nothing — silently, because a tuple pattern that fails to match is not an error.
An embedded host has no embedding service **by definition**, so the platform the
requirement exists for was exactly the one whose skills never reached the
database.

Measured before fixing:

```
builtin rows in the database (0) != skills the loader discovered (3)
```

Every consumer reading from persistence — the admin UI (R3), the REST API (R4), a
mobile embedded host — saw an **empty catalogue** from a process that logged
`registered builtin skills` and looked healthy.

### The fix

Persistence and embedding are now independent concerns:

```rust
// AFTER
if let Some(db) = &self.persistence {
    // Best-effort embedding: None (no matcher) and Err (matcher failed) both
    // degrade to persisting WITHOUT one, rather than dropping the skill.
    let embedding: Vec<f32> = /* … */;
    db.save_skill(&skill, &embedding).await
}
```

**An embedding enriches vector search; it is not a precondition for the skill
existing.**

### Provider verification

Acceptance is **row-count equality**, not "some rows exist" — a subset test
passes while skills are silently dropped.

| Provider | Result | Notes |
|---|---|---|
| `memory` | ✅ verified | 3 == 3, plus an exact-set test so over-registration cannot satisfy it |
| `surreal` (embedded) | ✅ verified | 3 == 3 against file-backed SurrealKV, **no server needed** |
| `postgres` | ⛔ **BLOCKED** | needs `pgvector` for PG16 |

**R1 is therefore PARTIAL, not MET.** One provider is unverified, and it is named
rather than dropped.

:::note Two "BLOCKED" verdicts that measurement refused
**Surreal** was nearly recorded BLOCKED on "needs a live server." It does not —
UAR compiles `kv-surrealkv`, which runs in-process. Recording BLOCKED would have
left the *embedded* path unproven behind a tidy status. (Note it is also not
`"memory"` → `mem://`; that engine is not compiled in and fails with
`Unsupported scheme: memory`. The embedded store must be file-backed.)

**Postgres** was nearly recorded BLOCKED on "no server." One *is* live on
`127.0.0.1:5432`. The real blocker, found only by attempting the install:
`migrations/20251225000000_init_uar.sql:2` requires `CREATE EXTENSION vector`,
and `brew install pgvector` ships only `postgresql@17`/`@18` bottles against a
16.14 server. A PG17-built `.so` in a PG16 install is an ABI mismatch, not a fix.
:::

## The bug the fix nearly introduced

Worth documenting because the pattern recurs: **fixing a silent data-loss bug can
create another one.**

`register_all` routes through `register`, and `register_all` is what
`initialize()` and `refresh()` call on skills they **just read** from a provider.
Making `register` persist therefore made the *load* path write back — and
`save_skill` upserts:

```sql
ON CONFLICT (skill_id) DO UPDATE SET … embedding = EXCLUDED.embedding
```

So a host with no embedder would overwrite **every stored embedding with an empty
vector on every restart**, silently degrading vector search on a database that
was previously healthy.

**Fix:** load and save are separate operations.

| Operation | Persists? | Used by |
|---|---|---|
| `index_only` / `index_all` | No | `initialize()`, `refresh()` — skills read **from** a provider |
| `register` / `register_all` | Yes | `register_builtins()`, create/update — skills genuinely **entering** the system |

### Best practice extracted

**When you change a function, ask who else calls it.** A green test on the path
you were thinking about says nothing about the three paths you were not. This bug
would have shipped behind a passing test suite.

## Builtins cannot be deleted (R2)

The guard is at the **database**, not only in the service layer:

```sql
-- migrations/20260731000000_builtin_skill_delete_guard.sql
-- BEFORE DELETE trigger raising `system_skill_immutable`
-- when OLD.definition->>'origin' = 'builtin'
```

`SkillService::delete_skill_permanent` already refused builtins, but that guard
lives in **one call path**. `DatabaseStorageProvider::delete_skill` passes
straight to `DELETE FROM skills`, so any caller holding the provider — a
maintenance task, a repair script, a refactor reaching one layer lower — bypasses
it. The trigger closes the route at the database.

Two details that matter:

- **`CHECK` is the wrong tool.** It fires on INSERT, not DELETE.
- **The origin value is lowercase.** `SkillOrigin` carries
  `#[serde(rename_all = "lowercase")]`, so the JSON is `"builtin"`, not
  `"Builtin"`. A trigger matching the capitalised form would never fire **while
  reading as protection** — the worst kind of bug. A test pins this.

**Disabling remains allowed.** The requirement is "turned off, never deleted"; a
guard that also blocked disabling would over-shoot.

## Testing guidance

Three versions of one regression test were written before a real one, each
worthless in a different way. They are recorded because each failure mode is
common:

1. **A double that discards the value under test.** `InMemoryProvider::save_skill`
   takes `_embedding` and drops it — it cannot observe an embedding clobber. The
   test would pass whether or not the bug existed.
2. **A code path that never executed.** Calling `initialize()` on a service with
   **no providers attached** — `initialize` iterates `self.providers`, so the load
   path never ran and the assertion held *even with the bug present*.
3. **A trait double that does not compile.** Hand-implementing `PersistenceLayer`
   (24 required methods) rather than the small `SkillStorageProvider` (8).

**Best practice:** assert that the path under test *ran*, not only that the result
looks right. Every one of these passed or compiled cleanly while proving nothing.

```rust
// Guard against a vacuous pass
let loaded = service.registry().read().await.len();
assert!(loaded >= 1, "the load path never ran; this test proved nothing");
```

## Reproducing

```bash
cd universal-agent-runtime

# Embedded surreal + in-memory equality tests
cargo test --features in-memory-backend --test builtin_db_registration

# Provenance (pack version/commit/count)
cargo test --lib provenance

# Builtin delete guard — requires DATABASE_URL; skips LOUDLY without it
DATABASE_URL=… cargo test --test builtin_delete_guard
```
