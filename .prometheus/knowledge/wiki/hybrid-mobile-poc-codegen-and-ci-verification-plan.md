---
type: Reference
id: hybrid-mobile-poc-codegen-and-ci-verification-plan
title: Hybrid Mobile PoC Codegen and CI Verification Plan
tags:
- hybrid-mobile
- codegen
- ci-verification
- proof-of-concept
- flutter-rust-bridge
- tauri
- surrealdb
- local-first-sync
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-15T23:37:02.827479+00:00
created_at: 2026-07-15T23:37:02.827479+00:00
updated_at: 2026-07-15T23:37:02.827479+00:00
revision: 0
---

## Context

- **Phase:** `phase-codegen-and-ci-verification`
- **Project:** Hybrid Mobile Architecture
- **KBD root:** `/Users/gqadonis/Projects/hybrid-mobile-architecture-src`
- **Captured:** `2026-07-15T22:59:44Z`
- **Status:** `execute_ready` after planning; 0/10 changes complete
- **Plan commit:** `b7038d2`, pushed
- **Plan file:** `.kbd-orchestrator/phases/phase-codegen-and-ci-verification/plan.md`

## Revised Phase Goal

The phase outcome was revised on 2026-07-15: deliver a **working proof-of-concept application**, not only codegen and CI verification.

The PoC must be built in `apps/<name>/` using the repository scaffolds and skills, guided by the KnowMe reference documentation in `docs/reference-app/`:

- Functional spec
- Moodboard
- User journeys

The PoC should prove the skill package end-to-end and showcase the broadest practical supported capability set:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Shared Rust core across Flutter, Tauri, and web
- Feature subset selected through web research on showcase-app best practices and 2026 on-device AI feasibility

## Supporting Objectives

The original codegen/CI scope remains as supporting validation, proven through the PoC:

- Run the real codegen pipeline on the PoC:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - full `flutter pub get`
  - full `pnpm install`
- Confirm pre-codegen warnings clear after generated code and sibling packages exist.
- Resolve or work around the PEM install blocker:
  - `@prometheus-ags/entity-graph-core@workspace:*` is unresolvable outside the PEM monorepo.
- Verify the PoC builds and runs on at least one real target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run on every push:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - boundary test suites against the PoC

## Plan Structure

The committed plan contains **10 OpenSpec proposals** across **3 waves**.

| Wave | Changes | Notes |
|---|---|---|
| W0, serial | **C-101** bootstrap pillars → **C-102** PoC scaffold + first codegen | C-102 is the highest-information step because the codegen pipeline has never run; scaffold fixes feed back into the repository. |
| W1 | C-103 chat → C-104 memory RAG → C-105 local model → C-106 sync, app-sequential; C-107 whisper + C-110 CI in parallel worktree lanes | Demo narrative builds feature-by-feature. |
| W2 | C-108 MCP + Hands agent → C-109 settings + mobile sovereign-mode model | Final feature expansion, then reflection. |

## Plan-Time Decisions

1. **PEM blocker: local tarball pre-resolve**
   - PEM monorepo is present and both packages are prebuilt.
   - Use `pnpm pack` and `file:` tarballs.
   - Controlled via `PEM_HOME` environment variable.
   - Use strip-fallback when PEM is absent.

2. **Sync: local Docker Compose**
   - Use Postgres + Electric for the cross-device demo on one machine.
   - Use status-stub fallback if Electric integration overruns.

3. **Bootstrap flags**
   - Default mode: check-only.
   - Long operations require explicit flags:
     - `--install`
     - `--full`

## Harness Assignments

Assignments follow the proven scorecard:

- `claude`: 8/8
- `codex`: 2/2
- `opencode`: 0/2; no opencode lanes assigned

Model/lane allocation:

- `claude/opus-4.8` for the three hardest tasks:
  - first codegen
  - candle/Metal local model work
  - sync
- `claude/sonnet-5` for feature wiring
- `codex/gpt-5.6-sol` for:
  - whisper
  - settings
  - CI

## Success Criteria

The phase is successful when:

- Bootstrap passes all 4 pillars on the current machine.
- The demo narrative runs live:
  1. voice input
  2. transcription
  3. ingestion
  4. cited answer
  5. offline behavior
  6. sync
- Targets verified:
  - macOS Tauri
  - iOS simulator
  - web
- CI is green.

## Execution State

- Completed: `kbd-plan` for `phase-codegen-and-ci-verification`, step 0 of 10.
- Next command: `/kbd-execute`.
- First dispatch: C-101 bootstrap using `claude/sonnet-5`.
- C-102 should run after C-101 lands.
- Wave-1 fan-out begins after C-102.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification