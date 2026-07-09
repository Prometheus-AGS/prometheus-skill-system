# Decision Log — phase-bdd-video-proof

## 2026-07-09T13:15:00Z — Stack picks

**Decision:** Two parallel stacks — `@cucumber/cucumber` 13 +
`playwright-bdd` 9 + `tsx` for TypeScript; `cucumber` 0.23 + `thirtyfour`
0.37 + `ffmpeg` for Rust.

**Provenance:** Research (three subagent threads).
**Confidence:** High.
**Elicitation ID:** none — no contested stack.

## 2026-07-09T13:15:00Z — Rust browser driver

**Decision:** `thirtyfour` 0.37.2 primary. `fantoccini` and `headless_chrome`
documented as alternatives.

**Rationale:** `thirtyfour` shipped 2026-07-05 (most active); modern typed
async WebDriver API; multi-browser. `headless_chrome` wins downloads/stars
but is Chrome-only + CDP-specific — appropriate for advanced cases only.
`fantoccini` is stable but older API and last release 2026-02.

**Provenance:** Research.
**Confidence:** Medium (community drift toward CDP is real; revisit in 12
months if `headless_chrome` becomes dominant across all use cases).

## 2026-07-09T13:15:00Z — Video format

**Decision:** WebM (VP8) capture via Playwright / CDP; MP4 remux via
`ffmpeg -c copy` (lossless stream copy).

**Rationale:** No Rust or JS browser driver ships native MP4. `ffmpeg`
stream copy avoids re-encode cost and preserves quality.

**Provenance:** Research.
**Confidence:** High.

## 2026-07-09T13:15:00Z — Certification bundle format

**Decision:** Local `docs/certifications/<module>/<sha>/` layout with
`manifest.json` (SHA-256 per artifact + git SHA + module fingerprint) +
`cucumber-report.json` + `videos/*.mp4` + `screenshots/**/*.png` +
`report.html`. Signing = git SHA + SHA-256. IPFS pinning stays optional.

**Rationale:** No OSS prior art found for cucumber test attestation
bundles. Adopting a simple, reviewable format keeps reviewers unblocked
without requiring an IPFS node or an external signing service.

**Provenance:** Research (zero adoptable candidates).
**Confidence:** High for format; GPG/Sigstore deferred to a follow-up
phase.

## 2026-07-09T13:15:00Z — Immutable-tests rule enforcement

**Decision:** Promote `shared/scripts/protect-tests.sh` as the reference
implementation. Ship a companion `test-file-diff-guard.sh` for CI. New
`bdd-lifecycle-loop` skill documents both.

**Rationale:** No OSS tool enforces "code agents may not edit tests" via
hooks/lint/CI. Our PreToolUse hook is ahead of the state of the art.

**Provenance:** Research.
**Confidence:** High.

## 2026-07-09T13:15:00Z — Flake budget model

**Decision:** Wrap cucumber-js `--retry-tag-filter @flaky` primitive with
a `flake-budget.json` (max N tagged scenarios, max age N days). CI fails
if budget exceeded.

**Rationale:** Trunk.io's SaaS embodies this pattern; the primitive is
already in cucumber-js. Adopt the pattern, build the operative piece
locally.

**Provenance:** Research.
**Confidence:** Medium (need to validate exact budget defaults in
implementation).
