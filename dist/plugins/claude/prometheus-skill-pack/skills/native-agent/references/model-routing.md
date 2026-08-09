# Native Agent — Model Routing

`SKILL.md` declares `model_routing.routing_reference: references/model-routing.md`.
This is that file. It documents which model class each phase of agent generation
resolves to, and why.

## Phase → class map

| Phase | Class | Rationale |
|---|---|---|
| `agent-specify` | `frontier` | Turning an open-ended request into a concrete agent spec — protocol choices, tool surface, and scope all live here. Ambiguity is highest at this boundary. |
| `agent-plan` | `frontier` | Crate decomposition and configuration design from an ambiguous spec. |
| `agent-generate` | `tiered` | Most file emission is mechanical template substitution (`small`); `system_prompt.md` and `agent.toml` carry judgment and route higher. |
| `agent-validate` | `small` | `cargo check` and `npm install` are deterministic tool invocations with no reasoning content. |
| `adv-review-packet` | `small` | Pure bash packet assembly. No LLM call. |
| `adv-review-judge` | `frontier` | Open-ended defect hunting, and it **must resolve to a model different from the packet's `producer_model`**. |

## Resolution

Model names resolve through the shared resolver,
`shared/scripts/lib/kbd-model-resolve.sh`, with AWS-CLI precedence:

```
explicit flag > PROMETHEUS_KBD_<ROLE>_MODEL > ~/.prometheus/kbd/models.toml
              > .kbd-orchestrator/project.json model_policy > built-in default
```

`policy_source` in the frontmatter names the lowest-precedence layer
(`project.json → model_policy`), not the only one. When `project.json` has no
`model_policy` block — the common case outside a KBD phase — resolution falls
through to `models.toml` and then to the built-in defaults. Never silently
downgrade a `frontier` phase.

## The judge must not be the producer

Agent generation ends in an adversarial review (`--mode agent`). That review's
only guarantee is that a **different** model judged the work, so two rules bind:

1. `KBD_PRODUCER_MODEL` must name the model running the generation session.
   When it is unset the creator **refuses to dispatch** — exit 2, no packet, no
   findings file. It is never defaulted, because a synthesized producer identity
   would make the collision check pass against a guess and record
   `verified-distinct` for a comparison that never happened.
2. The `judge` role resolves independently, falling back to `critic` on
   collision. If every configured model matches the producer, the dispatcher
   logs `JUDGE_MODEL_COLLISION` and the findings record
   `cross_model_check: same-model-collision` — which is **not** a passing review,
   whatever the verdict says.

See `skills/process/adversarial-review/references/model-configuration.md` for the
two-file configuration split, and `docs/guide/09a-adversarial-review.md` for the
failure this gate exists to prevent.
