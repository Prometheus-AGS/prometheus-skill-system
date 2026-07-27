# Isolation and Model Routing

## Why fresh context, why cross-model

Two independent failure modes are being defended against:

1. **Context contamination.** A reviewer that shares the implementing
   session's context has already absorbed the author's framing, rationale,
   and confidence. Information asymmetry is what makes review work — the
   evolver codified this for its collection tasks ("always run as isolated
   subprocesses — never inline"), and this skill applies the same rule to
   judgment.
2. **Self-grading bias.** A model reviewing its own output systematically
   under-reports its own errors. The repo's existing separation agents
   (kbd-idea-critic, kbd-task-verifier, kbd-goal-evaluator) exist for exactly
   this reason; adversarial-review extends the rule to concrete diffs and
   planning artifacts, and strengthens it from "separate context" to
   "separate model" via liter-llm.

Isolation is **structural**, not honor-system: the judge is an API call whose
entire input is the review packet (`build-review-packet.sh` output). There is
no channel through which session history could reach it.

## Model resolution

```
[MODEL_ROUTING] phase=adv-review-judge class=frontier model=<resolved> producer=<producer>
```

1. Read `producer_model` from the packet (progress.json → `KBD_PRODUCER_MODEL`
   → `ANTHROPIC_MODEL` → `unknown`).
2. Read the alias table from `~/.config/liter-llm/config.toml`
   (`LITER_LLM_CONFIG` overrides the path).
3. Candidate order: `frontier`, then `medium`, then `small` alias values.
   The first candidate that differs from the producer wins. A
   different-model medium judge beats a same-model frontier self-grade —
   tier purity is sacrificed for independence, never the reverse.
4. All candidates match the producer → `JUDGE_MODEL_COLLISION` warning to
   stderr, proceed with the frontier alias. Never silent, never fatal.
5. No alias table at all → pass the literal class name `frontier` through and
   let liter-llm resolve it.

Phase classes (declared in SKILL.md frontmatter `model_routing`):

| Phase | Class | Rationale |
|---|---|---|
| `adv-review-preflight` | small | env scan + file reads, deterministic |
| `adv-review-packet` | small | pure bash assembly, no LLM call |
| `adv-review-judge` | frontier | open-ended defect hunting requires full reasoning |

## Fallback chain

Following liter-llm-bridge semantics — warn loudly, degrade honestly, never
block the pipeline on missing review infrastructure:

| Tier | Trigger | Guarantee | Marker |
|---|---|---|---|
| liter-llm | binary + provider available | fresh context AND cross-model | `isolation_mode: "liter-llm"`, exit 0 |
| harness-native subagent | `dispatch-judge.sh` exit 3 | fresh context only (same model family) | `isolation_mode: "harness-native"` |
| skip | exit 4 / no subagent capability | none | `adversarial_review: SKIPPED (<reason>)` in progress.json |

The harness-native fallback prompt is **exactly** the mandate file plus the
packet JSON — adding anything else (task summaries, prior conversation)
breaks the isolation contract.

## Preflight contract

`preflight-models.sh` output (cached at `.kbd-orchestrator/model-preflight.json`):

```json
{
  "status": "ok | degraded | needs_configure | no_providers | unavailable",
  "providers_detected": ["anthropic", "groq"],
  "classes_available": ["small", "medium", "frontier"],
  "aliases": { "frontier": "anthropic/claude-sonnet-4-6", "medium": "groq/llama-3.3-70b-versatile" },
  "distinct_models": 2,
  "config_path": "~/.config/liter-llm/config.toml",
  "config_exists": true,
  "checked_at": "2026-07-27T00:00:00Z"
}
```

Status handling by the calling skill:

- `unavailable` → offer `/liter-llm-bridge install`; until then fall back tier 2.
- `no_providers` → ask the user which providers to configure and name the env
  var per provider (canonical table: liter-llm-bridge
  `references/provider-env-vars.md`). **Never collect or store key values** —
  the user exports the key; config.toml holds aliases only.
- `needs_configure` → run `/liter-llm-bridge configure` (fills gaps only;
  pinned aliases are never overwritten).
- `degraded` → warn: only one distinct model; collisions expected until a
  second provider is configured.
- `ok` → proceed.

Cache invalidation: `--force`, config.toml newer than cache, or age > 24 h.
