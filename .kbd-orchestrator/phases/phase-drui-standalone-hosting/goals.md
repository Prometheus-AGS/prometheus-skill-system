# Goals — phase-drui-standalone-hosting

## Context

`docs/deep-research/deep-research-ui.html` is currently non-functional on
its own for two reasons:

1. Absolute script paths (`/static/htmx.min.js`, etc.) 404 under `file://`
   and require a server that maps `/static/*` to the vendored assets.
2. API calls target `http://127.0.0.1:7891` (the `prometheus-research`
   daemon), but that daemon does not serve the HTML shell or the static
   assets — only `/health`, `/api/v1/jobs*`, `/components/*`, and SSE.

The UI also does not carry the KnowMe brand system yet, even though it's
the primary visual surface for the deep-research feature.

## Goals

- [ ] **G-01: Extend `prometheus-research` to serve the UI shell** — add
  `GET /` (returns `deep-research-ui.html`) and `GET /static/*path`
  (serves files from `src/static/`) to the axum server on `:7891`. One
  process, one port, one launchd service. Users open
  `http://127.0.0.1:7891/` and the whole UI works end-to-end.

- [ ] **G-02: Relative script paths + README** — switch script tags in
  `deep-research-ui.html` from `/static/...` to `./static/...` so
  `file://` at least renders the shell. Add `docs/deep-research/README.md`
  documenting three run modes: native launchd, `cargo run`, docker
  compose.

- [ ] **G-03: Docker Compose packaging** — ship
  `docs/deep-research/docker-compose.yml` that builds the
  `prometheus-research` binary in a container and exposes `:7891`.
  Compatible with Colima and Docker Desktop. `docker compose up`
  reproduces the full experience on any machine with Docker.

- [ ] **G-04: KnowMe brand adoption** — restyle
  `deep-research-ui.html` using the KnowMe brand system from
  `/Users/gqadonis/Projects/know-me/branding/knowme-brand-guide.html`.
  Adopt the token set (`--bg`, `--fg`, `--ember`, `--ember-soft`,
  `--border`, spacing, radius) verbatim from
  `knowme-brand-template.html`. Preserve the existing OKLCH accent
  where it improves contrast; otherwise use ember. Keep dark-mode-first
  with the FOUC guard.

- [ ] **G-05: KnowMe logo + icon assets** — vendor
  `primary-light.svg` and `primary-dark.svg` from
  `/Users/gqadonis/Projects/know-me/branding/logos/` into
  `substrate/prometheus-research/src/static/brand/`. Generate PNG
  variants at **16, 32, 180, 512** with transparent background using
  `rsvg-convert` (fall back to `inkscape` or `sharp` if unavailable).
  Wire into `<head>`: theme-aware `favicon.svg`,
  `apple-touch-icon.png` (180), and a nav-visible header lockup.
  Update `manifest.json` icons array.

- [ ] **G-06: Wire into `install-binaries.sh` + CI smoke** — verify the
  extended axum server still comes up cleanly under `--mode server`.
  Extend `.github/workflows/prometheus-research.yml` (or add a step)
  that runs the binary in `--mode server`, waits for `/health`, then
  curls `/`, `/static/htmx.min.js`, and `/brand/primary-light.svg` and
  asserts non-empty 200 responses. Proves the hosted UI stays working
  after any future refactor.

## Non-goals

- **A separate standalone axum crate** — deferred; extending the
  existing daemon is simpler.
- **Nginx/Caddy sidecar in docker-compose** — the extended daemon
  already serves its own static assets; a reverse proxy would add a
  moving part with no unique benefit.
- **Production TLS / auth on the UI shell** — the daemon binds to
  loopback; if remote hosting becomes a goal, that's a future phase.
- **Recreating the KnowMe brand system** — we adopt tokens and logos
  as-is from the source repo; no restyling of the brand itself.
