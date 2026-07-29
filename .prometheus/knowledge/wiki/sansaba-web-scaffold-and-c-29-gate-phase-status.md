---
type: Reference
id: sansaba-web-scaffold-and-c-29-gate-phase-status
title: Sansaba Web Scaffold and C-29 Gate Phase Status
tags:
- sansaba-workspace
- web-scaffold
- kbd-phase
- stop-gate
- prometheus-skill-pack
- react-vite
- local-first
links:
- codex-plugin-verify-and-publish-phase-goals
sources:
- stdin
- manual:Sansaba Workspace/web-scaffold-and-c29-gate
timestamp: 2026-07-28T12:59:24.551776+00:00
created_at: 2026-07-28T12:59:24.551776+00:00
updated_at: 2026-07-28T12:59:24.551776+00:00
revision: 0
---

## Context

- **Phase:** `web-scaffold-and-c29-gate`
- **Project:** Sansaba Workspace
- **KBD root:** `/Users/gqadonis/Projects/sansaba/San Saba Automation/sansaba-workspace`
- **Captured:** `2026-07-28T12:56:06Z`
- **Position:** `web-scaffold-and-c29-gate › letter-agreement-c29-gate`
- **Status:** `apply_ready`
- **Progress:** changes `4/7`

## Phase Goals

The phase target is full implementation according to the project specifications:

- `CLAUDE.md`
- `docs/SSW-WEB-002-scaffold-plan.md`
- `docs/SSW-ARCH-001-architecture-spec-and-plan.md` Appendix C
- `docs/branding/branding-guide.md`

These documents are authoritative; this phase context records required outcomes and current execution state.

## G-1 — Install KnowMe Project Skills

Run `add-project-skills.sh` against the Sansaba Workspace repo per `SSW-WEB-002` step 2.

Required installed skills include the KnowMe doctrine plus the three locally authored skills:

- `pem-local-first`
- `sync-doctrine`
- `content-block-ui`
- `reference-ui-fidelity`
- `hybrid-design-tokens`
- `a11y-gate`

Verified script location as of `2026-07-27`:

```text
~/.claude/plugins/marketplaces/knowme-hybrid-architecture/scripts/add-project-skills.sh
```

Cache copy:

```text
~/.claude/plugins/cache/knowme-hybrid-architecture/hybrid-mobile-architecture/1.1.0/scripts/
```

## G-2 — Scaffold `web/` per SSW-WEB-002

Create a `web/` client with:

- React 19
- Vite 8
- TypeScript 7
- shadcn-ui
- Tailwind 4
- PEM `3.0.0-alpha.0` + PGlite for local-first persistence
- Zustand 5 for transient UI state only
- TanStack Router/Table
- Assistant-UI
- `bridge/{a2ui,agui}` layer

### Binding Constraints

From `SSW-WEB-002` §1 and §3:

- Do **not** run `scaffold-tauri.sh` as the app generator.
- Scaffold Vite fresh.
- Transport is **web REST + SSE to the Axum BFF**, not Tauri IPC.
- Strip Tauri coupling:
  - no `src-tauri/`
  - no `invoke()` / `listen()` in stores

## Current Session Status

The session should be parked and a new session should be started before modifying the stop-gate.

### Reasons to Start Fresh

1. **The hook is live in the current session.** Claude Code loads hook configuration at session start. Editing `position-stop-gate.sh` in this session risks continuing to run the old hook or running a half-edited hook mid-turn. The gate fires on every stop, including stops during the attempted fix.
2. **Current context is contaminated by San Saba C-29 debugging.** The active context includes flint-forge, Keto, Cedar, and type inference work. That increases the risk of scope creep while editing the shared skill-pack.
3. **Cross-repo blast radius.** `prometheus-skill-pack` governs KBD behavior across projects, not only Sansaba. Changes there should be made in a separately scoped session. Related skill-pack maintenance context exists in [Codex Plugin Verify-and-Publish Phase Goals](/codex-plugin-verify-and-publish-phase-goals.md).

### Caveat for the New Session

Opening the new session inside `prometheus-skill-pack` will still load the same hook against that cwd. Because `.kbd-orchestrator/current-waypoint.json` exists there, check its status first or drop a `PAUSE` file before editing the gate.

## Open Work and Decisions

- `letter-agreement-c29-gate` is at `17/29`:
  - §1–4 done
  - §5–7 remain
- Open architecture decision remains: what to do with **12 modified flint-forge files**, including one uncompiled `mutations.rs` edit.
- There is live DB drift to resolve or account for.
- Nothing else is intentionally in flight; the current session should remain parked until the pause mechanism is fixed.

## Recommended Next Step

Start a fresh session scoped to `prometheus-skill-pack`, fix the stop-gate/pause mechanism there, then return to this Sansaba session to decide how to handle the modified flint-forge files and resume the C-29 gate.

# Citations

1. stdin
2. manual:Sansaba Workspace/web-scaffold-and-c29-gate