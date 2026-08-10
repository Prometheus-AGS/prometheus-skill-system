# Model Routing Reference

## Phase → Model Class

| Phase Key | Class | Rationale |
|---|---|---|
| `zeespec-interrogate` | frontier | Deep constraint extraction from user answers; quality of classification matters more than speed |
| `zeespec-score` | small | Deterministic computation from recorded classifications; no reasoning required |
| `zeespec-manifest` | frontier | Synthesizes 60 answers into a coherent constraint picture; requires cross-dimension reasoning |
| `zeespec-persist` | small | Structured JSON writes from validated state; no reasoning required |
| `zeespec-status` | small | Read-only reporting from known files |

## Class Definitions

| Class | Description | Examples |
|---|---|---|
| `frontier` | Highest-capability model; required for reasoning under ambiguity | claude-sonnet-4-6, gpt-4o |
| `medium` | Mid-tier model; structured tasks with moderate complexity | claude-haiku-4-5, gpt-4o-mini |
| `small` | Cheap, fast model; deterministic structured tasks | local Qwen 2.5, mistral-7b |

## Routing Directive Format

Emit before each phase transition:

```
[MODEL_ROUTING] phase=zeespec-interrogate class=frontier model=claude-sonnet-4-6 env=cloud
```

Append to `.zeespec/<subject>/model-routing.log`.

## Policy Source

Read from `.kbd-orchestrator/project.json → model_policy` if present.
If absent, use defaults from this file.

## liter-llm Integration

If `liter-llm-bridge` is active, it intercepts `[MODEL_ROUTING]` directives
and routes to the appropriate endpoint. ZeeSpec does not need to know the
concrete endpoint — it emits the class directive and liter-llm resolves.
