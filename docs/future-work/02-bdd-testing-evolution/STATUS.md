# BDD Testing Evolution — Status Matrix

_Last updated: 2026-07-09 during `phase-bdd-video-proof`_

Status legend:

- **shipped** — the doc's proposal is implemented in this repo
- **partially-shipped** — the operative form is in a skill but the doc's
  full scope isn't complete
- **ready** — spec is stable and awaiting implementation
- **planned** — sketched but not yet stabilized

## Matrix

| ID | Title | Doc status | Ship status | Implemented in |
|----|-------|------------|-------------|----------------|
| [BDD-001](BDD-001-manifest-dual-key-cleanup.md) | Manifest dual-key cleanup migration | ready | partially-shipped | `openspec/changes/archive/change-003-bdd001-manifest-dual-key-cleanup/` |
| [BDD-002](BDD-002-flake-quarantine.md) | Flake quarantine system | ready | **shipped** | `skills/testing/bdd-lifecycle-loop/scripts/flake-budget.sh` |
| [BDD-003](BDD-003-ipfs-pin-sweep.md) | IPFS pin sweep job | ready | partially-shipped | `skills/testing/bdd-video-proof/references/IPFS.md` (Mode B) |
| [BDD-004](BDD-004-video-skill-productization.md) | BDD video skill productization | planned | **shipped** | `skills/testing/bdd-video-proof/` v2.0 (local cert bundle) |
| [BDD-005](BDD-005-testid-drift-detection.md) | testid drift detection | ready | planned | Referenced in `bdd-cucumber-js` / `bdd-cucumber-rs`; script not yet ported |
| [BDD-006](BDD-006-immutable-tests-rule.md) | Immutable-tests CLAUDE.md rule | ready | **shipped** | `CLAUDE.md` § BDD Immutable-Tests Rule; `shared/scripts/protect-tests.sh` (hook); `skills/testing/bdd-lifecycle-loop/scripts/test-file-diff-guard.sh` (CI) |
| [BDD-007](BDD-007-candidate-test-drafts.md) | Candidate test drafts | ready | partially-shipped | `test-file-diff-guard.sh` excludes `tests/features/drafts/`; the promotion workflow itself is documented in `bdd-lifecycle-loop` but not scripted |
| [BDD-008](BDD-008-pk-codegraph-extraction.md) | pk-codegraph extraction | ready | planned | — |
| [BDD-009](BDD-009-pk-codegraph-runtime-coverage.md) | pk-codegraph runtime coverage | planned | planned | — |
| [BDD-010](BDD-010-impact-set-hash-runner.md) | Impact-set hash test runner | planned | planned | — |
| [BDD-011](BDD-011-environment-hash-augmentation.md) | Environment hash augmentation | planned | planned | — |
| [BDD-012](BDD-012-two-phase-gates.md) | Two-phase test gates (PR fast / release thorough) | planned | planned | — |
| [BDD-013](BDD-013-story-feature-contract.md) | User-story to feature contract | planned | planned | Adjacent to `bdd-lifecycle-loop` outside-in section |
| [BDD-014](BDD-014-feedback-aggregation-in-docs.md) | Feedback aggregation in docs site | planned | planned | — |
| [BDD-015](BDD-015-feedback-to-draft-scenario.md) | Feedback record to draft-scenario emitter | planned | planned | Would connect to BDD-007 `drafts/` |

## Shipped in phase-bdd-video-proof (2026-07-09)

Four skill families now bracket the BDD lifecycle:

| Skill | Version | Purpose |
|-------|---------|---------|
| [`bdd-cucumber-js`](../../../skills/testing/bdd-cucumber-js/SKILL.md) | 1.0.0 | Author + run cucumber-js 13 + playwright-bdd + tsx |
| [`bdd-cucumber-rs`](../../../skills/testing/bdd-cucumber-rs/SKILL.md) | 1.0.0 | Author + run cucumber 0.23 + thirtyfour |
| [`bdd-lifecycle-loop`](../../../skills/testing/bdd-lifecycle-loop/SKILL.md) | 1.0.0 | author → run → triage → maintain workflow; flake budget; immutable-tests CI guard |
| [`bdd-video-proof`](../../../skills/testing/bdd-video-proof/SKILL.md) | 2.0.0 | Local cert bundle (default) or IPFS pinning (legacy Mode B) |

The prior `bdd-testing` skill (v2.0.0) is now a thin redirect to the four
above so downstream projects that reference it by name keep resolving.

## Next candidates for implementation

Ranked by leverage:

1. **BDD-005 testid drift detection** — the two new authoring skills
   already recommend `data-testid` selectors. Porting the drift detection
   script closes the loop.
2. **BDD-007 candidate drafts promotion workflow** — the guard already
   allows `tests/features/drafts/`, but promotion out of drafts (with
   human sign-off) still needs a script.
3. **BDD-012 two-phase gates** — pairs naturally with the flake budget:
   PR runs enforce budget on `@smoke`, release runs enforce on
   everything.
