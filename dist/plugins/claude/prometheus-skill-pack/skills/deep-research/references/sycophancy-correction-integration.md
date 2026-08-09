# sycophancy-correction Integration

Stage 05 (Verify) applies sycophancy-correction bias detection to each source's
extracted claims. Sources that systematically over-claim or suppress
contradictory evidence receive a credibility penalty.

## Tool Used

`detect_sycophancy` from the `sycophancy-correction` MCP server.

```
detect_sycophancy(
  text = <source_claims_concatenated>,
  strictness = "standard"  # or "strict" for exhaustive depth
)
```

## Strictness by Depth

| Depth | Strictness |
|-------|-----------|
| shallow | `permissive` |
| deep | `standard` |
| exhaustive | `strict` |

## Severity → Credibility Penalty

| Severity | Penalty |
|----------|---------|
| `critical` | -20 points |
| `high` | -15 points |
| `medium` | -10 points |
| `low` | -5 points |
| none | 0 |

Multiple severity flags stack (max total penalty: -30 points).

## Common Sycophantic Patterns in Research Sources

Patterns that trigger detection:

- **Over-confidence without basis** — "Clearly the best solution..." without
  supporting benchmarks or citations
- **Contradiction suppression** — Mentions competing approaches only to dismiss
  them without evidence
- **Hedging removal** — Presents preliminary findings as settled fact
- **Vendor-favorable framing** — Source is vendor's own documentation claiming
  unqualified superiority

## Availability Check

When `sycophancy-correction` MCP is unavailable:
- Stage 05 skips the bias check
- No sycophancy penalty is applied
- `manifest.json` records `"sycophancy_correction_used": false`
- Source credibility is scored on the 5-dimension rubric only

## Example

```
detect_sycophancy(
  text = "Qdrant is clearly the fastest vector database. All benchmarks confirm...",
  strictness = "standard"
)
→ { severity: "medium", patterns: ["over_confidence", "unsupported_superlative"] }
→ credibility_penalty = -10
```
