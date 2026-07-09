# Reflection — phase-prometheus-research-ui

_Generated: 2026-07-09_

## Delta — Planned vs. Delivered

### Goal Achievement

| Goal | Status | Evidence |
|------|--------|----------|
| G-01: Update `deep-research` SKILL.md | **MET** | `skills/research/deep-research/SKILL.md` +97 lines; documents `prometheus-research --mode server`, health check, 5 MCP tools, AG-UI SSE stream, 8 A2UI endpoints. Commit `72a328a`. |
| G-02: Ship polished `deep-research-ui.html` | **MET** | `docs/deep-research/deep-research-ui.html` transformed: OKLCH dark-luxury design system, vendored HTMX/Alpine, real `POST /api/v1/jobs` + `EventSource` SSE + `DELETE` cancel, animate/delight/polish CSS, demo mode via `?demo=1`. Browser-tested, 3/3 views verified. Commit `bb183a7`. |
| G-03: Wire `render_component` into surface-bridge Tier 2 | **MET** | `skills/learn/ui-surface/SKILL.md` rewritten from "STUBBED — not yet implemented" to live 4-step flow documenting `AguiEvent→UiIntent` field mapping. `substrate/prometheus-research/src/agui/emit.rs` fixed: `agui_to_ui_intent()` now maps all 4 event variants to the correct surface-bridge schema. Commit `201d1d9`. |
| G-04: Add CI job for `prometheus-research` | **MET** | `.github/workflows/prometheus-research.yml` — fmt/clippy/test matrix, path-triggered on `substrate/prometheus-research/**`. Commit `e4ed65d`. |
| G-05: Integration smoke test | **MET** | `substrate/prometheus-research/scripts/smoke-test.sh` — 16 checks across 6 phases, exit 0 verified locally, graceful skip when binary absent. Commit `2f65aeb`. |

**Goals met: 5/5 (100%)**

_Note: `progress.json` recorded `goals_met: 4` mid-phase; all 5 were confirmed met on reflection._

---

## Delivered Changes

| Change | Description | Commit |
|--------|-------------|--------|
| change-prui-004-ci-workflow | GitHub Actions workflow for prometheus-research fmt/clippy/test | `e4ed65d` |
| change-prui-003-surface-bridge-wiring | Fixed `emit.rs` UiIntent schema mismatch; documented Tier 2 as live | `201d1d9` |
| change-prui-001-skill-md-update | Added `## Background Execution` section to deep-research SKILL.md | `72a328a` |
| change-prui-002-htmx-ui-real-sse | Real SSE + Anthropic design system for deep-research-ui.html | `bb183a7` |
| change-prui-005-smoke-test | Full HTTP API lifecycle smoke test script (16 checks, exit 0) | `2f65aeb` |

---

## Root Causes for Deltas

**No missed goals, but three execution stumbles:**

1. **emit.rs UiIntent schema mismatch (HIGH)** — `substrate/prometheus-research/src/agui/emit.rs` was sending `{intent, payload}` but surface-bridge expected `{intent_type, title, body, options, multiselect, request_id}`. The Tier 2 MCP App iframe was broken from day one of the binary phase without anyone noticing. Root cause: the struct was defined locally in `emit.rs` without cross-referencing the canonical `UiIntent` type in `surface-bridge/src/types.rs`. Discovered only during change-prui-003.

2. **ui-surface SKILL.md Tier 2 claimed "not yet shipped"** — the skill documented Tier 2 as a stub stub despite the binary having been live for two weeks. Root cause: skill documentation wasn't updated when `surface-bridge` shipped; the two repos evolved independently.

3. **`((VAR++))` under `set -e` in bash** — the smoke test initially exited non-zero on the first `((PASS++))` increment because bash evaluates `((0))` as false under `set -e`. Required a one-line fix (`PASS=$((PASS + 1))`). Root cause: false assumption that `(( ))` increment is always safe under `set -e`.

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA (artifact-refiner) | 0/5 (no refiner configured this phase) |
| Manual browser verification | 3/3 views tested (dashboard, new-research wizard, job progress view) |
| Smoke test coverage | 16/16 checks passed, exit 0 |
| CI coverage | fmt + clippy + test, path-triggered |

No artifact-refiner logs exist for this phase. Browser testing and the smoke test served as the QA gate.

---

## Technical Debt Introduced

| Item | Severity | Notes |
|------|----------|-------|
| `hls.min.js` stub | LOW | A stub is committed to `src/static/hls.min.js`. Real HLS.js is not vendored — the song media card feature won't play in production. Acceptable for now since the card is demo-only. |
| `docs/deep-research/static` symlink | LOW | Points to `substrate/prometheus-research/src/static/`. Fragile if the crate moves. Should become a proper copy step in the binary's install script. |
| SSE `?demo=1` mode relies on `URLSearchParams` in the browser | LOW | When the static `serve` tool rewrites `.html` URLs, the query string is dropped. Demo mode only works when the file is served by the binary directly (`:7891`) or with a query-preserving server. |
| `cancelJob()` is fire-and-forget on network errors | LOW | The `DELETE` catch block is `/* best-effort */`. A failed cancel doesn't surface to the user. Acceptable for a background operation but worth a follow-up. |

---

## Lessons Captured

1. **Cross-struct consistency check**: when a crate emits events consumed by another crate's server, read both struct definitions side-by-side before marking the change done. Caught late here at cost of an extra change.

2. **Skill documentation must be updated atomically with the feature**: shipping `surface-bridge` without updating `ui-surface/SKILL.md` created a false impression that Tier 2 was unimplemented. Documentation is a first-class deliverable.

3. **`set -e` + `((VAR++))` is a bash footgun**: always use `VAR=$((VAR + 1))` or `let VAR++ || true` in scripts with `set -e`. The `(( ))` form exits non-zero on zero result.

4. **Static file vendoring needs an install step**: symlinking vendored JS into the docs directory works locally but breaks portability. The correct pattern is to have the install script copy static assets alongside the HTML, or to rely on the binary as the only server.

5. **The `?demo=1` pattern is clean but fragile under static servers**: CDN/static servers that normalize URLs should preserve query strings. Consider switching to a `#demo` hash or `localStorage` flag for demo mode when binary isn't available.

---

## Recommended Next Phase

**`phase-prui-polish`** — focused on the three gaps left open:

1. **Real HLS.js** — vendor the actual `hls.min.js` and wire the song media card to a real HLS stream when a URL is found
2. **`docs/deep-research/static` install step** — add `cp -r src/static docs/deep-research/static` to `install-binaries.sh` so the symlink is replaced with a real copy on fresh installs
3. **Cancel error feedback** — surface `DELETE` failures in the UI toast rather than swallowing them silently
4. **SSE reconnect indicator** — the SSE dot goes amber on `onerror` but there's no explicit reconnect countdown; add a `Reconnecting in Ns…` overlay

Alternatively, if the focus shifts to a different domain, the remaining items above can be tracked as carry-forward tickets rather than a dedicated polish phase.
