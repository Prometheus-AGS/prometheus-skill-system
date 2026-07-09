# Plan — phase-bdd-video-proof

_Generated: 2026-07-09_

## Change Backend
OpenSpec (`openspec/` directory present at project root)

## Overview

8 changes, ordered to build **cucumber-js first (with existing tests as
reference)**, then cucumber-rs, then the cross-cutting loop and
certification, then integration and validation. Two "adjust existing skill"
changes bracket the phase: 001 forks `bdd-testing` into `bdd-cucumber-js`,
008 wires the smoke tests.

## Changes

| # | Change ID | Goal | Description | Library | Tasks |
|---|-----------|------|-------------|---------|-------|
| 1 | `change-bdd-001-fork-cucumber-js` | G-01 | Fork `skills/testing/bdd-testing/` → `skills/testing/bdd-cucumber-js/`; bump to `@cucumber/cucumber` 13 + `playwright-bdd` 9 + `tsx`; portable wording | cand-001, cand-002, cand-003 | 8 |
| 2 | `change-bdd-002-cucumber-js-examples` | G-05 | Add `references/examples/` to `bdd-cucumber-js`: one HTTP-only feature + one Playwright-driven feature with steps | — | 5 |
| 3 | `change-bdd-003-cucumber-rs-skill` | G-02 | New `skills/testing/bdd-cucumber-rs/`: `cucumber` 0.23 + `thirtyfour` 0.37; async World; feature/step patterns; `references/migration-from-0.20.md` | cand-004, cand-005 | 9 |
| 4 | `change-bdd-004-cucumber-rs-examples` | G-05 | Add `references/examples/` to `bdd-cucumber-rs`: HTTP-only + `thirtyfour`-driven examples | — | 5 |
| 5 | `change-bdd-005-lifecycle-loop-skill` | G-03 | New `skills/testing/bdd-lifecycle-loop/`: author→run→triage→maintain loop; flake-budget.sh (wraps `--retry-tag-filter @flaky`); test-file-diff-guard.sh; documents `protect-tests.sh` | — | 8 |
| 6 | `change-bdd-006-video-cert-bundle` | G-04 | Extend `skills/testing/bdd-video-proof/` with local cert-bundle format: `docs/certifications/<module>/<sha>/` layout; `mint-certification-bundle.sh` (ffmpeg remux + SHA-256 manifest); keep IPFS optional | cand-008 | 7 |
| 7 | `change-bdd-007-cross-references` | G-06 | Cross-ref BDD-005/006/007 from every new skill README; update `CLAUDE.md` immutable-tests section to point at `bdd-lifecycle-loop`; index in `docs/future-work/02-bdd-testing-evolution/STATUS.md` | — | 4 |
| 8 | `change-bdd-008-validate-smoke-tests` | G-07 | Per-skill `scripts/smoke-test.sh` (minimal 1-scenario end-to-end); run `npm run validate:strict` on all four new/refactored skills; add to CI workflow | — | 6 |

**Total tasks: 52**

## Execution Order Rationale

- **001 before 002**: examples land inside the forked skill.
- **003 before 004**: same reason for the Rust side.
- **001/003 in parallel possible** (independent skills), but ordered sequentially for clean commit sequence.
- **005 after 001+003**: the loop skill references both authoring skills, so both must exist first.
- **006 can start after 001** (video-proof extension is independent of Rust skill); ordered here to keep video work grouped after loop.
- **007 after all skills exist**: cross-references need target skills to link to.
- **008 last**: smoke tests validate everything shipped.

## Goal Coverage Map

| Goal | Covered by |
|------|-----------|
| G-01 cucumber-js authoring | 001 |
| G-02 cucumber-rs authoring | 003 |
| G-03 lifecycle loop | 005 |
| G-04 video-proof cert bundle | 006 |
| G-05 visual + non-visual examples | 002, 004 |
| G-06 BDD-005/006/007 integration | 007 |
| G-07 cross-platform install + smoke tests | 008 |

## Apply Commands

```
/kbd-apply change-bdd-001-fork-cucumber-js
/kbd-apply change-bdd-002-cucumber-js-examples
/kbd-apply change-bdd-003-cucumber-rs-skill
/kbd-apply change-bdd-004-cucumber-rs-examples
/kbd-apply change-bdd-005-lifecycle-loop-skill
/kbd-apply change-bdd-006-video-cert-bundle
/kbd-apply change-bdd-007-cross-references
/kbd-apply change-bdd-008-validate-smoke-tests
```

## First Change

```
/kbd-apply change-bdd-001-fork-cucumber-js
```
