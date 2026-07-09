# Analysis — phase-bdd-video-proof

_Analyzed: 2026-07-09_

## Mode

**Stack specified.** Two independent stacks (cucumber-js / TypeScript and
cucumber-rs / Rust) — no stack discovery needed. Each is a first-class
target; the phase ships parallel skill families.

## Landscape summary

Three research threads (see decision-log for tool traces) produced these
key facts:

1. **cucumber-js 13.0.0** is current (2026-06-02); our `^11.0.0` is two
   majors behind. Bump during refactor.
2. **cucumber-rs 0.23.0** is current (2026-04-23); MSRV 1.88. `#[async_trait]`
   was dropped in 0.21; the World is written as plain `async fn`.
3. **`playwright-bdd` (vitalets)** at 9.2.0 is the canonical cucumber-js
   ↔ Playwright bridge. It converts BDD scenarios into native Playwright
   tests, giving us the Playwright runner (workers, retries, trace viewer,
   video, HTML report) for free instead of hand-rolling hooks.
4. **`thirtyfour` 0.37.2** is the pragmatic default browser driver for
   cucumber-rs (modern typed API, actively released July 2026, multi-browser
   via WebDriver). `headless_chrome` wins on downloads/stars but is
   Chrome-only + CDP. `fantoccini` is older but stable.
5. **Video capture**: Playwright records WebM (VP8) natively;
   `ffmpeg -c copy` remuxes to MP4 losslessly. No Rust browser driver has
   native MP4 — all require ffmpeg sidecar.
6. **Local certification bundle**: no OSS prior art. We define it.
7. **Immutable-tests enforcement**: no OSS tooling. Our
   `shared/scripts/protect-tests.sh` is ahead of the state of the art;
   promote it as the reference implementation.
8. **Flake quarantine**: cucumber-js native `--retry-tag-filter @flaky` is
   the primitive; Trunk.io's flake-budget workflow is the pattern to steal.

## Per-gap candidate evaluation

### G-01 — cucumber-js authoring skill

| Candidate | Version | Verdict | Evidence |
|-----------|---------|---------|----------|
| `@cucumber/cucumber` | 13.0.0 | **ADOPT** | current stable, ~2.2M weekly npm; direct upgrade path from our v11 |
| `playwright-bdd` (vitalets) | 9.2.0 | **ADOPT** | canonical bridge; ~410k weekly; converts BDD → Playwright test runner |
| `tsx` | latest | **ADOPT** | official cucumber-js docs now recommend `tsx` over `ts-node/esm` |
| direct `@cucumber/cucumber` + `playwright-core` in hooks | — | **SKIP (legacy)** | prior state; loses Playwright runner benefits |

**Build vs adopt call:** Refactor existing `skills/testing/bdd-testing/`
into a portable `bdd-cucumber-js` skill using the three adoptions above.
Keep the current directory as a compat alias or fork depending on impact —
decide during `/kbd-plan`. No net-new authoring library needed.

### G-02 — cucumber-rs authoring skill

| Candidate | Version | Verdict | Evidence |
|-----------|---------|---------|----------|
| `cucumber` | 0.23.0 | **ADOPT** | current stable, 230k/month, MSRV 1.88; native `async fn` in traits |
| `thirtyfour` | 0.37.2 | **ADOPT (primary browser driver)** | most recently released (2026-07-05); modern typed WebDriver API; multi-browser |
| `fantoccini` | 0.22.1 | **DOCUMENT (secondary)** | more stars but older; mention as alternative for teams already using it |
| `headless_chrome` | 1.0.22 | **DOCUMENT (special-purpose)** | 3× the monthly downloads of WebDriver crates; use when CDP features (screencast, network intercept) are required; Chrome-only |

**Build vs adopt call:** Ship a new `skills/testing/bdd-cucumber-rs/` skill.
Primary path: `cucumber` + `tokio` + `thirtyfour` + `ffmpeg` sidecar for
video. Document `headless_chrome` for CDP-specific scenarios.

### G-03 — BDD lifecycle loop skill

No adoptable OSS project — the create → run → triage → maintain loop is
green field. Steal patterns from:

- **cucumber-js `--retry-tag-filter @flaky`** — primitive for flake handling
- **Trunk.io's flake-budget model** — wrap `@flaky` with a
  `flake-budget.json` (max N tagged scenarios, max age N days) enforced in
  CI. This is the operative piece missing everywhere else.
- **Standard Cucumber Steps (Rob Moffat)** — canonical step library so
  agent-generated `.feature` files are executable without new glue.
- **Immutable-tests rule (our BDD-006)** — already ahead of OSS; promote
  `shared/scripts/protect-tests.sh` as reference impl.

**Build vs adopt call:** Ship `skills/testing/bdd-lifecycle-loop/` as pure
documentation + a `flake-budget.sh` script + a CI guard
(`test-file-diff-guard.sh`). No external adoption; primitives already in
our repo.

### G-04 — video-proof certification skill

Existing `skills/testing/bdd-video-proof/` covers IPFS pinning. Gap is the
**local, self-contained certification bundle** format. No OSS prior art
found in searches for "test attestation", "BDD certification bundle", or
"test evidence bundle" — we define the format.

**Bundle format (proposed):**

```
docs/certifications/<module>/<sha>/
├── manifest.json          # SHA-256 of each artifact + git SHA + module fingerprint
├── cucumber-report.json   # raw cucumber output
├── videos/
│   ├── scenario-01.mp4    # ffmpeg-remuxed from Playwright WebM
│   └── ...
├── screenshots/
│   ├── scenario-01/*.png
│   └── SCREENSHOTS.md     # manifest with hash per screenshot
└── report.html            # human-readable index
```

Signing method: **git SHA + SHA-256 of manifest.json**. GPG/Sigstore is a
follow-up phase (documented in an open question, not shipped now).

**Build vs adopt call:** Extend `bdd-video-proof` with a
`scripts/mint-certification-bundle.sh` that assembles the layout above.
Keep IPFS pinning as an optional target.

### G-05 — visual + non-visual scenario examples

No candidate needed — just reference feature files under each skill's
`references/examples/`. Two examples per skill (cucumber-js and
cucumber-rs): one HTTP-only, one browser-driven. **Build.**

### G-06 — Integrate with existing BDD skills

15 BDD-* future-work docs describe the target architecture. Cross-reference
BDD-005 (testid drift), BDD-006 (immutable tests), BDD-007 (candidate
drafts) from new skill READMEs. Update `CLAUDE.md` immutable-tests section
to point at `bdd-lifecycle-loop` skill. **Build (documentation).**

### G-07 — Cross-platform install + validation

Each new/refactored skill needs `scripts/smoke-test.sh` that runs a minimal
1-scenario feature end-to-end. Run `npm run validate:strict` before phase
close. **Build.**

## Decisions summary (build / adopt / skip)

| Gap | Verdict | Primary candidate |
|-----|---------|-------------------|
| G-01 | **REFACTOR + ADOPT** | `@cucumber/cucumber` 13 + `playwright-bdd` 9 + `tsx` |
| G-02 | **BUILD + ADOPT** | `cucumber` 0.23 + `thirtyfour` 0.37 |
| G-03 | **BUILD** | our patterns; steal Trunk flake-budget model |
| G-04 | **BUILD** | local cert-bundle format (net-new) |
| G-05 | **BUILD** | example feature files |
| G-06 | **BUILD (docs)** | cross-ref BDD-005/006/007 |
| G-07 | **BUILD** | smoke-test.sh per skill |

## Open questions (deferred to /kbd-plan)

1. **Fork vs refactor `skills/testing/bdd-testing/`?**
   - Fork = new `bdd-cucumber-js` skill; keep `bdd-testing` as thin compat
     alias so downstream (ssr-frontend) doesn't break.
   - Refactor = edit in place; ssr-frontend adapts.
   - Recommendation: **fork**. Lower risk to downstream projects.

2. **GPG / Sigstore signing of cert bundles.** Defer — start with git SHA +
   SHA-256 manifest hash. Follow-up phase if reviewers demand it.

3. **cucumber-rs 0.20 → 0.23 migration guide.** Skill needs one page
   describing the `#[async_trait]` removal (0.21) and MSRV bumps. Ship as
   `references/migration-from-0.20.md`.

4. **`bdd-lifecycle-loop` vs KBD orchestration coupling.** Ship as pure
   documentation + scripts. Do NOT couple to `/kbd-plan` — keep the skill
   usable in projects that don't run KBD.

## Confidence

**High.** All three research threads returned within budget (13, 14, 9 tool
calls respectively; all under 15 minutes). No contested stack (independent
stacks), no library candidates that failed the maintenance filter.
