# Assessment — phase-bdd-video-proof

_Assessed: 2026-07-09_

## Project identity

`prometheus-skill-pack` — enterprise skill collection with two BDD-adjacent
skills already in `skills/testing/`. `.kbd-orchestrator/project.json` is
absent (warned but non-blocking). Change backend: OpenSpec (`openspec/`
present).

## Executive summary

Two prior BDD skills exist but are project-narrow (cucumber-js only, Next.js
SSR wording), incomplete (no lifecycle loop, no Rust coverage, no reusable
certification format), and don't reference each other. 15 BDD-* future-work
docs (`BDD-001…BDD-015`) sketch the target architecture but none are shipped.
The `docs/future-work/02-bdd-testing-evolution/` set already answers most
open questions we would otherwise ask.

## Existing surfaces (what's already here)

| Path | State | Notes |
|------|-------|-------|
| `skills/testing/bdd-testing/SKILL.md` (233 lines) | Present, v1.0.0 | cucumber-js + Playwright, Next.js SSR wording, 0 Rust references |
| `skills/testing/bdd-testing/scripts/run-bdd.sh` | Present | project-scoped runner |
| `skills/testing/bdd-testing/scripts/generate-report.sh` | Present | HTML report |
| `skills/testing/bdd-testing/references/world-pattern.md` | Present | CustomWorld guide |
| `skills/testing/bdd-testing/references/video-recording.md` | Present | Playwright video config |
| `skills/testing/bdd-video-proof/SKILL.md` (70 lines) | Present, v1.0.0 | scripts NOT in this repo — points at `scripts/run-video-proof.ts` that lives in `ssr-frontend` |
| `skills/testing/bdd-video-proof/references/{SETUP,IPFS}.md` | Present | IPFS pinning workflow |
| `docs/future-work/02-bdd-testing-evolution/BDD-001…BDD-015.md` | 15 docs | Full architecture backlog: dual-key cleanup, flake quarantine, IPFS pin sweep, video productization, testid drift, immutable-tests rule, candidate drafts, pk codegraph, impact-set hash, two-phase gates, story-feature contract, feedback aggregation |
| `docs/certifications/` | ABSENT | Target output path for G-04 does not exist |
| `agents/e2e-runner*.md` | ABSENT in this repo | Referenced in registry but not present locally |
| `package.json` `@cucumber/cucumber` | ^11.0.0 present | Recent, works with G-01 |

## Gap analysis vs G-01 through G-07

### G-01 — Cucumber-js authoring skill
**Status: PARTIAL — refactor, don't rebuild.**
- Existing `skills/testing/bdd-testing/` already covers cucumber-js + Playwright + video.
- Gap: strongly worded around Next.js SSR project structure; not portable. No profile/reporter matrix. No ESM vs CJS vs ts-node guidance. No "when to choose HTTP-only vs Playwright" table.
- **Action:** refactor `bdd-testing` into a portable, well-documented cucumber-js skill and rename references accordingly (or fork to a new `bdd-cucumber-js` skill and keep `bdd-testing` as a compatibility umbrella).

### G-02 — Cucumber-rs authoring skill
**Status: NOT MET.**
- 0 Rust references in any existing testing skill. No `bdd-cucumber-rs` directory.
- Downstream Rust crates (sovereign-sync, prometheus-research, storage-provider) use `#[tokio::test]` + `cucumber` crate ad-hoc in one place but no skill guides the pattern.
- **Action:** ship a new `skills/testing/bdd-cucumber-rs/` with async World, tokio + reqwest patterns for HTTP, `fantoccini`/`thirtyfour` patterns for browser.

### G-03 — BDD lifecycle loop skill
**Status: NOT MET.**
- No skill describes the create → run → triage → maintain loop as a workflow. The immutable-tests rule (BDD-006) exists as prose in `CLAUDE.md` and a future-work doc, but there's no operative skill that agents invoke.
- **Action:** ship `skills/testing/bdd-lifecycle-loop/` that codifies the loop, references the immutable-tests rule, integrates flake quarantine (BDD-002).

### G-04 — Video-proof certification skill
**Status: PARTIAL — skill exists, but the deliverable it produces is IPFS-pinned, not a local certification bundle.**
- `bdd-video-proof` today writes to `docs/videos-manifest.json` + IPFS CIDs. That's a good audit trail but requires an IPFS node.
- Gap: no local `docs/certifications/<module>/<sha>/` layout. No self-contained bundle (JSON report + video + screenshot manifest + SHA + fingerprint) that reviewers can inspect without IPFS.
- **Action:** extend `bdd-video-proof` (or add a sibling `bdd-video-cert`) with the local certification-bundle format described in G-04. Keep IPFS as an optional target.

### G-05 — Visual + non-visual scenario examples
**Status: NOT MET.**
- No `references/examples/` files in either existing skill. Nothing shows the same behavior tested two ways.
- **Action:** add example feature files under each new/refactored skill demonstrating the choice.

### G-06 — Integrate with existing BDD skills (BDD-005/006/007)
**Status: NOT MET.**
- The 15 future-work docs describe the target; only BDD-006 (immutable-tests rule) is enforced today, and only via CLAUDE.md prose.
- Gap: the new skills need to explicitly cite BDD-006 and provide the operative form of BDD-005 (testid drift detection) and BDD-007 (candidate test drafts).
- **Action:** cross-reference BDD-* docs from the new skill READMEs; update CLAUDE.md to point at the new lifecycle-loop skill.

### G-07 — Cross-platform install + validation
**Status: PARTIAL — existing skills probably validate, but no smoke test.**
- `npm run validate:strict skills/testing/bdd-testing` should be re-run to confirm.
- No smoke script that runs a minimal cucumber scenario against a fixture project to prove the skill's instructions actually work.
- **Action:** every new/refactored skill needs an executable `scripts/smoke-test.sh` that runs a minimal 1-scenario feature end-to-end.

## Open questions

1. **Fork vs refactor `bdd-testing`?** Decide during `/kbd-plan` whether to
   rename existing skills (`bdd-testing` → `bdd-cucumber-js`, breaking
   version bump) or keep names and refactor in place. Recommendation: fork
   into a new skill and leave `bdd-testing` as a slim compatibility redirect,
   because downstream projects (ssr-frontend) already reference it by name.

2. **Certification bundle signing?** G-04 mentions "signed" — is that
   git-signed (SHA + tag), GPG-signed, or Sigstore? Recommendation: start
   with git SHA + SHA-256 hash of the bundle contents recorded in a
   `manifest.json`; treat GPG/Sigstore as a follow-up phase.

3. **Should `bdd-lifecycle-loop` be a KBD-adjacent skill or a native KBD phase
   generator?** i.e., does it emit `/kbd-plan`-style changes, or is it a
   pure documentation skill? Recommendation: pure documentation skill for
   now, with a script that generates a minimal feature+step scaffolding — do
   not couple it to KBD orchestration.

4. **Rust crate versions to target for cucumber-rs?** `cucumber` 0.21+ is
   current stable, with async traits. Recommendation: pin to 0.21 and
   document the migration from 0.20 in a `references/migration-0.20.md`.

## Recommended focus for /kbd-plan

**Change ordering (proposed for /kbd-plan to refine):**

1. Refactor/fork `bdd-testing` → portable `bdd-cucumber-js` skill (G-01, G-05)
2. Ship `bdd-cucumber-rs` skill (G-02, G-05)
3. Ship `bdd-lifecycle-loop` skill (G-03, G-06)
4. Extend `bdd-video-proof` with local certification bundle (G-04)
5. Cross-reference all BDD-* future-work docs (G-06)
6. Add smoke tests + validate:strict for all four skills (G-07)

Estimated 6 changes minimum. Analyze stage recommended — need to research
current cucumber-rs API surface, `fantoccini` vs `thirtyfour` state, and
Playwright video format best practices.
