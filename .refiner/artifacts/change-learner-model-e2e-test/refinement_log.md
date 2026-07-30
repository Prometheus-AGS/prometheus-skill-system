# Refinement log — change-learner-model-e2e-test

Lightweight validation (no `.kbd-orchestrator/constraints.md` constraint applies
to this diff; full multi-agent adversarial-review skipped per user decision
given session cost — 45/45 tests green is the primary evidence).

## Changed files

- `substrate/sovereign-sync/tests/domain_sync.rs` — new
  `learner_model_replicates_end_to_end_between_two_nodes` test
- `substrate/sovereign-sync/src/rest_api.rs` — two production bugs the test
  surfaced and fixed: (1) `LearnerModelAdapter`'s storage dir was hardcoded to
  the real `~/.prometheus/learn/learner-model` in `from_control_plane`,
  ignoring `data_root` — non-hermetic for `try_new_at`-based tests; scoped it
  under `data_root` instead (production `try_new`/`new` behavior unchanged).
  (2) `build_push_envelope` always set `envelope.identity` from
  `kbd_control.status().project_id` regardless of family, but
  `handle_incoming_message`'s `learner-model` branch checks identity against
  `default_learner_id()` (the OS user) — a real mismatch that would silently
  drop every learner-model sync in production. Made identity computation
  family-aware to match the receive side.
- `substrate/sovereign-sync/Cargo.toml` — added `chrono` to `[dev-dependencies]`
  (needed by the new test; already a transitive dep, just not directly
  declared for the package's own test target).
- `substrate/sovereign-sync/Cargo.lock` — resulting lockfile update.

## Constraint check (`.kbd-orchestrator/constraints.md`)

| Constraint | Status | Note |
|---|---|---|
| C-01 generated artifacts in sync | N/A | No `.claude-plugin/*`, `.mcp.json`, `hooks/hooks.json`, or `scripts/build-codex-plugin.js` touched |
| C-02 no committed secrets | PASS | No credentials/tokens in the diff |
| C-03 docs updated with surface changes | N/A | No Codex plugin surface change |
| C-04 generators stay idempotent | N/A | No generator script touched |
| C-05 bash 3.2 compat | N/A | No shell script touched by this change |

## Build/test evidence

- `cargo test -p sovereign-sync`: 45/45 passed (26 lib + 3 domain_sync + 15
  integration_tests + 1 new)
- `cargo check -p sovereign-sync --tests`: clean

## Verdict

PASS — no blocking constraints, tests green. Proceed to archive.
