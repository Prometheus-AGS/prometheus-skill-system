# Model Routing

The skill creator routes each PMPO phase through the project model policy in
`.kbd-orchestrator/project.json`:

| Phase | Policy key | Class |
| --- | --- | --- |
| Specify | `creator-specify` | `frontier` |
| Plan | `creator-plan` | `frontier` |
| Execute | `creator-execute` | `tiered` |
| Reflect | `creator-reflect` | `frontier` |
| Persist | `creator-persist` | `small` |

For a `tiered` Execute phase, route each independently generated artifact by
risk. Use `small` for deterministic copying and manifest edits, `medium` for
bounded adaptations, and `frontier` for new architecture, security-sensitive
behavior, or ambiguous cross-file contracts. Record the chosen class with the
artifact plan before generation.

If `model_policy` or a phase key is absent, use `frontier`. This compatibility
fallback preserves quality for projects created before phase routing metadata
was introduced. A policy may map classes to any available provider model; this
skill never hard-codes provider model names.
