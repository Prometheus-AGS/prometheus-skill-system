# Progress record schema

Each record is a Prometheus `Reference` page with a stable slug, UTC timestamps, revision,
phase/status tags, and these sections:

1. `Intent` — the actual user outcome pursued.
2. `Observed state` — direct evidence only.
3. `Decision and delta` — what changed and why.
4. `Verification` — exact public-boundary or static evidence.
5. `Failure/lesson` — causal lesson, not blame or vague metadata.
6. `Next experiment` — smallest aligned next action toward the full goal.

The project copy belongs in `.prometheus/knowledge/wiki`. The private project superset
belongs in `~/.prometheus/knowledge/private/<project-slug>/wiki`. Reviewed generalized
lessons may be ingested into `~/.prometheus/knowledge/shared`; raw session history may not.
