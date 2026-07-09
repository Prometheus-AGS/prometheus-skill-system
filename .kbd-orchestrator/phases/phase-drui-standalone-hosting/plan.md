# Plan — phase-drui-standalone-hosting

_Generated: 2026-07-09_

## Change Backend
Native KBD (no OpenSpec proposals for a 6-change infrastructure phase — the
surface is small and self-contained; using inline task tracking).

## Overview

6 changes, ordered so the KnowMe brand assets land **first** (they're the
input to G-04 and G-05 restyling), then the axum static-serving change so
the assets have a way to reach the browser, then relative paths + README,
then docker-compose, then the restyle itself, then CI.

## Changes

| # | Change ID | Goals | Description |
|---|-----------|-------|-------------|
| 1 | `change-drui-001-brand-tokens-and-assets` | G-05 | Copy `primary-{light,dark}.svg` from `~/Projects/know-me/branding/logos/` to `substrate/prometheus-research/src/static/brand/`. Generate PNGs at 16/32/180/512 with `rsvg-convert` (fallback: `inkscape`, then `sharp`). Vendor KnowMe token set as a standalone `brand/tokens.css` importable from the UI. |
| 2 | `change-drui-002-serve-ui-shell` | G-01 | Extend axum server in `substrate/prometheus-research/src/main.rs` (or dedicated `server.rs`) with `GET /` → `deep-research-ui.html` and `GET /static/*` + `GET /brand/*` → `ServeDir` over `src/static/`. Include-str! the HTML so no fs read at request time. |
| 3 | `change-drui-003-relative-paths-and-readme` | G-02 | Change script tags: `/static/...` → `./static/...`. Add `docs/deep-research/README.md` with three run modes (native launchd, `cargo run`, docker-compose) and troubleshooting. |
| 4 | `change-drui-004-docker-compose` | G-03 | Write `docs/deep-research/docker-compose.yml` + `docs/deep-research/Dockerfile` (multi-stage: `rust:1.88` build → `debian:bookworm-slim` runtime). Compose exposes `:7891`. |
| 5 | `change-drui-005-restyle-ui-knowme` | G-04 | Import `./brand/tokens.css` at top of the style block. Replace existing CSS vars with KnowMe tokens. Add `<link rel="icon">` (SVG + PNG fallback), `<link rel="apple-touch-icon">`. Add KnowMe logo lockup to the header. Keep the SSE/HTMX behavior, only shift colors and lockup. |
| 6 | `change-drui-006-ci-smoke` | G-06 | Extend `.github/workflows/prometheus-research.yml` with a smoke step: start binary in `--mode server`, poll `/health`, then curl `/`, `/static/htmx.min.js`, `/brand/primary-light.svg` and assert 200 + non-empty body. |

## Execution Order Rationale

- **001 first** because 005 needs the brand tokens + logos to reference.
  Assets under `substrate/prometheus-research/src/static/brand/` also need
  to exist before the axum static handler in 002 has meaningful content
  to serve.
- **002 before 003/005** because relative paths and the restyle both
  need a working server to be verifiable.
- **004 (docker-compose) after 001-003** so the container build ships
  the correct assets and default paths.
- **005 (restyle) after 002/003** so the UI is server-hosted with
  relative paths before styles change — avoids conflating restyle-broke-it
  vs infra-broke-it.
- **006 (CI) last** so the smoke checks pin the final state.

## Apply Commands

```
/kbd-apply change-drui-001-brand-tokens-and-assets
/kbd-apply change-drui-002-serve-ui-shell
/kbd-apply change-drui-003-relative-paths-and-readme
/kbd-apply change-drui-004-docker-compose
/kbd-apply change-drui-005-restyle-ui-knowme
/kbd-apply change-drui-006-ci-smoke
```

## First Change

Start with `change-drui-001-brand-tokens-and-assets` — vendors the
assets that later changes depend on.
