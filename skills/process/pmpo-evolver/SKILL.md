---
name: pmpo-evolver
description: Strategy router for evolving released products — routes to one of five evolution perspectives (competitive, trend, unique-product, idea-validation, self-learning) and orchestrates the full PMPO loop per perspective
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-ags
  category: process
  tags: [pmpo, evolution, strategy, competitive-analysis, self-learning, liter-llm, iterative-evolver]
---

# pmpo-evolver

Strategy router for evolving released products across five evidence-based perspectives. Sits above `iterative-evolver`, `kbd-evolve`, and `pmpo-outer-loop` as the entry point for product evolution work.

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting pmpo-evolver — <evolution-name> (perspective 1 of 5)
```

When all selected perspectives are synthesized, emit:

```
Completed pmpo-evolver — <evolution-name> (perspectives: <list>)
```

Emit to plain response text — no tool call needed.

## Entry commands

```
/pmpo-evolver <evolution-name> [--perspective <mode>] [--depth quick|standard|deep]
/pmpo-evolver-status <evolution-name>
```

## When to use

Use `/pmpo-evolver` when:
- A product has reached a stable release and you want to plan the next meaningful evolution
- You want data-driven perspective selection (auto mode) rather than manual direction
- You are running a standing outer loop and need per-tick perspective routing
- An operator has an idea they want to research, validate, and spec before committing to a KBD phase

Do NOT use for initial product construction — use `iterative-evolver` or `kbd-evolve` directly.

## Five perspectives

### 1. competitive
Research latest developments in competing products. Produces a parity matrix: features they have, we have, or where we lead. Routes to `iterative-evolver` with competitive gaps as goals.

**Trigger signals:** `competitor-registry.json` exists and is stale; competitor-scan feedback source present in `loop.json`

### 2. trend
Web research on domain innovations, emerging standards, community signals. Produces a domain taxonomy lookup + trend brief. Routes to `iterative-evolver` analyze phase.

**Trigger signals:** No competitor registry; `--perspective trend`; domain-taxonomy.md present

### 3. unique-product
When no direct competitors exist, research the next logical evolution step from carry-forwards, design philosophy, and web research on the problem space. Uses carry-forward-aggregate output as primary input.

**Trigger signals:** `design-philosophy.md` present; no competitor registry; carry-forwards exist

### 4. idea-validation
Operator proposes an idea. Three-gate pipeline: plausibility → domain research → spec + human gate. Routes to KBD phase seeding on approval.

**Trigger signals:** `--perspective idea-validation --idea "<text>"`; or operator-provided idea in `loop.json`

### 5. self-learning
Collect and synthesize learning signals from feedback sources (gh-issues, commit-history, sentiment feeds, telemetry). Uses Karpathy-style usage analysis. Routes to `iterative-evolver` assess phase with signal digest as context.

**Trigger signals:** `loop.json` contains `gh-issues`, `commit-history`, `sentiment-feed`, or `telemetry-url` sources

---

## Strategy routing logic

The router runs before any other work:

```
[MODEL_ROUTING] phase=evolver-route class=small
```

| Condition | Selected perspective |
|-----------|---------------------|
| `--perspective <mode>` | Use that mode directly |
| `competitor-registry.json` exists AND last_scanned > staleness_ttl | `competitive` |
| `feedback_sources` contain gh-issues/commit-history/sentiment | `self-learning` |
| `design-philosophy.md` exists AND no competitor registry | `unique-product` |
| carry-forwards exist AND trends not recently run | `trend` |
| `--perspective combined` OR no clear signal | `combined` (sequential) |

Combined mode uses `perspective_cursor` to track which perspectives have completed within a single evolver run.

---

## PMPO loop per perspective

Each perspective executes a complete PMPO cycle:

```
Assess → (Analyze where needed) → Plan → Execute → Reflect → Strategic Dream → Persist
```

**competitive:** Assess gap between parity matrix and our features → Analyze competitor changelogs → Plan missing features → Execute via KBD → Reflect on parity change → Dream on strategic implications → Persist updated parity matrix

**trend:** Assess domain taxonomy sources → Analyze emerging standards → Plan adoption → Execute → Reflect on trend coverage → Dream → Persist

**unique-product:** Assess carry-forwards + design philosophy → Analyze next-step options → Plan chosen direction → Execute → Reflect → Dream → Persist

**idea-validation:** Gate 1 (plausibility) → Gate 2 (domain research) → Gate 3 (spec + human gate) → KBD phase seed → Execute → Reflect → Dream → Persist

**self-learning:** Collect feedback sources → Normalize to LearningSignal[] → Analyze signal patterns → Plan response actions → Execute → Reflect on signal change → Dream on direction → Persist updated signals

---

## Model routing per phase

All phases emit `[MODEL_ROUTING]` directives. When liter-llm is available, each phase calls `complete(model=<class>)` through the liter-llm MCP bridge. When not available, the host model handles all phases.

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

For the full model routing protocol, liter-llm provider discovery, and cost tracking, see [references/model-routing.md](references/model-routing.md).

---

## liter-llm integration

**Detect availability:**
```bash
if command -v liter-llm > /dev/null 2>&1 || liter-llm --version > /dev/null 2>&1; then
  MODEL_ROUTING=liter-llm
else
  MODEL_ROUTING=harness-native
fi
```

**Emit directive at each phase transition:**
```
[MODEL_ROUTING] phase=evolver-<key> class=<class> model=<resolved-model> env=<env>
```

**Use model in each phase:**
```bash
# Example: competitive scan
# [MODEL_ROUTING] phase=evolver-competitive-scan class=frontier
. "${CLAUDE_PLUGIN_ROOT}/shared/scripts/lib/kbd-model-resolve.sh"
result="$(kbd_complete "$(kbd_resolve_role judge)" "$SYSTEM_PROMPT" "$USER_PROMPT" 2048)" || {
  echo "[evolver] model call failed (see message above) — continuing degraded" >&2
}
```

> There is **no `liter-llm complete`** subcommand — the binary ships only `api` and
> `mcp` (it is a proxy *server*). This skill documented and called that
> non-existent command; because callers only checked that the *binary* existed and
> masked failures with `2>/dev/null || echo "{}"`, extraction silently returned
> empty results. `kbd_complete` speaks OpenAI REST to the resolved gateway and
> reports failures instead of swallowing them.

**Fallback chain:** `kbd_complete` over the REST gateway → harness-native model override → host model. Never silently upgrade `small` to `frontier`, and never let a failed call read as an empty result.

For discovery (which providers are configured, health check, class-to-model mapping), see [references/model-routing.md](references/model-routing.md).

---

## Context management

Long-running collection tasks (feedback-digest, changelog-fetch, carry-forward-aggregate, post-cycle-dream) are always run as isolated subprocesses — never inline in the evolver session. The session reads only the normalized JSON output. This preserves the main context window for strategic reasoning.

See [references/context-management.md](references/context-management.md) for the full context budget rules and cost estimation table.

---

## Platform compatibility

All five perspectives work identically across Claude Code, Codex, OpenCode, Kimi, and Zed. The `install-skills-flat.sh` script auto-discovers this skill and sub-skills. No platform-specific configuration is required.

Sub-skills are in `skills/validate-idea/` and are invoked via `/validate-idea` on any platform.

---

## Key artifacts produced

| Perspective | Primary artifacts |
|-------------|------------------|
| competitive | `.evolver/<name>/competitor-registry.json`, `.evolver/<name>/parity-matrix.json` |
| trend | `.evolver/<name>/trend-brief.md` |
| unique-product | `.evolver/<name>/carry-forwards.json`, next KBD phase goals |
| idea-validation | `.evolver/<name>/archive/<idea-id>/manifest.json`, optional `SPEC.md` |
| self-learning | `.evolver/<name>/learning-signals-<tick>.json`, `evolver-lessons.md` |
| all | `.evolver/<name>/state.json` with updated `perspective`, `learning_signals[]`, `evolver_lessons[]` |

## References

- [feedback-sources.md](references/feedback-sources.md) — All feedback source types with examples
- [model-routing.md](references/model-routing.md) — liter-llm class assignments, provider discovery, cost tracking
- [context-management.md](references/context-management.md) — Context budget rules and isolation patterns
- [competitive-analysis.md](references/competitive-analysis.md) — Competitor registry + parity matrix formats
- [learning-signals.md](references/learning-signals.md) — LearningSignal normalization protocol
- [strategic-dreaming.md](references/strategic-dreaming.md) — Post-cycle strategic dreaming protocol
- [domain-taxonomy.md](references/domain-taxonomy.md) — Domain keyword → research source mapping
