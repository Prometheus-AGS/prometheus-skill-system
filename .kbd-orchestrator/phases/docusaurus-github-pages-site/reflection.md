# Phase Reflection: docusaurus-github-pages-site

**Project:** prometheus-skill-pack
**Date:** 2026-07-28
**Phase completion:** 100% implementation (8/8); 3/3 goals MET
**Changes completed:** 8 / 8

## Deltas from plan (lead with what diverged)

### D-1 — The phase's own post-execution assessment was wrong by the time reflect ran

`assessment.md` "Post-execution reassessment" (2026-07-27) scored **Goal 2 as
AUTHORED, NOT DEPLOYED**, citing two 404 probes: workflow absent from the
default branch, and Pages not enabled. It concluded "no deploy can occur"
and listed 46 uncommitted paths as blocking.

All three claims are now false. Verified 2026-07-28:

| Probe | Assessment claim (07-27) | Verified now (07-28) |
|---|---|---|
| `actions/workflows/docs-pages.yml` | 404 — not on default branch | `321475908`, state `active` |
| `repos/…/pages` | 404 — Pages not enabled | enabled, `build_type: workflow`, https enforced |
| Deploy run | none possible | run `30277418174` — **success**, build + deploy jobs both green, 1m52s |
| Working tree | 46 uncommitted paths | clean (2 unrelated wiki files) |

**Root cause:** the reassessment was written mid-flight, before commit
`d0143c2` was pushed and before the operator enabled Pages. It was never
re-run afterward, so the phase carried a stale NOT-DEPLOYED verdict into
reflect. The `git pull` at the start of this session is what surfaced the
gap — the assessment described a working-tree state two commits stale.

**Corrective action:** reflect must re-probe deployed reality rather than
inherit the assessment's verdict. Applied here. Going forward, any phase whose
goal is "deployed" should re-run its deploy probes *at reflect time*, not only
at execute time.

### D-2 — Certification blockers recorded in progress.json were already resolved

`progress.json → completion.certification` lists two blockers: "operator:
enable GitHub Pages (Source: GitHub Actions)" and "operator: brand decision
for deferred dgp-009". The first is **resolved** (Pages is enabled and has
served a successful deploy). The second is not a blocker on this phase's
goals at all — dgp-009 was explicitly deferred *out of scope* by decision
D-003, so it cannot gate certification of the eight in-scope changes.

**Root cause:** the ledger conflated an out-of-scope deferred change with an
in-scope certification blocker. **Corrective action:** deferred-out-of-scope
items belong in "Next Phase Focus", never in `certification.blockers`.

### D-3 — Waypoint carried contradictory state from the prior phase

`current-waypoint.json` still held `goals_total: 6` from
`phase-learn-grader-validation` while this phase's `goals.md` declares 3.
`position.json` was staler still — dated 2026-07-13 with cursor
`phase-codex-plugin-verify-and-publish`, two phases behind, so `/kbd-status`
had to fall back to waypoint rendering.

**Root cause:** `/kbd-new-phase` copied the waypoint forward without resetting
goal counters, and nothing calls `kbd_position_sync` at phase transitions.
**Corrective action:** operator confirmed `goals.md` (3) is authoritative;
corrected below. Position sync at phase boundaries is a carry-forward.

### D-4 — Only 4 of 8 changes carry recorded adversarial certification

`change_certifications` records dgp-002/003/004/006. The other four
(001, 005, 007, 008) are verified but have no recorded adversarial gate. Per
plan this was intentional (dgp-001 is <3 files and auto-skips; docs-only
changes skip by heuristic), so this is a **known, deliberate** coverage gap
rather than a miss — but the ledger does not distinguish "skipped by policy"
from "not run", which makes the gap unreadable after the fact.

### D-5 — dgp-004 anti-theater gate accepted at the 2-rejection soft cap

Recorded verbatim in `change_certifications`: S-03 fired on inline resolution
annotations, and the change was accepted at the soft cap rather than on a
clean pass, "flagged for manual review". That manual review has not happened.
dgp-006 subsequently changed approach (kept resolutions in verification.md)
to avoid the same trigger — so the lesson was learned mid-phase, but dgp-004
itself was never revisited.

## Goals

Scored against `goals.md` (3 goals — operator-confirmed authoritative over the
waypoint's stale `goals_total: 6`).

| Goal | Status | Notes (independently verified, not echoed from plan) |
| --- | --- | --- |
| Stand up a Docusaurus site for skill-pack docs (skills catalog, KBD lifecycle, learn domain, substrate crates) | **MET** | Local build exit 0 under `onBrokenLinks: 'throw'`; catalog generator emits **140 skills / 17 categories**; `kbd/`, `substrate/`, `learn/`, `sovereign-sync/` sections all present and serving. Live probes: `/docs/` 200, `/docs/catalog/` 200, `/docs/substrate/` 200, `/docs/kbd/overview` 200. |
| Deploy to GitHub Pages via Actions on push to main | **MET** | Contradicts the phase's own assessment (see D-1). Workflow active on default branch; Pages enabled with `build_type: workflow`; run `30277418174` succeeded end-to-end (build + deploy) on the `d0143c2` push. Site serves at `https://prometheus-ags.github.io/prometheus-skill-system/`. |
| Migrate/link existing docs into the site without duplicating canonical sources | **MET** | 11 `site/docs/guide/*` copies deleted; `quick-start.md` relocated to `docs/guide/00-quick-start.md` as a tracked rename; three `plugin-content-docs` instances read canonical dirs *outside* `site/` (`../docs/guide`, `../docs/learn`). Verified no remaining duplicate canonical sources in the site tree. |

## Delivered Changes

All by: claude-code (single-session `/kbd-apply` driver; `sourceTool` recorded
as `unknown` in the ledger — see D-6 in Cross-Tool Coordination).

- `change-dgp-001` — Pages config correction (url/baseUrl/org/project, `SITE_URL`/`BASE_URL` env)
- `change-dgp-002` — `docs-pages.yml` deploy workflow (pinned SHAs, least-privilege) — QA + adversarial FORCED, 2 rounds, 1 CRITICAL fixed
- `change-dgp-003` — guide dedupe: delete site copies, serve canonical `../docs/guide` — 2 rounds at cap, 3 content bugs fixed
- `change-dgp-004` — learn-internals instance + kb-adapter canonical merge — accepted at anti-theater soft cap (see D-5)
- `change-dgp-005` — substrate section (storage-provider, learner-model, surface-bridge, sovereign-client, prometheus-research)
- `change-dgp-006` — skills catalog generator (140=140 parity, idempotent, search-indexed) — 6 findings, all applied
- `change-dgp-007` — search + sidebar split (`sidebars-catalog/guide/learn-internals`)
- `change-dgp-008` — KBD lifecycle section (overview, stages, hooks-and-waypoints, quality-gates)

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 4/8 (4 auto-skipped by policy: <3 files / docs-only) |
| Adversarial certification recorded | 4/8 (dgp-002, 003, 004, 006) |
| Changes requiring refinement | 3 (dgp-002, 003, 006 — all fixes applied) |
| Anti-theater gate | 3 clean PASS, 1 accepted at soft cap (dgp-004) |
| Stage gates | assess PASS (0 CRITICAL) · plan PASS (2 rounds, 1 CRITICAL fixed) · assess-post PASS (score 0.0, strict) |

**No `.refiner/artifacts/` logs exist** — the artifact-refiner QA path was not
used this phase; certification came from `refine_validate` + adversarial review
recorded inline in `progress.json`. The standard Artifact Quality table above is
reconstructed from that ledger, not from refiner logs.

### Recurring constraint violations

None recurring across 2+ changes. Isolated: S-03 (caveat collapse) fired once,
on dgp-004's inline resolution annotations.

## Technical Debt

1. **`/docs/learn-internals/` returns 404** — the `learn-internals` plugin
   instance has no index doc (`docs/learn/` has no `README.md` or `index.md`,
   unlike `docs/guide/README.md`). Leaf pages serve fine (200 on
   `kb-adapter-guide`, `surface-tier-detection`, `crdt-conflict-semantics`)
   and the navbar links directly to the first doc, so **no user path hits the
   404** — but a hand-typed or externally-linked section root does. Fix: add
   `docs/learn/README.md`.
2. **dgp-004 manual review never performed** (D-5) — accepted at the
   2-rejection soft cap with an explicit "flagged for manual review" note.
3. **Duplicate `10-` numbering in `docs/guide/`** — `10-language-skills.md`
   and `10-learn-skills.md` share a prefix. No id collision today (Docusaurus
   derives distinct ids from filenames) and the build is green, but sidebar
   ordering between the two is unspecified/arbitrary.
4. **Deprecated Docusaurus config surface** — build warns twice that
   `siteConfig.onBrokenMarkdownLinks` moves to `siteConfig.markdown.hooks.*`
   in v4. Harmless now; breaks on the v4 upgrade.
5. **`site/` was not installable from a clean checkout during this session** —
   `npm run build` failed with `docusaurus: command not found` until `npm ci`
   was run. CI does this correctly; only local verification was affected. No
   fix needed, but it means "build green" was never locally reproducible until
   now.
6. **Catalog counts 140 skills; `find skills -name SKILL.md` counts 303** —
   the delta is curated-catalog scope vs raw file count (nested sub-skills in
   bundles). Not a defect, but nothing in the site or generator documents why
   the numbers differ, which will read as a bug to the next person.
7. **`progress.json → certification` is stale** (D-2) — still lists a resolved
   Pages blocker and an out-of-scope deferred change as blockers.

## Architecture Integrity

- **AGENTS.md "Never Do" violations: NONE** — `AGENTS.md` exists but contains
  no "Never Do" section to check against (grep returned nothing). Recording
  this as *unverifiable*, not as a pass.
- **Constraint violations: NONE.** C-01 independently verified this session:
  `npm run validate:codex` → "Codex artifacts up to date and valid." C-02
  (no secrets): workflow uses `${{ }}` refs and pinned SHAs only, no literals.
  C-03/C-04/C-05 not applicable — this phase touched no Codex plugin surface,
  no generator besides the new catalog script (which dgp-006 verified
  idempotent), and no launchd-invoked script.
- **Anti-goal (duplication) avoided**: the canonical-serving pattern means the
  site holds zero copies of `docs/guide` or `docs/learn` content.

## Cross-Tool Coordination Notes

- **Progress tracking: GAPS FOUND.** Three distinct staleness failures in one
  phase — `assessment.md` re-scored against a superseded tree (D-1),
  `certification.blockers` never cleared after the operator acted (D-2), and
  `position.json` two phases behind (D-3). Common cause: **state is written
  once at stage exit and never re-validated**, so any external event
  (a push, an operator toggling a setting) silently invalidates it.
- **`sourceTool: "unknown"`** in `progress.json` despite a single known
  executor. The field was never populated, so the ledger cannot attribute work.
- **Handoff quality: CLEAR.** All four handoffs (`assess`, `analyze`, `plan`,
  `execute`) present and well-formed; the execute handoff correctly named the
  backend, the ordering contract, and the first pending change. The stage gate
  passed cleanly on the execute handoff.
- **Recommendations:** (a) re-probe external reality at reflect time for any
  goal phrased as "deployed"/"published"; (b) clear resolved blockers when the
  resolving event is observed, not at the next stage; (c) call
  `kbd_position_sync` at every phase transition; (d) populate `sourceTool`.

## Lessons Learned

- **A phase's own assessment is evidence, not truth.** The post-execution
  reassessment here was internally rigorous — it ran real `gh api` probes and
  drew a defensible conclusion — and was still wrong 24 hours later because
  the world moved. Verdicts about *external* state (deploys, enabled settings,
  remote branches) expire; verdicts about *tree* state do not.
- **Distinguish "skipped by policy" from "not run" in QA ledgers.** Four
  changes carry no adversarial certification for entirely legitimate reasons,
  but nothing records *which* reason, so the gap is indistinguishable from an
  oversight after the fact (D-4).
- **Soft caps need a follow-through owner.** The 2-rejection soft cap
  correctly prevented an infinite loop on dgp-004, but "flagged for manual
  review" with no assignee meant the review never happened. A soft-cap
  acceptance should create a tracked item, not a note.
- **Mid-phase learning propagated correctly.** dgp-006 changed its
  verification-writing approach specifically to avoid the S-03 trigger dgp-004
  hit. That is the loop working — worth preserving as a pattern.
- **A section root without an index doc 404s silently.** Multi-instance
  `plugin-content-docs` gives no build error for a missing index; the build
  is green and the navbar works, so only a direct probe of the bare route
  finds it.
- **Deferred-out-of-scope ≠ blocked.** Conflating the two (D-2) makes a
  complete phase look incomplete and misdirects the next phase's attention.

## Next Phase Focus

**Recommended next phase: `phase-docs-site-hardening`**

Top 3 priority areas:

1. **Close this phase's debt** — add `docs/learn/README.md` (fixes the
   `/docs/learn-internals/` 404); perform or formally waive the dgp-004 manual
   review; resolve the duplicate `10-` prefix; migrate the deprecated
   `onBrokenMarkdownLinks` config ahead of Docusaurus v4.
2. **Harden the deploy gate** — the phase deliberately shipped a *minimal* gate
   (decision D-002: build + `onBrokenLinks: throw`). The donor's fuller
   `release:check` chain (built-site validation, external link checks, browser
   gate) was deferred and is the natural next increment now that the pipeline
   is proven green end-to-end.
3. **Fix the state-staleness class of bug** (D-1/D-2/D-3) — this is the phase's
   most transferable finding and affects every future KBD phase, not just docs:
   re-probe external reality at reflect, clear blockers on observation, sync
   position at phase boundaries.

**Needs human review before proceeding:**

- **dgp-009 brand port** (`--km-*` token contract) — still unanswered from
  assess. It is out of scope for the closed phase; it needs either a goals
  amendment in a future phase or an explicit drop. This is a *decision*, not a
  blocker.
- Whether to adopt a custom domain (decision D-001 chose the default Pages URL,
  parameterized so the switch is env + CNAME only).

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation. Note
specifically that this phase's `assessment.md` contains a superseded
post-execution section (see D-1) — trust this reflection's verified table over
that section's Goal 2 verdict.
