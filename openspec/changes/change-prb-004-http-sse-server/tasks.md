# Tasks — change-prb-004-http-sse-server

- [x] Implement `src/agui/emit.rs` — `emit_to_surface_bridge()` POSTs UiIntent to surface-bridge (non-fatal)
- [x] Create `src/http_server/health.rs` — `GET /health` → `{status, version, pid, service}`
- [x] Create `src/http_server/sse.rs` — `GET /api/v1/jobs/{id}/events` SSE stream from broadcast channel
- [x] Create `src/http_server/rest.rs` — POST/GET/DELETE `/api/v1/jobs`, `/components/{name}`, `/static/{*path}`
- [x] Implement `src/http_server/mod.rs` — Axum 0.8 router (`{id}` syntax), CorsLayer, `run_server()`
- [x] Create `src/static/` placeholder files for HTMX/Alpine (vendored in change-005)
- [x] Run `cargo build --release` — 0 errors
- [x] Smoke test: `/health` → `{"status":"ok"}`, `POST /api/v1/jobs` → job_id, `/events` → `text/event-stream`
