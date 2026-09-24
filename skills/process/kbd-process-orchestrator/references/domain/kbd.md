# KBD Philosophy & Universal Reference

This file documents the KBD (Knowledge-Based Development) philosophy.
It is used during the Assess and Plan phases to ground decisions.

Read this file on every KBD session regardless of which project is active.

---

## What is KBD?

**Knowledge-Based Development (KBD)** is an iterative, PMPO-driven software
development process designed to keep AI tools coherent, aligned, and productive
across multi-session, multi-tool, multi-phase projects.

KBD solves three core problems:

1. **Session amnesia** — AI tools lose context between sessions
2. **Tool fragmentation** — different AI tools each have their own planning state
3. **Spec drift** — implementation diverges from documented intent over time

---

## Core Principles

### 1. Durable State is Shared

The canonical runtime journal under `.kbd-orchestrator/` is authoritative.
Tools read generated projections and mutate state through typed commands or
supported driver adapters. Legacy file-only state is migration input, not
permission to hand-edit generated progress or waypoint files.

### 2. Waypoint-First Resumption

On every session start, load `.kbd-orchestrator/current-waypoint.json` before
doing anything else, then inspect canonical status and pending tasks. A stored
`exactNextCommand` is contextual guidance; it can be stale.

### 3. KBD is the Orchestrator, Tools are Workers

KBD owns phase state. AI tools (Roo, Cursor, Codex, etc.) are execution agents.
They report through `kbd-apply` task boundaries and typed KBD commands; the
runtime regenerates progress and waypoint views.

### 4. OpenSpec is Optional Enhancement

OpenSpec provides structured change management with proposal → design → tasks.
If OpenSpec is not present, KBD uses `.kbd-orchestrator/changes/<id>/change.md`
with the same structure. KBD never depends on OpenSpec to function.

### 5. Progress is Committed to Git

Review and commit intended KBD artifacts and projections under project policy.
Git provides history and review; the canonical runtime coordinates live writes.
Commits provide:

- Cross-tool visibility (git pull gives any tool the latest state)
- History and auditability
- Reviewable changes (merge conflicts do not replace runtime coordination)

### 6. Assess Before Plan, Plan Before Execute, Execute Before Reflect

The PMPO loop is not optional. Skipping Assess means you plan without facts.
Skipping Plan means you execute without direction. Skipping Reflect means you
lose lessons and fail to seed the next phase.

Dispatch artifacts begin Execute. The parent stage stays active while workers
complete all assigned changes/tasks and required QA, independent review,
verification, and archival. Only then may `execute:after` and the execute
completion handoff occur; `execution.md` or `execute-dispatch.json` alone never
unlocks Reflect. See `skills/kbd-execute/SKILL.md` for the boundary checklist.

---

## The PMPO Lifecycle

```
┌──────────────────────────────────────────────────────┐
│                KBD Phase Lifecycle                    │
│                                                      │
│  ┌─────────┐    ┌─────────┐    ┌──────────┐        │
│  │ ASSESS  │───▶│  PLAN   │───▶│ EXECUTE  │        │
│  └─────────┘    └─────────┘    └────┬─────┘        │
│       ▲                             │               │
│       │                             ▼               │
│       │                        (tools do work)      │
│       │                        typed task events    │
│       │                             │               │
│       │                             ▼               │
│       │                        ┌─────────┐         │
│       └────────────────────────│ REFLECT │         │
│                                └─────────┘         │
│                                      │              │
│                                      ▼              │
│                              Next Phase Seed         │
└──────────────────────────────────────────────────────┘
```

---

## Phase Naming Convention

Phases should be named descriptively with a prefix:

```
phase-0-baseline
phase-1-foundation
phase-2-<module-name>
phase-3-<module-name>
phase-N-production-hardening
```

---

## Project Context Discovery Order

KBD reads project identity from, in priority order:

1. Explicit argument (e.g., `/kbd-plan hotseaters phase-2`)
2. `.kbd-orchestrator/project.json` → `name`
3. `AGENTS.md` — first H1 or project name mention
4. `CLAUDE.md` — first H1 or project name mention
5. `README.md` — first H1
6. `package.json` → `name`
7. `Cargo.toml` → `[package] name`
8. `pyproject.toml` → `name`
9. Directory name of the repository root

---

## Cross-Tool Handoff Protocol Summary

See `references/cross-tool-handoff.md` for the full protocol.

Quick reference:

1. Starting a task → `kbd-apply begin-task` with canonical IDs and real totals.
2. Completing a task → `kbd-apply end-task`; the runtime derives progress.
3. Completing code → typed change transition; track evidence, certification,
   and publication separately, then use driver `verify` / `archive`.
4. Hitting a blocker → typed `prometheus kbd blocker record` with remediation.
5. Resuming → read the waypoint, canonical status, and pending tasks; never
   manually update generated views.
