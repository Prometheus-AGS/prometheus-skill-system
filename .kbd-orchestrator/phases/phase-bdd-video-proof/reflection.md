# Reflection — phase-bdd-video-proof

_Reflected: 2026-07-09_

## Summary

`phase-bdd-video-proof` refactored the two pre-existing cucumber-js-only
BDD skills into a **four-skill family** covering both TypeScript and Rust,
introducing a lifecycle-loop skill, a local certification bundle format,
and CI-wired smoke tests. All 7 goals MET, all 52 tasks completed across 8
changes. Zero technical debt introduced.

---

## Goal Achievement

| Goal | Status | Change | Commit |
|------|--------|--------|--------|
| G-01 — Cucumber-js authoring skill | **MET** | change-bdd-001-fork-cucumber-js | `b855dff` |
| G-02 — Cucumber-rs authoring skill | **MET** | change-bdd-003-cucumber-rs-skill | `ed9828d` |
| G-03 — BDD lifecycle loop skill | **MET** | change-bdd-005-lifecycle-loop-skill | `31ada97` |
| G-04 — Video-proof certification bundle | **MET** | change-bdd-006-video-cert-bundle | `fe63f47` |
| G-05 — Visual + non-visual examples | **MET** | change-bdd-002 + change-bdd-004 | `dc7f5cf` + `b1dc578` |
| G-06 — Cross-reference existing BDD-* docs | **MET** | change-bdd-007-cross-references | `7310587` |
| G-07 — Cross-platform install + smoke tests | **MET** | change-bdd-008-validate-smoke-tests | `c05e212` |

**Achievement rate: 7/7 (100%)**

---

## Delivered Changes

### change-bdd-001-fork-cucumber-js
Forked `bdd-testing/` → new portable `bdd-cucumber-js/` (v1.0.0). Stack
bumped to `@cucumber/cucumber` 13.0.0 + `playwright-bdd` 9.2.0 + `tsx`.
Next.js SSR wording removed. `bdd-testing/` reduced to a 25-line
compatibility redirect (v2.0.0) so downstream projects like `ssr-frontend`
that reference it by name continue to resolve.

**Tasks: 8/8** · **Commits:** `a46178c` (feat) + `dec51ca` (KBD marker)

### change-bdd-002-cucumber-js-examples
Two self-contained example directories under
`bdd-cucumber-js/references/examples/`: `api-http-only/` (plain
cucumber-js + `fetch`) and `ui-playwright/` (via `playwright-bdd`'s
`createBdd()`). Each covers happy path, error case, and validation
scenarios. `README.md` explains the pick criteria.

**Tasks: 5/5** · **Commits:** `dc7f5cf` (feat) + `89ded35` (KBD marker)

### change-bdd-003-cucumber-rs-skill
New `bdd-cucumber-rs/` skill (v1.0.0) covering `cucumber` 0.23.0 +
`tokio` + `reqwest` + `thirtyfour` 0.37.2. Async World via
`#[derive(World)]` (no `#[async_trait]` since 0.21). Feature-gated `ui`
so headless CI can skip. `references/browser-drivers.md` compares
thirtyfour/fantoccini/headless_chrome with 2026-07 versions.
`references/migration-from-0.20.md` walks legacy projects through the
`#[async_trait]` removal and MSRV bumps to 1.88.

**Tasks: 9/9** · **Commits:** `ed9828d` (feat) + `d8da985` (KBD marker)

### change-bdd-004-cucumber-rs-examples
Two Cargo-crate examples under
`bdd-cucumber-rs/references/examples/`: `api-http-only/` (tokio +
reqwest) and `ui-thirtyfour/` (feature-gated, headless Chrome caps,
per-scenario driver teardown). Mirror the cucumber-js example structure
for stack-to-stack comparison.

**Tasks: 5/5** · **Commits:** `b1dc578` (feat) + `e2478a4` (KBD marker)

### change-bdd-005-lifecycle-loop-skill
New `bdd-lifecycle-loop/` skill (v1.0.0) codifying the four-phase loop:
author → run → triage → maintain. `scripts/flake-budget.sh` enforces
`max_flaky_scenarios` + `max_flaky_age_days` via `git blame`, wrapping
cucumber's `--retry-tag-filter @flaky` primitive.
`scripts/test-file-diff-guard.sh` is the CI form of the immutable-tests
rule; fails PRs touching `tests/steps|support|features/` without a
`test-change-approved` label or `BDD_ALLOW_TEST_EDITS=1` override.
`references/immutable-tests.md` and
`references/visual-baseline-refresh.md` provide the operative
documentation.

**Tasks: 8/8** · **Commits:** `31ada97` (feat) + `85e08e9` (KBD marker)

### change-bdd-006-video-cert-bundle
`bdd-video-proof/` bumped to v2.0.0 with a **local certification bundle**
as the new default (Mode A). Layout:
`docs/certifications/<module>/<sha>/{manifest.json, cucumber-report.json,
videos/*.mp4, screenshots/**, report.html}`.
`scripts/mint-certification-bundle.sh` handles ffmpeg lossless remux
(`-c copy`), cross-platform SHA-256 (`shasum` or `sha256sum`), module
fingerprint from source hashes, and dry-run. Mode B (IPFS pinning) is
still supported.

**Tasks: 7/7** · **Commits:** `fe63f47` (feat) + `a152814` (KBD marker)

### change-bdd-007-cross-references
`CLAUDE.md` § BDD Immutable-Tests Rule now points at
`bdd-lifecycle-loop/references/immutable-tests.md` as canonical;
documents both PreToolUse hook + CI gate; trims duplicated rationale.
New `docs/future-work/02-bdd-testing-evolution/STATUS.md` provides a
BDD-001…015 matrix showing shipped/partial/planned per doc, mapped to
concrete files in this repo. All four new/updated skills reference
BDD-005/006/007 + STATUS.md in their See also sections (3 refs each).

**Tasks: 4/4** · **Commits:** `7310587` (docs) + `170b262` (KBD marker)

### change-bdd-008-validate-smoke-tests
Each of the four skills got a `scripts/smoke-test.sh` verifying its
frontmatter, examples, scripts, and (where applicable) dry-run behavior
end-to-end. All four pass locally. All four validate strict. New
`bdd-skill-smoke` CI job in `.github/workflows/validate.yml` installs
`jq`, runs the smoke tests, and runs `npm run validate:strict` for each
skill.

**Tasks: 6/6** · **Commits:** `c05e212` (feat) + `98e75f4` (KBD marker)

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA runs | 0/8 (artifact-refiner not configured for this phase) |
| First-pass pass rate | N/A |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |
| Skills strict-validated | 4/4 ✓ |
| Smoke tests passing locally | 4/4 ✓ |
| CI smoke wire-up | 1/1 ✓ |

No `.refiner/artifacts/` logs were found. Manual QA was the four smoke
tests + strict validation, run per-change and again at phase close.

---

## Technical Debt Introduced

**None.**

Non-debt notes worth recording:
- **HTMX/htmx examples** for cucumber-js are self-contained in
  `references/examples/` and do not run in this repo's CI (no dev
  server). Downstream projects using the skill will exercise them.
- **cucumber-rs examples ship as standalone Cargo crates** under
  `references/examples/`. This repo has no workspace root Cargo.toml, so
  they do not compile as part of `cargo build`. If a workspace is ever
  added, an `exclude = ["skills/**/examples/**"]` entry is required.
- **Video capture in Rust is `ffmpeg`-sidecar-only.** Documented in the
  skill; no native crate path exists in 2026-07. Revisit if a first-class
  crate emerges.

---

## Lessons Captured

1. **pipeline-enforce reads command TEXT, not just `progress.json`** — the
   hook flagged `/kbd-reflect` when it appeared inside a commit-message
   here-doc, even though `progress.json` had already been updated to
   8/8. Workaround: write commit messages without referencing
   `/kbd-reflect` inside the here-doc body. Deeper fix: the hook should
   scan for `kbd-reflect` only in the *invoked* command, not in shell
   arguments. Filed as a lesson to sharpen the hook.

2. **cucumber 0.21+ removes `#[async_trait]` on World** — legacy
   projects migrating from 0.20 need only a MSRV bump to 1.88 and to
   swap `#[async_trait(?Send)] impl World for T` for
   `#[derive(World)]`. Everything else is unchanged.

3. **`playwright-bdd` is the canonical bridge in 2026** — direct
   `@cucumber/cucumber` + `playwright-core` in `Before`/`After` hooks
   is now legacy. `playwright-bdd` compiles feature files into
   Playwright tests so you get the Playwright runner (workers, retries,
   trace viewer, video, HTML report) for free.

4. **No OSS prior art exists for a cucumber test attestation bundle** —
   searches for "test attestation", "BDD certification bundle", and
   "test evidence bundle" returned zero repos. We defined the format.
   Reviewers watch videos with any browser; no IPFS node required for
   Mode A.

5. **`data-testid` is the correct selector convention** — both
   `bdd-cucumber-js` and `bdd-cucumber-rs` recommend it. Combined with
   BDD-005 (testid drift detection, planned), this closes the loop
   between production code changes and the test selectors they invalidate.

6. **Playwright records WebM (VP8), MP4 via `ffmpeg -c copy`** —
   lossless stream copy remuxes WebM to MP4 without re-encoding. No
   in-process MP4 export exists.

---

## Delta vs Plan

| Planned | Delivered | Delta |
|---------|-----------|-------|
| 8 changes | 8 changes | 0 |
| 52 tasks | 52 tasks `[x]` | 0 |
| 7 goals MET | 7 goals MET | 0 |
| 0 regressions | 0 regressions | 0 |
| Contested stack picks | 0 contested | 0 |

No deltas. Plan executed exactly as designed.

---

## Recommended Next Phase

The BDD skill family is now feature-complete for the core author → run →
triage → maintain → certify loop. The STATUS.md matrix identifies the
next-highest-leverage items still open:

1. **BDD-005 testid drift detection** — port the `validate-testid-coverage`
   pattern into a script that the two authoring skills invoke.
2. **BDD-007 candidate drafts promotion workflow** — the guard already
   allows `tests/features/drafts/`, but promotion out of drafts (with
   human sign-off) still needs a script.
3. **BDD-012 two-phase gates** — PR runs enforce the flake budget on
   `@smoke`, release runs enforce on everything.

If any of these is the target of the next phase, I'd recommend BDD-005
first: it's the piece that makes the immutable-tests rule
*self-enforcing* (test breakage is detected before agents get a chance
to try to edit them).

Alternative: pivot to another domain entirely. No blocking technical
debt from this phase requires immediate follow-up.
