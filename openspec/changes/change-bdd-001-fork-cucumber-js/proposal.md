# Proposal — change-bdd-001-fork-cucumber-js

Fork `skills/testing/bdd-testing/` into a new portable
`skills/testing/bdd-cucumber-js/` skill. Bump dependencies to
`@cucumber/cucumber` 13.0.0, `playwright-bdd` 9.2.0, and `tsx`; remove
Next.js SSR wording so the skill is project-agnostic. Leave the original
`bdd-testing` skill in place as a thin compatibility redirect so downstream
projects (ssr-frontend) don't break.

## Library candidates

- **cand-001**: `@cucumber/cucumber` 13.0.0 (npm, weekly 2.2M) — adopt
- **cand-002**: `playwright-bdd` 9.2.0 by vitalets (npm, weekly 410k) — adopt
- **cand-003**: `tsx` latest (npm) — adopt (recommended by cucumber-js docs)

## Goal
G-01 — Cucumber-js authoring skill.
