# change-drui-006-ci-smoke

**Phase:** phase-drui-standalone-hosting
**Goal:** G-06 — Wire into `install-binaries.sh` + CI smoke

## Spec

Verify the extended axum server (`prometheus-research`, routes added in
`change-drui-002-serve-ui-shell`) still comes up cleanly under `--mode
server`, and add a CI smoke job to
`.github/workflows/prometheus-research.yml` that:

1. Builds the release binary.
2. Starts it in `--mode server` in the background.
3. Polls `GET /health` until it responds (with a timeout, so a hung server
   fails the job instead of hanging CI).
4. Curls `GET /`, `GET /static/htmx.min.js`, and `GET /brand/primary-light.svg`,
   asserting HTTP 200 and a non-empty response body for each.
5. Tears down the server process regardless of outcome.

This proves the hosted UI (shell + static assets + brand assets, all served
from one axum process on `:7891`) keeps working after any future refactor,
closing out G-01 through G-05 with a regression guard.

## Acceptance Criteria

- [ ] `prometheus-research --mode server` verified locally: binds `:7891`,
      `/health` returns 200, `/`, `/static/htmx.min.js`, and
      `/brand/primary-light.svg` all return 200 with non-empty bodies.
- [ ] `.github/workflows/prometheus-research.yml` has a new job that
      reproduces the above checks in CI, independent of the existing
      fmt/clippy/test matrix.
- [ ] The new job fails loudly (non-zero exit) if the server never becomes
      healthy, or if any of the three content routes 404s or returns an
      empty body.
