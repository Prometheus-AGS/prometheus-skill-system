# Goals — phase-bdd-video-proof

## Context

The prometheus-skill-pack currently ships two BDD-adjacent skills (BDD-005
immutable tests rule, BDD-006 features/steps organization, plus scattered
Cucumber usage in downstream projects like `ssr-frontend`), but the skills
themselves do not close the full BDD loop: authoring, maintenance, live
visual + non-visual execution, and certification of full-function delivery.

We need to refine the BDD-testing skill family so that agents can:
- Author features and step definitions in **both** cucumber-js (TypeScript
  ecosystem) and cucumber-rs (Rust ecosystem) with idiomatic patterns
- Drive **live visual integration testing** (real browser via Playwright
  + video capture, screenshot diffs) and **non-visual integration**
  (headless HTTP/CLI/library-level assertions) from the same feature files
- Certify a module as "fully functional" via a video-proof artifact that is
  reproducible, machine-verifiable, and reviewable
- Maintain the loops long-term (quarantine flakes, refresh baselines,
  update steps when APIs drift — without violating the immutable-tests rule)

## Goals

- [ ] **G-01: Cucumber-js authoring skill** — a first-class skill (e.g.,
  `skills/testing/bdd-cucumber-js/`) that walks agents through installing
  cucumber-js, setting up `features/` + `steps/`, configuring
  `cucumber.js`/`cucumber.yaml`, choosing between CommonJS/ESM/TypeScript
  runners, and wiring reporters (JSON + HTML + JUnit). Includes idiomatic
  patterns for Playwright-driven visual scenarios and pure-HTTP scenarios.

- [ ] **G-02: Cucumber-rs authoring skill** — a first-class skill (e.g.,
  `skills/testing/bdd-cucumber-rs/`) covering `cucumber` crate 0.21+, async
  World types, feature-file conventions, and step-macro patterns. Documents
  how to drive tokio + reqwest for HTTP integration, and how to drive
  headless browser sessions (via `fantoccini` or `thirtyfour`) for
  visual scenarios.

- [ ] **G-03: BDD lifecycle loop skill** — a skill (e.g.,
  `skills/testing/bdd-lifecycle-loop/`) that codifies the create → run →
  triage → maintain loop as a repeatable KBD-adjacent workflow. Covers:
  writing failing scenarios first, iterating on step definitions,
  quarantining flakes (integrating with the existing e2e-runner agent's
  quarantine concept), refreshing visual baselines, and enforcing the
  immutable-tests rule during code generation.

- [ ] **G-04: Video-proof certification skill** — a skill (e.g.,
  `skills/testing/bdd-video-proof/`) that produces a signed,
  machine-verifiable "certification bundle" for a module: cucumber JSON
  report + Playwright video(s) + screenshot manifest + git SHA + module
  fingerprint. Bundle is stored under `docs/certifications/<module>/<sha>/`
  and referenced from the module's README so reviewers can watch the proof.

- [ ] **G-05: Visual + non-visual scenario examples** — reference feature
  files under `skills/testing/*/references/examples/` that show the same
  business behavior tested (a) via HTTP-only steps and (b) via
  Playwright-driven browser steps, so agents can see when to choose which
  scenario style. At least one example each for cucumber-js and cucumber-rs.

- [ ] **G-06: Integrate with existing BDD skills** — reconcile the new
  skills with `docs/future-work/02-bdd-testing-evolution/` (BDD-005 through
  BDD-007). Update CLAUDE.md prose in the target project pattern so the
  immutable-tests rule references the new lifecycle-loop skill instead of
  restating the rationale inline.

- [ ] **G-07: Cross-platform install + validation** — new skills validate
  against `npm run validate:strict` (agentskills.io spec), install cleanly
  via `scripts/install-skills-flat.sh` to Claude Code, Kimi, MiniMax,
  OpenCode, Codex, Cursor. Each skill has an executable smoke script under
  `scripts/` that verifies a minimal cucumber run passes end-to-end.

## Non-goals

- Building a *replacement* for cucumber-js or cucumber-rs — the skills
  document idiomatic usage of the upstream tools; they do not fork them.
- Wiring BDD into every existing project — the skills are additive; downstream
  projects adopt them individually.
- Full-stack visual regression frameworks (Percy, Chromatic, Applitools) —
  video-proof uses Playwright's native video/screenshot capture only,
  keeping the certification bundle self-contained and vendor-independent.
