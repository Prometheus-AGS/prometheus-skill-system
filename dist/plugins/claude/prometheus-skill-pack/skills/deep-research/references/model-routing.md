# Model Routing Reference

## Tier Mapping

| Tier | Capability | Default model | When used |
|------|-----------|--------------|-----------|
| `frontier` | Highest reasoning | claude-opus-4-8 (Opus 4.8) | Planning, verification, synthesis |
| `medium` | Balanced | claude-sonnet-5 (Sonnet 5) | Search, retrieve, collect |
| `small` | Efficient | claude-haiku-4-5 (Haiku 4.5) | Citation formatting, export |

Model IDs are resolved via **liter-llm-bridge**. The bridge maps tier names to
actual model IDs based on the current session's configured policy.

## Stage-to-Tier Mapping

| Stage | Phase key | Tier |
|-------|-----------|------|
| 01 — Planner | `research-plan` | `frontier` |
| 02 — Search | `research-search` | `medium` |
| 03 — Retrieve | `research-retrieve` | `medium` |
| 04 — Collect | `research-collect` | `medium` |
| 05 — Verify | `research-verify` | `frontier` |
| 06 — Resolve | `research-resolve` | `frontier` |
| 07 — Graph | `research-graph` | `frontier` |
| 08 — Cite | `research-cite` | `small` |
| 09 — Report (synthesize) | `research-synthesize` | `frontier` |
| 10 — Export | `research-export` | `small` |

## Environment Variables

| Variable | Effect |
|----------|--------|
| `LITER_LLM_BRIDGE_ENABLED=1` | Activate model routing (default: off) |
| `RESEARCH_FRONTIER_MODEL` | Override frontier tier model ID |
| `RESEARCH_MEDIUM_MODEL` | Override medium tier model ID |
| `RESEARCH_SMALL_MODEL` | Override small tier model ID |

## Fallback Behavior

When `LITER_LLM_BRIDGE_ENABLED` is not set or liter-llm-bridge is unavailable:
- All stages use the session default model (whatever the harness is running)
- No error is raised; routing degrades silently to session default
- `manifest.json` records `"liter_llm_used": false`

## Cost Considerations

For a `deep` run with default routing:
- Frontier stages (01, 05, 06, 07, 09) use ~60% of the total token budget
- Medium stages (02, 03, 04) use ~30%
- Small stages (08, 10) use ~10%

For cost-sensitive runs, set `RESEARCH_FRONTIER_MODEL` to a medium-tier model
to reduce cost at the expense of synthesis quality.
