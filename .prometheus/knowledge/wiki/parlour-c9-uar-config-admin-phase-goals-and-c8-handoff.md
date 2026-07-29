---
type: Reference
id: parlour-c9-uar-config-admin-phase-goals-and-c8-handoff
title: Parlour C9 UAR Config Admin Phase Goals and C8 Handoff
tags:
- parlour
- uar
- config-admin
- llm-config
- surrealdb
- electricsql
- pglite
- phase-goals
sources:
- stdin
- manual:parlour/c9-uar-config-admin
timestamp: 2026-07-16T19:28:28.127953+00:00
created_at: 2026-07-16T19:28:28.127953+00:00
updated_at: 2026-07-16T19:28:28.127953+00:00
revision: 0
---

## Context

- **Phase:** `c9-uar-config-admin`
- **Project:** `parlour`
- **KBD root:** `/Users/gqadonis/Projects/parlour/parlor-gather-space`
- **Captured:** `2026-07-16T19:26:25Z`
- **Source context:** `manual:parlour/c9-uar-config-admin`
- **Admin UI target:** `https://uar.parlour-world.io`

## Phase Goals

Make UAR LLM configuration and agents DB-backed, visible in the admin UI, editable from the UI, and synchronized across persistence backends with realtime/local-first behavior.

### Required Outcomes

- Seed env/YAML-configured LLM providers, models, and settings into the DB on first boot.
  - Seeding must be idempotent.
  - Seeding must be drift-safe.
  - After seeding, UAR should read and mutate config from the DB rather than treating env/YAML as the live source of truth.
- Expose providers, models, and settings in the UAR admin UI.
- Support adding and editing providers, models, and settings from the UI.
  - **Models CRUD** is the new required capability.
- Support adding and editing agents from the UI, persisted to the config DB.
  - Agent UI persistence largely exists but must be verified live.
- Preserve backend-agnostic client behavior across persistence backends.

## Realtime and Local-First Requirements

- **SurrealDB path:** use existing SurrealDB live queries.
- **Postgres path:** evaluate ElectricSQL as an optional realtime layer.
  - ElectricSQL is not mandatory if a lighter `LISTEN/NOTIFY` → `/api/live` bridge is sufficient.
  - The Postgres realtime decision is deferred to planning.
- **Client local cache:** use PGlite via `@prometheus-ags/prometheus-entity-management` v2.0.2 adapters.
  - Relevant adapters already shipped:
    - `surreal-live`
    - `electricsql`
    - `pglite-persistence`

## Non-Goals / Deferred Work

- Do not change the agent compile/register pipeline; that was completed in `c8b`.
- Do not force ElectricSQL adoption before determining whether a simpler bridge is enough.

## Success Criteria

1. The env-configured Qwen provider appears in the admin Providers UI and is editable.
2. Adding a provider, model, or agent from the UI persists across a UAR restart.
3. An edit pushes to the UI in realtime:
   - SurrealDB path: expected now via live queries.
   - Postgres path: pending realtime architecture decision.

## Handoff From C8

Previous position:

```text
c8-live-agentic-lobby › agentic-6-automation-chip-verify
status: execution_complete
progress: 6/6 changes complete
```

C8 completed `agentic-6 — disclosed-automation chip + phase gate`.

### Implemented in C8

- `/rooms/lobby` now renders a **Running quietly** panel disclosing Host automation:
  - Daily welcome cron.
  - Queued Moderator review.
- Component layering follows the C8 guard:
  - `LobbyAutomationChip` → `useLobbyAutomations` hook → `MockBossFangClient`.
  - The component does not call the client directly.
- `describeCadence` was extracted into a pure module so display logic can be unit-tested without jsdom/RTL.
- UI styling follows Flat 2.0:
  - Teal Agent badge.
  - Gold task status.
  - `bg3` sub-surfaces.
  - No borders.
  - Provenance text: `via Host · BossFang task board`.

### C8 Verification Completed

- `bun run test` passed: **52/52**.
  - Added tests verify:
    - Seed produces correct cron, assignee, and provenance.
    - Cadence formatter covers `cron`, `every`, and `at` formats.
- TypeScript and lint were clean for touched files.
  - Remaining repo-wide TypeScript/lint noise is pre-existing in `src/components/ui/*` shadcn templates and `soultrace.tsx`; these were not touched.
- C8 component-import guard holds across all `src/**/components/**`.
- Docker frontend stage built successfully, exit `0`:
  - Image: `parlour-web:c8-6c6c1c8-frontend`
  - `/app/dist/client/index.html` and assets confirmed present inside the image.
  - Docker frontend build is the authoritative frontend verification because local `bun run build` hits the `lovable-tagger` `ERR_REQUIRE_ESM` issue under Bun `1.3.6`.

## Live Verification Debt Carried Forward

C8 was verified at the build/unit tier only. Behavior-tier live E2E remains open and operator-gated.

Unchecked live items:

- BFF authz proxy cannot reach cluster-internal `keto-read` from a local run.
- UAR is not running on `:3001`, so these paths remain unexercised:
  - Agentic-4 signed `AgentArtifact` compile.
  - Agentic-5 live `/agui` stream.
- Reconciler round-trip has not been verified against the live PDP:
  - Charter edit → Keto reflects.

Required operator-deploy work before live E2E can run:

- Apply/deploy the BFF authz/reconciler.
- Bring up UAR.
- Run the remaining unchecked gate items against live Keto/UAR.

## State Updates From C8

- `progress.json` updated to `6/6`.
- `current-waypoint.json` updated to:

```text
EXECUTION_COMPLETE
nextStep: reflect
```

- Captured to the Surreal memory graph: `234` tokens.
- Not committed; project rule is to commit only when explicitly asked.

# Citations

1. [1] stdin
2. [2] manual:parlour/c9-uar-config-admin