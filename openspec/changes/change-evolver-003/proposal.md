# change-evolver-003 — skills/process/pmpo-evolver/SKILL.md + model routing table

**Phase:** pmpo-evolver
**Priority:** HIGH — the entry command; nothing invocable without this
**Gaps:** G-01, G-13
**Goals:** G1, G3
**Model class:** small (authoring — no synthesis)
**Depends on:** change-evolver-001 (schema), change-evolver-002 (feedback source taxonomy)

## Problem

`skills/process/pmpo-evolver/SKILL.md` does not exist (G-01). There is no strategy router entry command for the five evolution perspectives. There is no model routing table specific to pmpo-evolver phases that incorporates liter-llm class assignments (G-13).

## Solution

Create `skills/process/pmpo-evolver/SKILL.md` as the strategy router skill. Create `skills/process/pmpo-evolver/references/model-routing.md` as the liter-llm integration reference.

## New file: SKILL.md

**Frontmatter:**
```yaml
---
name: pmpo-evolver
description: Strategy router for evolving released products — routes to one of five evolution perspectives (competitive, trend, unique-product, idea-validation, self-learning) and orchestrates the full PMPO loop per perspective
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-ags
  category: process
  tags: [pmpo, evolution, strategy, competitive-analysis, self-learning, liter-llm]
---
```

**Entry commands:**
```
/pmpo-evolver <evolution-name> [--perspective <mode>] [--depth quick|standard|deep]
/pmpo-evolver-status <evolution-name>
```

**Sections:**
1. When to use — trigger conditions for each of the 5 perspectives
2. Strategy routing logic — auto-detection table (6 rows: competitive, trend, unique-product, idea-validation, self-learning, combined)
3. PMPO loop per perspective — Assess→Analyze→Plan→Execute→Reflect→Strategic Dream→Persist
4. Model routing per phase — table with phase, class, rationale (13 rows)
5. Context management — link to `references/context-management.md` (change-evolver-010)
6. liter-llm integration — how to detect availability, fallback chain
7. Perspective descriptions — one paragraph per perspective with key artifacts produced
8. Platform compatibility note — all five perspectives work identically across Claude Code, Codex, OpenCode, Kimi, Zed (install-skills-flat.sh auto-discovers)

**Model routing table (inline):**

| Phase | liter-llm class | Rationale |
|-------|----------------|-----------|
| Perspective routing selection | small | File reads + schema check; deterministic |
| Competitive landscape scan | frontier | Cross-domain synthesis, novelty detection |
| Parity matrix generation | medium | Structured comparison, bounded output |
| Trend research synthesis | frontier | Ambiguous external signals requiring judgment |
| Carry-forward aggregation | small | File reads + pattern extraction |
| Idea plausibility gate (Gate 1) | small | Binary yes/no classification |
| Idea domain research (Gate 2) | medium | Web search + bounded synthesis |
| Idea spec generation (Gate 3) | frontier | Novel spec drafting under constraints |
| Feedback source collection | small | Deterministic tool calls; no synthesis |
| Feedback sentiment classification | medium | NLP classification; bounded |
| Learning signal synthesis | medium | Pattern extraction across normalized signals |
| Strategic dreaming | frontier | Open-ended strategic synthesis |
| Evolver reflect | frontier | Quality judgment + delta analysis |

**liter-llm detection (within SKILL.md):**
```bash
# Check if liter-llm MCP is registered
if liter-llm --version 2>/dev/null; then
  MODEL_ROUTING=liter-llm
else
  MODEL_ROUTING=harness-native
fi
```
When liter-llm is available: emit `[MODEL_ROUTING] phase=<key> class=<class>` directives at each phase transition, then call the liter-llm `complete` MCP tool with `model=<class>`.
When not available: the harness model handles all phases (no cost optimization, but full functionality).

## New file: references/model-routing.md

**Contents:**
- liter-llm MCP tool invocation contract: `complete(model="small"|"medium"|"frontier", messages=[...])` → returns completion text
- Provider discovery: how to read `~/.config/liter-llm/config.toml` or `$LITER_LLM_CONFIG`; how `list_models` MCP tool returns configured aliases
- Health check: `health` MCP tool — verifies which providers are currently reachable; fall through to next class if provider unreachable
- Decision protocol: if `medium` class configured and reachable → use; if not → fall through to `frontier`; never silently upgrade `small` to `frontier`
- Cost tracking: call `get_cost` after each `complete` call during development; target: feedback collection + changelog ingestion ≤10% of frontier-all cost
- Full class→provider mapping table (example): Anthropic claude-haiku-4-5 = small; claude-sonnet-4-6 = medium; claude-opus-4-8 = frontier; Groq llama-3.3-70b = medium; Ollama qwen3:4b = small
- The `[MODEL_ROUTING]` directive format (canonical from iterative-evolver): `[MODEL_ROUTING] phase=evolver-<key> class=<class> model=<resolved-model> env=<env>`; logged to `.evolver/<name>/model-routing.log`

## Acceptance criteria

- [ ] `skills/process/pmpo-evolver/SKILL.md` exists with valid YAML frontmatter
- [ ] `npm run validate:strict skills/process/pmpo-evolver` passes
- [ ] SKILL.md is under 500 lines
- [ ] `skills/process/pmpo-evolver/references/model-routing.md` exists
- [ ] Model routing table in SKILL.md has all 13 phase entries
- [ ] liter-llm fallback chain documented in both SKILL.md and model-routing.md
