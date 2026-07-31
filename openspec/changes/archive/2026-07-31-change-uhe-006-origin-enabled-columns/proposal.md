# Make origin and enabled constrainable

**Change:** `change-uhe-006-origin-enabled-columns`
**Phase:** uar-host-execution
**Goal:** R2

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: probe said YES — no columns added

Task 1 was a probe, and the plan required this change to **complete either way**
so downstream dependencies never dangle. It completed on the cheaper branch.

### What the probe found (Postgres 18.4, run not reasoned)

| Question | Answer |
|---|---|
| Can a `CHECK` read `definition->>'origin'`? | Yes — **but it is the wrong tool.** `CHECK` fires on INSERT, so it blocks *loading* builtins, the opposite of the requirement. |
| Can a `BEFORE DELETE` trigger read it? | **Yes.** User skill deletes; builtin is refused; no columns involved. |
| Does a bulk `DELETE FROM skills` bypass it? | **No** — refused and rolled back atomically; unrelated rows survived. |
| Can a builtin still be *disabled*? | **Yes** — `enabled` stays freely writable, which is the "turned off, never deleted" requirement. |

### The bug the probe nearly shipped

My first trigger matched `'Builtin'`. `SkillOrigin` carries
`#[serde(rename_all = "lowercase")]`, so the wire value is **`"builtin"`** — the
trigger would never have fired while *reading as protection*, which is worse
than no trigger at all.

Caught by checking the enum's serde attribute rather than assuming the Rust
spelling, and now pinned by `builtin_origin_serialises_lowercase` so a future
rename breaks a test instead of silently disarming the guard.

### Delivered

`migrations/20260731000000_builtin_skill_delete_guard.sql` — a `BEFORE DELETE`
trigger raising `system_skill_immutable`. **No schema migration, no backfill, no
provider change**, because `postgres.rs:77` already serialises the whole `Skill`
into `definition` JSONB.

Two tests close the loop from Rust to SQL: one asserts the serialised wire value
is `"builtin"`, the other asserts a real `Skill` produces a top-level `origin`
the trigger matches — so a serialisation change surfaces as a test failure
rather than as data loss.

Task 3 (the NO branch) is marked **N/A**: it was conditional on the probe
failing, and it did not.
