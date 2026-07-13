---
license: MIT
name: kbd-process-orchestrator-architecture
version: '1.0.0'
description: >
  Reference material extracted from kbd-process-orchestrator/SKILL.md —
  the Tool Registry, Knowledge Stack, Integration Layer, integration
  timing, Global Phase Map, and the rationale for keeping integrations
  global rather than copied. Read on first use; not loaded on every
  invocation.
metadata:
  tags: [process, orchestration, reference]
---

# KBD Architecture Reference

This file collects the verbose reference material that used to live in
`SKILL.md` and that the orchestrator only needs to consult on first use or
when extending the integration layer. The contract (progress signals,
project discovery, waypoint protocol, hooks, completion semantics) stays
inline in `SKILL.md` — that is what every invocation reads.

## Tool Registry

KBD recognizes the following execution agents. Each has a recommended usage pattern:

| Tool                          | Best For                                                    | Entry Point               |
| ----------------------------- | ----------------------------------------------------------- | ------------------------- |
| **Antigravity**               | Complex multi-file features, planning, browser verification | `SKILL.md` slash commands |
| **Roo Code (Architect mode)** | Architecture decisions, system-level planning               | Roo Architect mode prompt |
| **Roo Code (Code mode)**      | Focused implementation of bounded tasks                     | Roo Code mode prompt      |
| **Cursor Agent**              | Multi-file refactoring, parallel subagent tasks             | Cursor Agent mode         |
| **Claude Code**               | Large architectural changes, CLI-driven execution           | `claude` CLI              |
| **Codex (OpenAI)**            | Parallel isolated tasks via git worktrees                   | OpenAI Codex app or CLI   |
| **Cline**                     | Terminal-first agentic workflows with Plan/Act mode         | Cline VSCode extension    |
| **Kilo Code**                 | Targeted code edits in VSCode                               | Kilo Code extension       |
| **Windsurf Cascade**          | Autonomous multi-step tasks with shared session             | Windsurf Cascade panel    |
| **OpenCode**                  | Quick targeted edits and file patches                       | OpenCode CLI/extension    |
| **Human**                     | Decisions requiring judgment, external tool operations      | Manual                    |

---

## Knowledge Stack

| Layer            | Sources                                                                 |
| ---------------- | ----------------------------------------------------------------------- |
| Project Identity | `.kbd-orchestrator/project.json`, `AGENTS.md`, `CLAUDE.md`, `README.md` |
| Spec Baselines   | `openspec/specs/*.md` (if OpenSpec), or project spec directory          |
| Change Specs     | `openspec/changes/<id>/*.md` (if OpenSpec), or change directories       |
| Execution State  | `.kbd-orchestrator/phases/<phase>/` artifacts                           |
| Progress         | `.kbd-orchestrator/phases/<phase>/progress.json`                        |

---

## Integration Layer

KBD delegates specialized work to **4 global skills**. These skills are NOT
copied into this skill's directory — they live in the global `.agent/skills/`
directory and are invoked by reference. This preserves a single source of truth
for each skill, ensures updates propagate automatically, and avoids maintenance split.

Full integration contracts are defined in `references/integrations/`:

| Global Skill           | KBD Phase               | Role                                                                               | Integration Guide                               |
| ---------------------- | ----------------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------- |
| **iterative-evolver**  | Assess                  | Deep codebase + spec gap analysis with cross-session continuity                    | `references/integrations/iterative-evolver.md`  |
| **artifact-refiner**   | Execute (per-change QA) | Constraint-driven code quality gate before archiving each change                   | `references/integrations/artifact-refiner.md`   |
| **bdd-testing**        | Execute (verification)  | Behavioral verification gate — BDD scenarios must pass before DONE                 | `references/integrations/bdd-testing.md`        |
| **pmpo-skill-creator** | Reflect (meta)          | KBD self-improvement — extend kbd with new sub-skills discovered during reflection | `references/integrations/pmpo-skill-creator.md` |

### Why Global, Not Copies?

Each of these skills has its own:

- **PMPO loop** with independent phase states
- **Named file-backed state** (`.evolver/`, `.refiner/`, `.creator/`)
- **Entry commands** that are already registered globally

Copying them would require maintaining two versions, break the single-source-of-truth
principle, and make it impossible to share state between KBD and non-KBD invocations
of the same skill. The integration adapters in `references/integrations/` specify
the exact invocation contract (what to pass, what to read back) without duplicating
any logic.

### Integration Timing in the KBD Loop

KBD operates as both a standalone orchestrator and as the **inner loop**
for iterative-evolver's Execute phase. The integration timing is the same
either way — when invoked from the evolver, the evolver bridge adds
result propagation back to the outer loop.

```
ASSESS
 └─ /kbd-assess (lightweight, waypoint-aware)
    └─ if deep analysis needed: /evolve-assess "<project>-<phase>" ← iterative-evolver

ANALYZE (optional; skippable with recorded reason)
 └─ /kbd-analyze (tiered engineering-landscape research → library-candidates.json)
    └─ delegates to /evolve-analyze when an evolver bridge is present

SPEC
 └─ /kbd-spec (turns assessment + analysis into change specs)
    └─ native-kbd specs (spec.md + tasks.json + verification.md) OR /opsx:new
    └─ ZeeSpec coverage gate: NO-GO blocks spec→plan until /zeespec-interrogate

PLAN
 └─ /kbd-plan (orders the change list)
    └─ auto-detects backend (openspec/ directory or project.json.specBackend)
    └─ reads library-candidates.json; annotates adopt/adapt changes
    └─ if evolver bridge: maps evolver plan items ↔ KBD changes

EXECUTE (per change)
 └─ <executing tool> implements the change
    └─ /bdd-testing: write feature file + step defs   ← bdd-testing
    └─ <tool runs the feature>: pnpm test:bdd
    └─ /refine-validate: constraint QA gate            ← artifact-refiner
       └─ if FAIL: /refine-code for iterative refinement
    └─ /opsx:archive or native archive

REFLECT
 └─ /kbd-reflect (phase retrospective)
    └─ aggregates artifact-refiner QA metrics (pass rate, violations)
    └─ if evolver bridge: writes execution results back to evolver state
    └─ if structural gap found: /extend-skill or /create-skill  ← pmpo-skill-creator
```

---

## Global Phase Map

Phases are project-defined. KBD does not impose a fixed phase sequence.
The orchestrating agent reads `.kbd-orchestrator/phases/` to discover
existing phases and their status.

A typical phase progression for software projects:

| Phase       | Pattern                        | Status          |
| ----------- | ------------------------------ | --------------- |
| Phase 0     | Baseline & KBD Setup           | Recommend first |
| Phase 1     | Foundation / Core Architecture | High priority   |
| Phase N     | Feature Modules (iterative)    | Per roadmap     |
| Phase Final | Production Hardening           | Final           |
