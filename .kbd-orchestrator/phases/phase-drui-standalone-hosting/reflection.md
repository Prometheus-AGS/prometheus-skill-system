# Reflection — phase-drui-standalone-hosting

_Reflected: 2026-07-09_

## Summary

`phase-drui-standalone-hosting` took `deep-research-ui.html` from a
non-functional standalone file (absolute paths that 404'd under `file://`,
no server route serving the shell or static assets) to a fully hosted,
brand-adopted, container-packaged, CI-guarded UI. All 6 goals MET, 0
regressions, 0 technical debt carried forward. `change-drui-001` through
`change-drui-005` were delivered directly via descriptive commits;
`change-drui-006` was driven through the `/kbd-apply` per-task loop
(4/4 tasks) after the change lacked a native-kbd task scaffold — the
scaffold was authored as part of this phase's close-out.

---

## Goal Achievement

| Goal | Status | Change | Commit |
|------|--------|--------|--------|
| G-01 — Serve UI shell (`GET /`, `GET /static/*`) | **MET** | change-drui-002-serve-ui-shell | `d320a6d` |
| G-02 — Relative script paths + README | **MET** | change-drui-003-relative-paths-and-readme | `644855c` |
| G-03 — Docker Compose packaging | **MET** | change-drui-004-docker-compose | `86200ef` |
| G-04 — KnowMe brand adoption | **MET** | change-drui-005-restyle-ui-knowme | `833d93f` |
| G-05 — KnowMe logo + icon assets | **MET** | change-drui-001-brand-tokens-and-assets | `9d5bbee` |
| G-06 — install-binaries.sh wiring + CI smoke | **MET** | change-drui-006-ci-smoke | (this session) |

**Achievement rate: 6/6 (100%)**

### Verification evidence

- **G-01/G-06:** `prometheus-research --mode server --port 7899` started
  cleanly; `curl` against `/health`, `/`, `/static/htmx.min.js`, and
  `/brand/primary-light.svg` all returned HTTP 200 with non-empty bodies
  (203686, 51250, and 793 bytes respectively). The same check now runs as
  a dedicated `ci-smoke` job in `.github/workflows/prometheus-research.yml`,
  independent of the existing fmt/clippy/test matrix.
- **G-02:** `deep-research-ui.html` uses `./static/...` (3 occurrences),
  and `docs/deep-research/README.md` documents all three run modes
  (native launchd, `cargo run`, Docker Compose) plus a troubleshooting
  section.
- **G-03:** `docs/deep-research/docker-compose.yml` and
  `docs/deep-research/Dockerfile` both present.
- **G-04/G-05:** KnowMe brand tokens and logo assets
  (`primary-{light,dark}.svg` + PNG variants at 16/32/180/512) vendored
  under `substrate/prometheus-research/src/static/brand/`; confirmed
  served correctly by the axum static handler.

---

## Delivered Changes

### change-drui-001-brand-tokens-and-assets
Vendored `primary-light.svg` / `primary-dark.svg` and generated PNG
variants (16/32/180/512) plus `brand/tokens.css`. **Commit:** `9d5bbee`.

### change-drui-002-serve-ui-shell
Extended the axum server with `GET /` (HTML shell), `GET /static/{*path}`,
and `GET /brand/{*path}` routes, all on the existing `:7891`
`prometheus-research` daemon. **Commit:** `d320a6d`.

### change-drui-003-relative-paths-and-readme
Switched script tags to relative paths; added `docs/deep-research/README.md`.
**Commit:** `644855c`.

### change-drui-004-docker-compose
Multi-stage `Dockerfile` (`rust:1.88` → `debian:bookworm-slim`) and
`docker-compose.yml` exposing `:7891`. **Commit:** `86200ef`.

### change-drui-005-restyle-ui-knowme
Adopted KnowMe brand tokens and Conviction logomark across the UI —
title/meta tags, theme-aware favicons, `<link>` to `brand/tokens.css`,
`:root` token replacement. **Commit:** `833d93f`.

### change-drui-006-ci-smoke
Driven via `/kbd-apply` (4/4 tasks — scaffolded from scratch, since no
`tasks.json` existed for this change yet):
1. Locally verified the server boots and all 4 routes return 200 +
   non-empty body.
2. Added the `ci-smoke` job skeleton (checkout, cargo build --release,
   start server, poll `/health` with a 30s timeout).
3. Added the route-assertion step (curl `/`, `/static/htmx.min.js`,
   `/brand/primary-light.svg`, assert 200 + non-empty body) and an
   `if: always()` teardown step.
4. Validated the YAML (`python3 -c "import yaml; yaml.safe_load(...)"`)
   and confirmed the existing `rust` job's fmt/clippy/test matrix is
   untouched.

Backend `verify` and `archive` both passed; archived to
`.kbd-orchestrator/changes/archive/2026-07-09-change-drui-006-ci-smoke/`.

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA runs | 0/6 (artifact-refiner not configured for this phase) |
| First-pass pass rate | N/A |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

No `.refiner/artifacts/` logs exist for this phase. `change-drui-006`'s
four tasks were each manually verified (live route checks against a running
instance, plus YAML re-parse) before being marked done.

---

## Technical Debt Introduced

**None new.** Two things worth tracking, both already implicit in the
phase's own goals.md non-goals:

- `install-binaries.sh` already copies `src/static/` (including the new
  `brand/` subdirectory) to `docs/deep-research/static/` on every run —
  this mirror remains an unmanaged build artifact by design (see
  `phase-prui-polish`'s reflection for the original rationale), not a
  regression introduced here.
- The `ci-smoke` job builds `--release` from scratch on every run (shares
  the `rust` job's cache key prefix but not its `matrix.command`
  dimension) — acceptable for now given the low change frequency of this
  crate; revisit if CI minutes become a concern.

---

## Lessons Captured

1. **Native-kbd changes need their task scaffold at plan time, not
   apply time.** `change-drui-001` through `-005` were delivered via
   direct commits without ever populating
   `.kbd-orchestrator/changes/<id>/tasks.json` — fine for solo,
   fast-moving infra work, but it meant `change-drui-006` had no scaffold
   for `/kbd-apply` to drive, and one had to be authored ad hoc
   (`change.md` + `tasks.json`) before the per-task loop could run. For
   phases that declare "Native KBD (inline task tracking)" in `plan.md`,
   scaffold every change's `tasks.json` during `/kbd-plan`, not lazily
   during `/kbd-apply`.

2. **`current-waypoint.json.exactNextCommand` silently desyncs.**
   `position-reminder.txt`'s "Next command" line renders from
   `.exactNextCommand`/`.exact_next_command`, NOT from the more
   frequently-updated `.next_command` field
   (`shared/scripts/write-position-reminder.sh:62`). Across this session
   `exactNextCommand` sat on a stale value (`/kbd-assess phase-prui-polish`)
   through an entire phase-close + new-phase + 6-change execute cycle,
   because nothing was writing to that specific field. Any script that
   updates `next_command` should update `exactNextCommand` in the same
   write, or the two should be collapsed into one field.

3. **Concurrent-session divergence is real and un-flagged.** This phase's
   first 5 changes landed via a separate session/tool while this session
   was mid-task on unrelated build/distribution work, with no local
   signal (no fetch, no notification) until a `git push` was rejected.
   `git fetch` (or a periodic background fetch) before trusting local
   waypoint state would have surfaced the newer phase sooner.

4. **`cp` without `-f` fails in two distinct silent ways** (found earlier
   this session, load-bearing for anyone touching `install-binaries.sh` /
   `install-skills-flat.sh` again): "Permission denied" on a root-owned
   destination, and "Text file busy" on a currently-running binary. Both
   get swallowed by `2>/dev/null || true` patterns already in these
   scripts. `cp -f` fixes both by unlink-and-recreate instead of
   write-in-place.

---

## Delta vs Plan

| Planned | Delivered | Delta |
|---------|-----------|-------|
| 6 changes | 6 changes | 0 |
| 6 goals MET | 6 goals MET | 0 |
| 0 regressions | 0 regressions | 0 |

No scope deltas. `change-drui-006` took an extra scaffolding step not
anticipated in `plan.md` (see Lesson 1), but the delivered surface matches
the plan exactly.

---

## Recommended Next Phase

The `deep-research-ui.html` / `prometheus-research` track is now feature-
complete for standalone hosting: served from one process, brand-adopted,
container-packaged, and CI-guarded. No blocking technical debt.

**Recommended next focus:** pick the next highest-value item from
`project.json`'s phase backlog — no single obvious successor was flagged
during this phase. Candidates worth considering, in no particular order:
hardening `sovereign-sync`/P2P sync (`phase-sovereign-sync-hardening` is
already a known phase slug), or closing out `phase-learn-feynman` /
`phase-learn-sovereign-sync` if either has open goals. Confirm with the
user before opening a new phase — this reflection does not pick one
unilaterally.
