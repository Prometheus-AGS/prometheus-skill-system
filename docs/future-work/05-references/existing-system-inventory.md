# Existing System Inventory

What was discovered to already exist during the session. Future Claude Code agents should read this before proposing new infrastructure — most of what looks like a gap is actually an existing system needing wiring or productization, not a build.

## prometheus-skill-pack (`/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/`)

### Top-level

- `README.md`, `CLAUDE.md`, `AGENTS.md`, `LICENSE`, `CHANGELOG.md`.
- `package.json`, `package-lock.json`, `node_modules/`. Limited Node tooling — most of the pack is shell + Cedar.
- `.mcp.json` — MCP server registry for installations using the pack.
- `.gitmodules` — submodules (notably `sycophancy-correction`).
- `.claude/`, `.claude-plugin/` — Claude Code plugin manifest and hooks.
- `.kbd-orchestrator/`, `.prometheus/` — orchestrator state and prometheus runtime.

### Skill catalog

- 64 skills under `skills/` covering distinct domains: agentic patterns, kdd, pmpo, openspec, evaluation, retrieval, mcp building, tooling, frontend frameworks, observability, etc.
- Each skill has at minimum `SKILL.md`. Many also have `agents/`, `prompts/`, `policies/`, sample artifacts.
- The `sycophancy-correction` skill is included as a submodule and exposes 8 detection patterns (S-01 through S-08) and 4 strictness modes.

### Hook infrastructure

- `hooks/hooks.json` and `.claude-plugin/hooks/hooks.json` — currently identical content rather than symlinked. Both register hooks for SessionStart, UserPromptSubmit, PreToolUse(Bash), PostToolUse(Write|Edit|MultiEdit), per-matcher SubagentStop, and Stop.
- `shared/scripts/` — bash scripts implementing each hook. Notable scripts include:
  - `pk-focus-on-prompt.sh` — naive keyword extraction + LLM call to fetch relevant KB entries.
  - `forge-reflect-on-stop.sh` — runs the PMPO Reflect phase.
  - `pk-lint-cron.sh` — exists but unwired.
  - `pk-focus-cleanup.sh` — cleans transient artifacts after a session.
  - `mem0-compress-on-stop.sh` — exists but not scheduled.
- All Stop-chain scripts use `|| true` for failure tolerance, which means failures are invisible.

### Cedar policies

- `policies/skill-mutation.cedar` — gates `skill.mutate` operations.
- `policies/entities.json` — entity definitions (dev/staging/prod environments).
- The Cedar PEP intercepts programmatic skill mutations but does not intercept raw `Edit`/`Write`/`MultiEdit` to a `SKILL.md` file (gap identified as SP-011).

### Slash commands

- Several `.claude/commands/*.md` files defining custom slash commands.
- The pack ships `/focus`, `/ingest`, etc. that are also shipped from `prometheus-knowledge`. No merge strategy currently (gap identified as SP-017).

### Already in flight / planned

- `.kbd-orchestrator/current-waypoint.md` indicates an active KBD project state.
- `docs/plans/2026-04-29-change-006-karpathy-loop-hooks.md` describes a planned hook layer for the Karpathy loop. Some of this is implemented; some is described but not visible at the hook layer (SP-007 verification needed).

## prometheus-knowledge (`/Users/gqadonis/Projects/prometheus/prometheus-knowledge/`)

### Workspace structure

Eight Rust crates in a Cargo workspace:

- `pk-core` — fundamental types (`WikiEntry`, `LibrarianEvent`, etc.).
- `pk-store` — markdown-backed storage with HNSW vector search.
- `pk-mcp` — MCP server exposing tools to Claude.
- `pk-librarian` — the curator process that ingests events and synthesizes wiki entries. Uses model routing (Sonnet for compile, local Qwen for lint/focus/fix).
- `pk-cli` — command-line entry point.
- Plus three smaller crates for utilities, prompts, and integration.

### Slash commands

- `.claude/commands/focus.md` and `.claude/commands/ingest.md` — the user-facing surfaces for librarian operations.

### Karpathy KB location

- Default: `~/.prometheus/knowledge/`. **Currently global across all projects on the machine** — the source of the confidentiality risk identified as SP-008.
- Configurable via `PK_KB_DIR` environment variable, but defaults are not per-project.

### Persistence model

- WikiEntry storage is markdown files on disk with vector embeddings for retrieval.
- Events (`LibrarianEvent`) are *currently in-memory* in the librarian process and not persisted as first-class records (gap identified as SP-019).

## ssr-frontend (`/Users/gqadonis/Projects/sansaba/ssr-frontend/`)

### Stack baseline

- Next.js 16 + React 19 + TypeScript strict mode + Tailwind 4 + Radix UI + Zustand.
- Prisma ORM with `mssql` adapter.
- Supabase for the feedback engine backend.
- Ory Kratos for auth.
- Cucumber.js + Playwright for BDD.

### Test infrastructure

- `cucumber.js` profile config — multiple profiles (default, api, ui, ui-real-data, reports, pdf, agents, video, video-single).
- `tests/features/ui/` — 38 .feature files. `tests/features/api/` — 3 .feature files.
- `tests/steps/` — 22 .steps.ts files.
- `tests/support/` — `world.ts`, `hooks.ts` (auth setup, cleanup, video lifecycle).
- `tests/.auth/state.json` — persisted auth state for fast re-runs.
- `tests/reports/` — html/json reports, videos, traces.

### Video pipeline scripts (`scripts/`)

- `run-video-proof.ts` — resumable per-scenario runner. Reads `video-proof-state.json`, only re-runs scenarios with `status != 'passed'` or `videoSizeBytes == 0`. Spawns `cucumber-js` per scenario with `--profile video-single`. Has a 240s default timeout per scenario, 15s heartbeat, configurable via env.
- `validate-video-coverage.ts` — three-phase validation (`videos`, `uploaded`, `published`). Each phase is strictly more demanding.
- `upload-videos-to-ipfs.ts` — uploads `.webm` files to IPFS, captures CIDs, writes/updates `docs/videos-manifest.json`.
- `generate-video-run-report.ts` — produces a Markdown report of the latest run with per-scenario status, duration, video link, and CID. Fails the gate if pass-rate or coverage isn't 100%.
- `generate-bdd-docs.ts` — builds the docs site at `docs/site/` from feature files. Generates per-area, per-feature, per-tag, and HTMX scenario partial pages. Mirrors output to `public/docs/`.
- `record-and-publish-videos.sh` — orchestrator that runs the full pipeline (wait for dev server, dry-run check, run-video-proof, upload, regenerate docs, validate, generate report).

### Manifest

- `docs/videos-manifest.json` — currently has ~250 entries with **dual keying**: 32-char hex (legacy UUID) and slug-form scenario IDs. Some scenarios appear in both forms. Cleanup is BDD-001.

### Docs site

- `docs/site/index.html` — area card grid with stat bar.
- `docs/site/area/<slug>.html` — per-area listing.
- `docs/site/feature/<slug>.html` — per-feature scenario list with collapsible HTMX detail partials.
- `docs/site/tags/<tag>.html` — per-tag aggregated scenario list.
- `docs/site/partials/scenario-<id>.html` — HTMX-loaded scenario detail with steps and tags.
- `docs/site/_meta.json` — generated metadata.

### Feedback engine

- `packages/feedback-core/` — shared types, schemas, evidence helpers, triage helpers.
- `packages/feedback-ui/` — placeholder package for reusable UI exports.
- `packages/feedback-agent/` — placeholder for future Mastra workflow.
- `src/components/feedback/feedback-runtime.tsx` — the runtime that injects pageContext + threadId, handles assistant tool calls, captures screenshots in `dom` or `display` mode, opens annotation overlay.
- `src/components/feedback/screenshot-capture.ts` — html2canvas (DOM mode) and getDisplayMedia (display mode).
- `src/components/feedback/annotation-canvas.tsx` — overlay supporting arrows, circles, text notes.
- `src/components/feedback/feedback-sidebar.tsx` — UI shell.
- `src/stores/feedback-telemetry-store.ts` and `src/hooks/use-feedback-telemetry.ts` — opt-in telemetry capture.
- `src/lib/feedback-project-adapter.ts` — project-specific knowledge-base sources, redaction, issue destination.
- `src/app/api/feedback/route.ts` — entry point on the server.
- `supabase/migrations/20260326000000_feedback_platform.sql` — schema for `feedback_threads`, `feedback_records`, `feedback_evidence`, `feedback_knowledge_base_chunks`, `feedback_triage_runs`.
- `supabase/functions/analyze-feedback/index.ts` — Edge Function for automated triage.

### Azure AI Foundry agent

- `docs/feedback-assistant.system.md` — source of truth for the `sansaba` agent system prompt.
- The agent has file_search bound to a vector store with all use-case docs.
- It defines three function tools: `capture_screenshot`, `annotate_screenshot`, `submit_feedback`.
- The system prompt itself was authored with sycophancy-correction review applied (run at `standard` strictness).

### KBD orchestrator and OpenSpec

- `.kbd-orchestrator/` — phase tracking with assess/plan/execute/reflect contract.
- `openspec/specs/` and `openspec/changes/` — change-driven development records.
- `.refiner/` — artifact registry.

### CLAUDE.md rules in force

- Mandatory KBD lifecycle protocol with disk-artifact requirements before phase transitions.
- Karpathy guidelines (think before coding, simplicity first, surgical changes, goal-driven execution).
- Flat 2.0 design system rules (no borders, background color differentiation).
- TanStack Query as single source of truth for server state.
- Component → Hook → Store communication contract.
- TypeScript no-`any` rule.
- 500-line per-file limit (refactor into directory if exceeded).
- Multiple "out of scope" features explicitly documented (1099 form upload, audit history, user management UI lives in a separate IAM project).

## document-generation-agent (`/Users/gqadonis/Projects/sansaba/document-generation-agent/`)

Not directly read in this session, but referenced extensively. SSR's CLAUDE.md states: "All document generation lives in the separate `document-generation-agent` project. This project owns every template (markdown/handlebars format) and generates documents via REST API endpoints. The SSR frontend calls those endpoints via `DOCUMENT_GENERATION_AGENT_URL` — it does NOT contain templates or generation logic."

Templates per the customer-approved Template Bible (`docs/template-meeting/Template-Review-dcfd80e0-f62f.md` in SSR, mirrored to `docs/generation/template-bible.md`):

- Letter Agreement + Conveyances + Exhibit A family (with new producing-Texas and producing-Louisiana variants).
- File checklists (producing/non-producing).
- County: letter-to-request-probate, request-for-certified-copies.
- Customer: quit-claim-deed, title-failure-sold-unrecorded-conveyance.
- Reports: invoices-due, purchases, letter-agreement-deals, drafts-due (internal/meeting/external).
- Per-buyer flows: invoice/buyer-invoice, notification/buyer-notification-with-check.

## What this means for new work

If you find yourself proposing to "build a video proof system," you are duplicating BDD-001's parent. Read it first.

If you find yourself proposing to "build a feedback engine," it exists. The work in this pack extends it (BDD-014, BDD-015), not rebuilds it.

If you find yourself proposing to "build a knowledge base," it exists in two forms: `prometheus-knowledge` (Karpathy-pattern wiki) and `feedback_knowledge_base_chunks` (Supabase). Pick the right one or extend both, but don't add a third.

If you find yourself proposing to "build a Cedar policy engine," it exists. Wire your enforcement into the existing `policies/` rather than starting fresh.
