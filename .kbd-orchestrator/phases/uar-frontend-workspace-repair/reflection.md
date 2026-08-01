# Reflection — uar-frontend-workspace-repair

**Closed:** 2026-08-01 · Single-goal phase, resolved in one pass.

## Goal

Unblock `tsc -b` and the Playwright suite, both disabled by an unresolvable
`@prometheus-ags/prometheus-entity-management` import.

| Goal | Verdict | Evidence |
|---|---|---|
| Repair the workspace so tsc and e2e run | **MET** | `tsc -b`: 90+ errors → **0**. Playwright: could not start → **5 passed, 3 skipped** |

## Delta — the previous reflection named the wrong cause

`uar-host-execution` recorded this as a package **name mismatch**: the workspace
package is `@prometheus-ags/entity-graph-workspace` while 90+ files import
`@prometheus-ags/prometheus-entity-management`.

True but incidental. Measured properly, the imported package **does exist** —
nested a level deeper at `packages/entity-graph-react`, already covered by the
workspace glob, and already linked in `frontend/node_modules`.

**The real cause:** its `package.json` points at `./dist/`, and `dist/` did not
exist. The package had never been built.

**Root cause of the misdiagnosis:** I read the top-level `package.json`, saw a
name that did not match, and stopped. A linked package with no build output is
unresolvable in exactly the way a missing package is — the symptoms are
identical, so the first plausible explanation looked sufficient.

**Corrective action:** when a module fails to resolve, check for build output
before concluding it is absent. `ls dist/` would have cost one command.

## What was actually required

Build the chain in order — `entity-graph-core`, then `entity-graph-react` —
from the **submodule's own pnpm workspace root**, which carries a separate
`tsup` install the outer workspace cannot reach.

Two environment notes worth keeping:

- Port 8080 was held by an unrelated `ssh` tunnel. The suite honours
  `UAR_FRONTEND_E2E_PORT`, so nothing needed killing — a config knob beat a
  process kill.
- `pnpm -r build` at the submodule root fails on its example apps; filter to the
  two packages that matter.

## The three R3 tests are skipped, and that is a finding

They need a **rendered skill row**. Mocking `GET /api/skills` does not produce
one: `skills-page.tsx` reads through the entity graph (`useSkills()`), so an
HTTP fixture never reaches the view.

Verified: **no test in this file has ever rendered a row.** All five
pre-existing tests mock `{ skills: [] }` and assert the empty state. The suite
has no precedent for the technique these tests need — seeding the entity graph
before navigation.

Leaving them red would be noise; deleting them would lose the coverage gap.
Skipped with the prerequisite named.

**R3 remains PARTIAL**: the behaviour is correct (verified by reading
`skills-page.tsx` — badge at :248, `disabled={isBuiltin}` on Delete, toggle
gated on `isBusy` only), but no executed test proves it.

## Carry-forward

An **entity-graph seeding helper for e2e**. Without it, no admin page whose data
flows through the entity graph can be tested end to end — a gap wider than these
three tests.
