---
name: start-business-build
description: Top-level orchestrator that takes a one-line business concept ("track competitor pricing", "summarize support tickets nightly") and chains every Prometheus pipeline stage end-to-end — ideation expansion, zeespec constraint capture, iterative-evolver assess+plan, OpenSpec change generation, forge enrich, AI implementation dispatch, forge reflect, pk ingest — and optionally finishes by packaging the result as a LibreFang WASM skill and offering /upload-to-bossfang. Single command from concept to deployable skill.
license: MIT
version: '1.0.0'
authors:
  - Prometheus AGS
metadata:
  category: process
  tags: [orchestrator, pipeline, ideation, headline, end-to-end]
  slash_command: '/start-business-build'
  parent_skill: native-agent
  pipeline_layers: [1, 2, 3, 4]
triggers:
  keywords:
    - start business build
    - new business idea
    - concept to deploy
    - end to end build
    - ideation to deployment
    - one shot build
  semantic: >
    User describes a business outcome at the highest level; this skill chains
    the entire Prometheus skill pack to produce a deployable artifact with
    zero manual stage-handoff.
---

# /start-business-build

The headline experience of the Prometheus skill pack. One command. One concept.
End-to-end execution through all four pipeline layers.

## Usage

```
/start-business-build "<concept>"            # interactive, full pipeline
/start-business-build "<concept>" --dry-run  # show plan, don't execute
/start-business-build "<concept>" --skip-deploy  # stop after forge reflect
```

## Pipeline

```
"<concept>"
    │
    ▼ Stage 1: Ideation Mindmap (stub in v1; full in phase-ideation-onramp)
    │   surreal-memory generate_ideation_mindmap → 6-branch concept tree
    │
    ▼ Stage 2: ZeeSpec Constraint Interrogation (Layer 1)
    │   Zachman 5W1H × 60 questions → constraint manifest with GO/CAUTION/NO-GO
    │
    ▼ Stage 3: Iterative-Evolver Strategic Plan (Layer 2)
    │   Assess → Analyze → Plan → ordered change list
    │
    ▼ Stage 4: OpenSpec Change Set (Layer 3)
    │   For each change: GIVEN/WHEN/THEN proposals, audit trail
    │
    ▼ Stage 5: forge enrich + AI implementation (Layer 4)
    │   Per change: forge enrich → dispatch to Claude/Codex → forge reflect
    │   pk ingest after each reflect (Karpathy loop closes)
    │
    ▼ Stage 6 (optional): Package + deploy
    │   forge package-librefang → <name>.lf-skill.zip
    │   Offer /upload-to-bossfang <url> interactively
    │
    ▼  ✅
    Concept → working code → deployed skill
```

## Inputs

| Input | Default | Source |
|---|---|---|
| `<concept>` | (required) | CLI positional arg |
| `working_dir` | cwd | env or implied |
| `model_class` | `frontier` for plan; `tiered` for execute | from `.kbd-orchestrator/project.json` if present |
| `skip_deploy` | false | `--skip-deploy` flag |
| `bossfang_url` | (none) | optional `--bossfang <url>`; if absent, prompts at end |
| `target` | `librefang-wasm` | one of `docker`/`librefang-wasm`/`both` (passed to native-agent if a new agent is generated) |

## Pre-flight Checks

Before stage 1, the orchestrator verifies (using the same pattern as
`scripts/check-prerequisites.sh`):

| Check | Purpose |
|---|---|
| `forge --version` resolves | Layer 4 enrichment available |
| `pk --version` resolves | Karpathy loop closes |
| `surreal-memory-server` reachable | mindmap + cross-session state |
| `liter-llm` reachable OR raw provider keys in env | model dispatch works |
| `wasm32-unknown-unknown` target installed | only if `target ∈ {librefang-wasm, both}` |

If any check fails, the orchestrator suggests the exact `npm run doctor`
command and exits with an actionable message — it does NOT silently degrade.

## Failure Modes & Recovery

The orchestrator writes a checkpoint after every stage to
`.prometheus/business-builds/<slug>/state.json`. If any stage fails or the
user interrupts, re-running `/start-business-build` with the same concept
resumes from the last successful checkpoint.

| Stage | Failure | Recovery |
|---|---|---|
| 1 (ideation) | mindmap generation failed | retry up to 3× with backoff; on persistent failure, fall through with concept text only |
| 2 (zeespec) | constraint manifest empty | abort — usually means concept is too vague; suggest user refine |
| 3 (evolver) | plan empty or contradictory | abort with the evolver's diagnostic; usually means contradictory zeespec constraints |
| 4 (openspec) | OpenSpec absent | fall back to native KBD change format |
| 5 (forge/AI) | per-change enrichment fails | log, skip, continue; report skipped changes at end |
| 5 (forge/AI) | implementer rejects task | reflect with `rejected` status; pk ingest captures the lesson |
| 6 (package) | wasm build fails | log, skip deploy, prompt user to fix |
| 6 (upload) | /upload-to-bossfang fails | the upload's own failure modes apply (see that skill) |

## Acceptance / Done Criteria

The orchestrator declares success when:

1. Every stage 5 change has either `accepted` or `rejected` status (no `pending`).
2. `pk ingest` ran for every accepted change.
3. If `target ∈ {librefang-wasm, both}` and `--skip-deploy` was NOT set:
   `<name>.lf-skill.zip` exists at the working dir.
4. If `--bossfang <url>` was given: the GET-back of the manifest from
   `<url>/skills/<name>` matches what was uploaded.

## Example Sessions

### Minimal concept → deployed WASM skill

```
$ /start-business-build "track shipping-cost trends across our top 5 carriers"

Stage 1: Ideation mindmap...                            ✅
Stage 2: ZeeSpec — 60 questions answered, 4 NO-GO       ✅ (manifest at .prometheus/.../zeespec.md)
Stage 3: Evolver plan — 3 changes ordered               ✅
Stage 4: OpenSpec changes generated                     ✅
Stage 5: change-001 (carrier-data-scraper)              ✅ accepted
Stage 5: change-002 (price-trend-analyzer)              ✅ accepted
Stage 5: change-003 (alert-dispatch)                    ⚠ rejected (carrier API rate limits)
        pk ingest captured: "carrier API rate limits force alerting to be daily, not realtime"
Stage 6: forge package-librefang ./shipping-cost-watch  ✅ → shipping-cost-watch.lf-skill.zip (78 KB)
Stage 6: /upload-to-bossfang? (Y/n)                     y
        URL: https://bossfang.example.com               ✅
        Skill installed and verified.

✨ Done. View at https://bossfang.example.com/skills/shipping-cost-watch
```

### Dry run

```
$ /start-business-build "track shipping costs" --dry-run

Would execute:
  Stage 1: Ideation mindmap (~30s, 1 frontier-model call)
  Stage 2: ZeeSpec interrogation (~2m, 12 frontier calls)
  Stage 3: Evolver plan (~1m, 4 frontier calls)
  Stage 4: OpenSpec generation (~30s, 3 frontier calls)
  Stage 5: forge enrich+implement+reflect × 3 changes (~15m, mixed model classes)
  Stage 6: package-librefang + offer upload (~30s, no model calls)

Estimated cost: $4.20 (frontier) + $0.80 (tiered)
Estimated wall time: 20m
```

## Reference

- [`scripts/orchestrate.sh`](scripts/orchestrate.sh) — the implementation.
- Each stage script lives in its parent skill:
  - Stage 1: `surreal-memory` MCP `generate_ideation_mindmap` tool
  - Stage 2: `skills/process/zeespec-interrogator/`
  - Stage 3: `skills/process/iterative-evolver/`
  - Stage 4: `openspec/` integration in `skills/process/kbd-process-orchestrator/`
  - Stage 5: `tools/forge-rs/` (`forge enrich` / `forge reflect`)
  - Stage 6: `forge package-librefang` (queued for phase-librefang-wasm-onramp)
            + `/upload-to-bossfang` (this same change, sibling sub-skill)

## Notes for Implementing AI

- Stage 1 mindmap generation is a stub in v1 — it currently echoes the
  concept text into the next stage. Full ideation lives in
  `phase-ideation-onramp`.
- Stage 6 packaging requires `forge package-librefang` which is queued as
  a `tools/forge-rs/.forge/changes/forge-package-librefang/` change. Until
  that lands, stage 6 falls back to a manual instruction set the orchestrator
  prints.
- The orchestrator MUST emit `phase-progress` events on stdout in JSON
  format so the calling tool (Claude Code, OpenCode) can render a live UI.
  Format: `{"stage": 3, "status": "running", "msg": "evolver plan ..."}`.
