# Generated skills register only when opted in

**Change:** `change-uhe-011-dynamic-skill-registration`
**Phase:** uar-host-execution
**Goal:** R4

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: opt-in shipped, default OFF, both sides proven

```
test result: ok. 6 passed; 0 failed
  without_the_opt_in_a_generated_skill_is_not_registered ... ok
  with_the_opt_in_a_generated_skill_is_registered ... ok
```

`SkillsApi::install_generated()` is deliberately **separate** from `install()`:

| Method | Semantics |
|---|---|
| `install()` | the caller asked; it happens |
| `install_generated()` | a tool produced it; gated, returns `Ok(None)` when off |

Two ways to opt in, because an embedded host cannot usefully set an env var on
itself:

- `UAR_REGISTER_GENERATED_SKILLS=true` (or `1`)
- `.with_generated_registration(true)` — programmatic, per-session

Matches the existing `builtin_loader.rs` convention exactly, `unwrap_or(false)`
included.

### Why the default carries the requirement

R4 says registration should happen **optionally**. That has to live in the
default, not the prose: a generator writing to the database by default silently
grows a user's catalogue with artifacts they never chose to keep, and a `skills`
table that fills on its own is far harder to diagnose than one that stays empty.

### Two test-design decisions worth keeping

1. **Asserted against the database, not the return value.** A function can
   return `None` while still having written a row — that would be the worst
   outcome, since the caller believes nothing happened.
2. **Passed `false`/`true` explicitly** rather than relying on an unset env var.
   Tests share a process; another test setting `UAR_REGISTER_GENERATED_SKILLS`
   would otherwise make this pass or fail by execution order.

`SkillsApi::for_test` was added so an integration test — a separate crate that
cannot reach `pub(crate)` — can exercise the facade without standing up a whole
runtime, and notably without supplying an LLM driver it has no use for.
