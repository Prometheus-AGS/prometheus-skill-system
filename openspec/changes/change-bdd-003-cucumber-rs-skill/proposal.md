# Proposal — change-bdd-003-cucumber-rs-skill

Ship a new `skills/testing/bdd-cucumber-rs/` skill covering the `cucumber`
0.23 crate + `thirtyfour` 0.37 browser driver. Primary path documented for
async World with tokio + reqwest for HTTP scenarios and thirtyfour for
browser-driven scenarios. Include `references/migration-from-0.20.md`
documenting the `#[async_trait]` removal in 0.21 and MSRV bumps to 1.88.
Document `fantoccini` and `headless_chrome` as alternatives.

## Library candidates

- **cand-004**: `cucumber` 0.23.0 (crates.io, ~230k/month) — adopt
- **cand-005**: `thirtyfour` 0.37.2 (crates.io, latest 2026-07-05) — adopt
- **cand-006**: `fantoccini` 0.22.1 — document as alternative
- **cand-007**: `headless_chrome` 1.0.22 — document as CDP alternative

## Goal
G-02 — Cucumber-rs authoring skill.
