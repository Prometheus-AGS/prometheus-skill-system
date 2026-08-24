# Design: explicit target source-tree lifecycle

`skill-system.json` owns the canonical target matrix, so each target now declares
`sourceTreeLifecycle` as either `required` or `install-only`. The required set is the five
tracked harness trees currently present in this repository: OpenCode, Cursor, Codex, Devin,
and Agents. The remaining targets are installation destinations and therefore declare
`install-only`; their absence from source is intentional rather than silent drift.

`readSkillSystem()` is the shared validation boundary used by distribution generation and
both installation paths. It rejects omitted or unknown policies and missing or empty
required trees before staging begins. A dedicated fixture test covers omitted, missing,
empty, install-only-absent, and repeated-validation behavior.

No normalizer is appropriate. These trees do not carry a generated `internal: true`-style
marker or another invariant that could be deterministically re-applied. Validation is the
complete control and is intentionally read-only.

## Verification record (2026-08-24) — adversarially re-checked, not accepted on the ledger

All nine tasks arrived already checked. Per this phase's own lesson (c401: a
checked box is not evidence), the gate was re-tested against the real tree rather
than trusted.

**The gate is genuine.** `readSkillSystem(sourceRoot)` was exercised against a
deliberately broken working tree, restoring it afterwards:

| Case | Result |
|---|---|
| healthy tree | passes — 14 targets, required = opencode, cursor, codex, devin, agents |
| `.cursor/skills` **emptied** | `required target source tree is empty: cursor (.cursor/skills)` |
| `.cursor/skills` **removed** | `required target source tree is missing: cursor (.cursor/skills)` |
| `sourceTreeLifecycle` **omitted** | `target cursor must declare sourceTreeLifecycle as required or install-only` |

It distinguishes empty from missing and names the target in every message — this
is exactly the `.windsurf/skills` silent loss that motivated the change. Two
consecutive runs pass identically (2.2), and the target list is read from
`skill-system.json` rather than hardcoded (2.3): flipping one target's policy in
the contract changes the validator's behaviour.

Not test-only: `readSkillSystem` is called from `generate-skills-index.js`,
`generate-skill-system-distribution.js`, `install-plugin-generation.js`, and
`install-system.js` — both install paths.

**One correction to my own method.** The first adversarial run threw
`The "path" argument must be of type string`, which I briefly read as the gate
failing. It was my error — I called `readSkillSystem()` with no `sourceRoot`.
Re-run correctly, the gate behaved as designed. Worth recording because a
crash-shaped throw can be mistaken for a working guard.

## Defect found while verifying: four stale import records

`scripts/tests/skill-system-distribution.test.mjs` failed. `skill-system.json`
records each submodule's commit a **second** time in `imports[].commit`, and the
test asserts it equals the actual gitlink — the half-updated-pin integrity check.
Four records were stale:

| Path | Recorded | Actual |
|---|---|---|
| `skills/imported/prometheus-entity-management` | `d7e8840` | `9c30ad1` |
| `tools/disk-space-guardian` | `86853b4` | `26487db` |
| `tools/prometheus-knowledge` | `cea7b90` | `5a175d1` |
| `tools/surreal-memory-server` | `6a03ee5` | `2e2188a` |

The first is mine — I advanced the PEM gitlink to 3.0.3 without updating its
record. The other three came from the parallel convergence work. Reconciled by
deriving each value from `git ls-tree` rather than hand-editing, asserting exactly
one occurrence per commit before substituting.

**This is a live instance of the class c404 exists to prevent**, in a second
location the change did not cover: `sourceTreeLifecycle` guards *tree presence*,
while nothing guards *pin record currency* except a test that must be run. Worth
a follow-up so the reconciliation is generated rather than remembered.
