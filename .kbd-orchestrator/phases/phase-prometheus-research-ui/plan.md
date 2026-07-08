# Plan — phase-prometheus-research-ui (Revised)

_Generated: 2026-07-08 | Revised: 2026-07-08 with UI/UX standards_

## Design Standards Binding This Phase

This plan integrates the following skill-collection and design standards
for every UI-touching change:

### Visual Language (Anthropic + htmx-alpine-lit + delight + animate)

| Token | Value | Source |
|-------|-------|--------|
| Background (dark) | `oklch(0.14 0.01 70)` — warm dark brown, not `#000` | Anthropic Claude aesthetic |
| Surface elevated | `oklch(0.18 0.01 70)` — one step lighter | Claude UI language |
| Foreground | `oklch(0.95 0.02 70)` — warm cream, not `#fff` | Claude |
| Accent | `oklch(0.70 0.14 45)` — terracotta/copper | Claude brand accent |
| Success | `oklch(0.72 0.16 145)` — sage green | Semantic only |
| Error | `oklch(0.65 0.20 30)` — warm red | Semantic only |
| Border radius | ≤ 6px in dense UI; 8px for cards | Anthropic rule |
| Shadows | None (contrast-based depth, not drop shadows) | Anthropic rule |
| Typography | `JetBrains Mono` or `Space Grotesk`; NO Inter/system-ui | Claude Code / Anthropic |
| Max colors | 2–3 hues, semantic use only | Anthropic |

### Motion Rules (animate skill)

- All state changes animated: 200–300ms with `cubic-bezier(0.16, 1, 0.3, 1)` (ease-out-expo)
- Entrance: fade + translate-y (-4px → 0), stagger 100ms per element
- Progress ring uses `stroke-dashoffset` transitions only (compositor-friendly)
- Stage timeline items slide in from left as they activate
- `@media (prefers-reduced-motion)` disables all motion
- No bounce, no elastic, no CSS defaults

### AG-UI / Vercel Streaming Design (htmx-alpine-lit + AG-UI protocol)

- SSE stream uses **start/delta/end** event triplet per chunk — never replace whole buffer
- Progress is driven by **named step events** (`STEP_STARTED`/`STEP_FINISHED`), not generic spinners
- Loading states: skeleton placeholders → live content; never raw spinners mid-stream
- State sync: Alpine stores the full job snapshot on first `agent.status`, then patches on deltas
- No polling — everything is EventSource push
- `a2ui.component` events swap HTMX fragments into designated target divs (no page reload)

### Delight Touchpoints (delight skill)

- Research start: pulse animation on the progress ring border for 400ms
- Stage completion: brief scale 1 → 1.02 → 1 on stage badge (100ms)
- Job complete: animated checkmark draw on the progress ring center icon
- Error state: warm-red tint fade on error card, shake (translateX) on the error badge
- Empty state (no jobs): subtle floating animation on the "start research" illustration
- SSE connected: status dot fades from amber → green with a 300ms glow pulse

### Polish Requirements (polish skill)

- All interactive elements need all 5 states: default, hover, focus, active, disabled
- Focus rings: 2px offset, `oklch(0.70 0.14 45)` (accent), never hidden
- Typography hierarchy: h1 32px 700, h2 20px 600, body 14px 400, mono 13px for job IDs/hashes
- Consistent spacing: 4px base grid, multiples only (8, 12, 16, 24, 32, 48)
- Component tokens defined in `:root` via CSS custom properties — no hardcoded hex anywhere
- Both the HTMX standalone page (`deep-research-ui.html`) and any surface-bridge Tier 2
  iframe must match this design language exactly

---

## Overview

5 changes, ordered by dependency and design cohesion:

1. CI workflow (no deps, small, enables confidence for subsequent Rust changes)
2. Surface-bridge protocol fix (Rust, prerequisite for Tier 2 Tier 2 UI correctness)
3. SKILL.md update (prose — documents the live system after protocol is fixed)
4. UI real-SSE replacement (largest change, depends on confirmed protocol from G-03)
5. Smoke test (validates the full stack end-to-end)

**Change backend:** OpenSpec (`openspec/changes/`)

---

## Ordered Change List

| # | Change ID | Goal | Scope | Risk |
|---|-----------|------|-------|------|
| 1 | `change-prui-004-ci-workflow` | G-04 | `.github/workflows/prometheus-research.yml` | Low |
| 2 | `change-prui-003-surface-bridge-wiring` | G-03 | `src/agui/emit.rs` + `skills/learn/ui-surface/SKILL.md` | **High** |
| 3 | `change-prui-001-skill-md-update` | G-01 | `skills/research/deep-research/SKILL.md` | Low |
| 4 | `change-prui-002-htmx-ui-real-sse` | G-02 | `docs/deep-research/deep-research-ui.html` | Medium |
| 5 | `change-prui-005-smoke-test` | G-05 | `substrate/prometheus-research/scripts/smoke-test.sh` | Low |

---

## Change Details

### change-prui-001-skill-md-update (G-01) — Execute 3rd

**Goal:** Update `skills/research/deep-research/SKILL.md` to document the live
`prometheus-research` binary (written *after* G-03 so docs describe the corrected protocol).

**Tasks:**
1. Add `## Background Execution (prometheus-research)` section after `## Quick Start`
2. Document `prometheus-research --mode server` startup (verify binary is on PATH via `which prometheus-research`)
3. Document MCP tool usage: `research_start`, `research_status`, `research_cancel`, `research_export`
4. Document SSE stream endpoint: `GET /api/v1/jobs/{id}/events` (`text/event-stream`, AG-UI event frames)
5. Document `render_component` MCP tool: returns an HTMX HTML fragment for one of the 8 A2UI components
6. Note launchd auto-starts `--mode mcp`; user only needs `--mode server` for the browser UI
7. Run `npm run validate:skill skills/research/deep-research`

**Recommended agent:** general

---

### change-prui-002-htmx-ui-real-sse (G-02) — Execute 4th (largest)

**Goal:** Transform `docs/deep-research/deep-research-ui.html` from a 4339-line simulation
into a production-quality AG-UI-connected research visualization UI.

**Design spec (MANDATORY — apply htmx-alpine-lit + animate + delight + polish skills):**

```
Visual direction:  Dark luxury / technical editorial
                   Anthropic warm-dark palette (see tokens above)
                   JetBrains Mono for IDs/code, Space Grotesk for prose
Style direction:   NOT glassmorphism; NOT generic shadcn card grid
                   Editorial hierarchy: one dominant progress ring, clear stage timeline
Motion:            ease-out-expo everywhere; no bounce; prefers-reduced-motion gate
Delight moments:   Start pulse, stage completion flash, job-done checkmark draw, SSE-connected dot
```

**CSS architecture:**
```css
:root {
  --bg: oklch(0.14 0.01 70);
  --surface: oklch(0.18 0.01 70);
  --fg: oklch(0.95 0.02 70);
  --accent: oklch(0.70 0.14 45);
  --success: oklch(0.72 0.16 145);
  --error: oklch(0.65 0.20 30);
  --border: oklch(0.28 0.01 70);
  --radius: 6px;
  --ease-expo: cubic-bezier(0.16, 1, 0.3, 1);
  --duration-fast: 150ms;
  --duration-normal: 300ms;
  font-family: 'Space Grotesk', 'JetBrains Mono', system-ui;
}
```

**Tasks:**
1. Read `startResearch()`, `simulateProgress()`, `simulateSongResearch()` — map Alpine state shape
2. Move CSS to `:root` tokens; replace all hardcoded colors with custom properties
3. Replace CDN HTMX/Alpine `<script>` with `/static/htmx.min.js` + `/static/alpine.min.js`
4. Rewrite `startResearch()`: `POST /api/v1/jobs` → capture `job_id`; wire `EventSource` to `/api/v1/jobs/{job_id}/events`
5. Map AG-UI events → Alpine state: `agent.status` → progress ring + stage timeline; `agent.message` → log list; `agent.error` → error card; `a2ui.component` → HTMX `hx-swap-oob` into component target divs
6. Apply `animate` skill motion spec: progress ring `stroke-dashoffset` transition, stage items slide in, entrance choreography on page load
7. Apply `delight` skill moments: start pulse, stage-done flash, job-done checkmark draw, SSE-connect dot color change
8. Apply `polish` skill: all 5 interaction states per button/input; focus rings; spacing grid; typography scale
9. Gate original simulation behind `?demo=1` query param (preserve for offline showcase)
10. Update cancel button → `DELETE /api/v1/jobs/{job_id}`
11. Test: start binary, open UI in browser, start job, verify SSE events drive live UI updates across all panels
12. Commit: `feat(deep-research-ui): real SSE AG-UI integration with Anthropic design system`

**Recommended agent:** general (with ui-ux-designer agent for design review pass)

---

### change-prui-003-surface-bridge-wiring (G-03) — Execute 2nd

**Goal:** Fix `UiIntent` schema mismatch between `prometheus-research` and `surface-bridge`;
update `ui-surface` SKILL.md to document Tier 2 as live.

**Root cause (from assessment):** `emit.rs` sends `{intent, payload}` — `surface-bridge`
deserializes `{intent_type, title, body, options, multiselect, request_id}`. Deserialization
fails silently; Tier 2 is completely broken until this is fixed.

**Surface-bridge UiIntent mapping from AguiEvent:**

| AguiEvent type | `intent_type` | `title` | `body` |
|----------------|---------------|---------|--------|
| `agent.status` | `"progress"` | Stage name (e.g., `"retrieve"`) | JSON of `{stage, progress, status}` |
| `agent.message` | `"feedback"` | `"Agent message"` | `event.message` |
| `agent.error` | `"feedback"` | `"Research error"` | `event.message` |
| `a2ui.component` | `"prompt"` | Component name | JSON of `event.props` |

**Tasks:**
1. Read `substrate/prometheus-research/src/agui/emit.rs` — current struct + function
2. Read `substrate/surface-bridge/src/types.rs` — target `UiIntent` struct
3. Rewrite `emit_to_surface_bridge()`: map `AguiEvent` → surface-bridge `UiIntent` per table above
4. Set `request_id = job_id`, `options = None`, `multiselect = false`
5. `cargo build -p prometheus-research` — must compile clean
6. Read `skills/learn/ui-surface/SKILL.md` — update Tier 2 STUBBED block to live docs
7. Tier 2 live docs should describe: `render_component` call → binary returns HTML fragment → POST to surface-bridge
8. Run `npm run validate:skill skills/learn/ui-surface`
9. Commit: `fix(prometheus-research): align UiIntent shape with surface-bridge; Tier 2 live`

**Recommended agent:** general (Rust edit is surgical, ~20 lines)

---

### change-prui-004-ci-workflow (G-04) — Execute 1st

**Goal:** Add `.github/workflows/prometheus-research.yml`. Template from `sovereign-sync.yml`.

**Tasks:**
1. Read `.github/workflows/sovereign-sync.yml` — copy matrix structure, toolchain action, cache
2. Write `.github/workflows/prometheus-research.yml`:
   - Path triggers: `substrate/prometheus-research/**` + workflow file itself
   - 3-job matrix: `fmt` (`cargo fmt --check`), `clippy` (`-D warnings`), `test` (`cargo test`)
   - `dtolnay/rust-toolchain@stable`, `actions/cache@v4`
   - `--manifest-path substrate/prometheus-research/Cargo.toml` on all cargo commands
   - Cache key: `${{ hashFiles('substrate/prometheus-research/Cargo.lock') }}`
3. Validate YAML: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/prometheus-research.yml'))"`
4. Commit: `ci: add GitHub Actions workflow for prometheus-research`

**Recommended agent:** general

---

### change-prui-005-smoke-test (G-05) — Execute 5th

**Goal:** `substrate/prometheus-research/scripts/smoke-test.sh` — portable integration test.

**Tasks:**
1. Create `substrate/prometheus-research/scripts/` directory
2. Write `smoke-test.sh`:
   - Start `prometheus-research --mode server` in background (fallback: `cargo run --manifest-path … -- --mode server`)
   - Poll `GET /health` with retries (max 10 × 500ms)
   - POST `/api/v1/jobs` `{"query":"smoke test"}` → assert `job_id` present
   - GET `/api/v1/jobs/{id}` → assert `status` present
   - Open SSE via `curl --no-buffer` → read first `data:` line within 10s
   - DELETE `/api/v1/jobs/{id}` → assert 200
   - Kill server; verify gone
   - Exit 0 on pass, 1 on failure; print green/red pass/fail line with `✓`/`✗` per step
3. `chmod +x substrate/prometheus-research/scripts/smoke-test.sh`
4. Local test: `bash substrate/prometheus-research/scripts/smoke-test.sh`
5. Commit: `test(prometheus-research): add full-lifecycle smoke-test.sh`

**Recommended agent:** general

---

## Execution Order

```
1. change-prui-004-ci-workflow      (enables Rust CI for subsequent Rust change)
2. change-prui-003-surface-bridge-wiring  (fixes HIGH risk Rust issue; docs after)
3. change-prui-001-skill-md-update  (docs reflect live corrected system)
4. change-prui-002-htmx-ui-real-sse (largest; design system applied here)
5. change-prui-005-smoke-test       (validates complete stack)
```

## First Command

```
/kbd-apply change-prui-004-ci-workflow
```
