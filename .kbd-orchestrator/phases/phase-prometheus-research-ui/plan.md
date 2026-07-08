# Plan — phase-prometheus-research-ui

_Generated: 2026-07-08_

## Overview

5 changes, ordered by dependency and risk:

1. CI first (no deps, small, verifiable immediately)
2. SKILL.md update (no deps, prose-only)
3. Surface-bridge protocol fix (prerequisite for Tier 2 in G-02)
4. UI real-SSE replacement (largest change, depends on G-03 for Tier 2 path)
5. Smoke test (depends on binary + routes from G-02/G-03 being stable)

**Change backend:** OpenSpec (`openspec/changes/`)

---

## Ordered Change List

| # | Change ID | Goal | Scope | Risk |
|---|-----------|------|-------|------|
| 1 | `change-prui-001-skill-md-update` | G-01 | `skills/research/deep-research/SKILL.md` | Low |
| 2 | `change-prui-002-htmx-ui-real-sse` | G-02 | `docs/deep-research/deep-research-ui.html` | Medium |
| 3 | `change-prui-003-surface-bridge-wiring` | G-03 | `substrate/prometheus-research/src/agui/emit.rs` + `skills/learn/ui-surface/SKILL.md` | High |
| 4 | `change-prui-004-ci-workflow` | G-04 | `.github/workflows/prometheus-research.yml` | Low |
| 5 | `change-prui-005-smoke-test` | G-05 | `substrate/prometheus-research/scripts/smoke-test.sh` | Low |

---

## Change Details

### change-prui-001-skill-md-update (G-01)

**Goal:** Update `skills/research/deep-research/SKILL.md` to document the live
`prometheus-research` binary.

**Tasks:**
1. Add `## Background Execution (prometheus-research)` section after `## Quick Start`
2. Document `prometheus-research --mode server` startup (verify binary is on PATH)
3. Document MCP tool usage: `research_start`, `research_status`, `research_cancel`, `research_export`
4. Document SSE stream endpoint (`GET /api/v1/jobs/{id}/events`)
5. Document `render_component` MCP tool for A2UI HTMX fragments
6. Add note that launchd auto-starts `--mode mcp`; user only needs `--mode server` for HTTP UI

**Recommended agent:** general

---

### change-prui-002-htmx-ui-real-sse (G-02)

**Goal:** Replace the simulation-based `docs/deep-research/deep-research-ui.html` with a
real SSE-backed UI that connects to `prometheus-research` on `:7891`.

**Tasks:**
1. Read current `startResearch()` and simulation function signatures to understand state shape
2. Replace CDN HTMX (`unpkg.com`) with `/static/htmx.min.js` from the binary
3. Replace CDN Alpine.js with `/static/alpine.min.js` from the binary
4. Rewrite `startResearch()`: POST `/api/v1/jobs` → capture `job_id`
5. Wire `EventSource` (or HTMX `hx-ext="sse"`) to `/api/v1/jobs/{id}/events`
6. Map AG-UI event types to Alpine state updates (`agent.status` → progress ring + stage, `agent.message` → log, `agent.error` → error state, `a2ui.component` → component swap)
7. Preserve `simulateProgress()` / `simulateSongResearch()` behind a `?demo=1` query param toggle
8. Update job list / cancel flows to call DELETE `/api/v1/jobs/{id}`
9. Test: open browser, start a job, verify SSE events flow and UI updates

**Recommended agent:** general

---

### change-prui-003-surface-bridge-wiring (G-03)

**Goal:** Fix the `UiIntent` schema mismatch between `prometheus-research` and `surface-bridge`,
and update `ui-surface` SKILL.md to document Tier 2 as live.

**Tasks:**
1. Read `substrate/prometheus-research/src/agui/emit.rs` — current `{intent, payload}` shape
2. Read `substrate/surface-bridge/src/types.rs` — target `UiIntent` struct fields
3. Rewrite `emit_to_surface_bridge()` to send `{intent_type, title, body, options, multiselect, request_id}` derived from `AguiEvent` variants
4. Verify `substrate/prometheus-research` still compiles (`cargo build -p prometheus-research`)
5. Read `skills/learn/ui-surface/SKILL.md` lines 121–123 — current STUBBED notice
6. Update Tier 2 section to document the live workflow: `render_component` → `surface-bridge` iframe
7. Run `npm run validate:skill skills/learn/ui-surface` to confirm SKILL.md still validates

**Recommended agent:** general (Rust edits are surgical)

---

### change-prui-004-ci-workflow (G-04)

**Goal:** Add a GitHub Actions CI workflow for `substrate/prometheus-research`.

**Tasks:**
1. Read `.github/workflows/sovereign-sync.yml` — template structure
2. Write `.github/workflows/prometheus-research.yml` with 3-job matrix: `fmt` / `clippy` / `test`
3. Path triggers: `substrate/prometheus-research/**` and the workflow file itself
4. Cache key: hash of `substrate/prometheus-research/Cargo.lock`
5. Verify YAML syntax (`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/prometheus-research.yml'))"`)

**Recommended agent:** general

---

### change-prui-005-smoke-test (G-05)

**Goal:** Add `substrate/prometheus-research/scripts/smoke-test.sh` — a portable integration
smoke test for the running binary.

**Tasks:**
1. Create `substrate/prometheus-research/scripts/` directory
2. Write `smoke-test.sh`: start server, poll `/health`, POST job, GET status, open SSE (read 1 event), DELETE job, kill server, exit 0/1
3. `chmod +x substrate/prometheus-research/scripts/smoke-test.sh`
4. Add optional CI wire-in note to `prometheus-research.yml` (comment block; not enabled by default since binary must be pre-built)
5. Test locally: `bash substrate/prometheus-research/scripts/smoke-test.sh`

**Recommended agent:** general

---

## Execution Order Rationale

- **G-01 first** (SKILL.md): prose-only, zero risk, makes subsequent work self-documenting.
- **G-02 second** (UI real SSE): the largest change; doing it before G-03 means the Tier 1 native-EventSource path works even if the Tier 2 surface-bridge path isn't yet fixed.
- **G-03 third** (surface-bridge wiring): fixes the HIGH-risk protocol mismatch so Tier 2 in the UI works cleanly; also corrects the stale SKILL.md.
- **G-04 fourth** (CI): small, self-contained; can merge independently.
- **G-05 last** (smoke test): validates the complete stack after G-02 and G-03 are stable.

---

## First Command

```
/kbd-apply change-prui-001-skill-md-update
```
