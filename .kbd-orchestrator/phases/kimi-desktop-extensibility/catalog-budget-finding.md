# kde-005 verdict — Kimi Desktop catalog budget

_2026-08-05. Method: source inspection of the shipped listing code + measurement
of the 145 installed descriptions._

## Verdict: a per-skill cap exists; a shared budget does not

**Kimi does NOT behave like Codex.** The two limits are structurally different,
and the difference decides whether curation is needed.

| | Codex | Kimi Desktop |
|---|---|---|
| Limit | One **shared** catalog budget | **Per-skill** description cap |
| Effect of adding a skill | Shortens *every other* skill's description | No effect on any other skill |
| Cap | Elastic, degrades with count | Fixed **250 chars** (`LISTING_DESC_MAX`) |
| Cap on skill *count* | Yes, effectively | **None found** |

So the premise carried since assess — "145 skills may already be truncating each
other" — is **false for this runtime**. Adding the 146th skill does not shrink
the other 145.

## Evidence

From `agent-core/dist/index.mjs`:

```js
const LISTING_DESC_MAX = 250;

function formatModelSkill(skill) {
  const lines = [`- ${skill.name}: ${truncate(skill.description, LISTING_DESC_MAX)}`];
  if (typeof skill.metadata.whenToUse === "string" && skill.metadata.whenToUse.length > 0)
    lines.push(`  When to use: ${skill.metadata.whenToUse}`);
  lines.push(`  Path: ${skill.path}`);
  return lines;
}
```

`getModelSkillListing()` renders every invocable skill through
`renderGroupedSkills` — a plain loop with **no slice, no count limit, and no
running total**. The result lands in the prompt as `KIMI_SKILLS`.

Two details worth keeping:

- `truncate` is grapheme-aware (`Intl.Segmenter`), so it cuts on character
  boundaries rather than mid-codepoint.
- **`whenToUse` is NOT truncated.** It is emitted in full on its own line. That
  is the escape hatch for anything that will not fit in 250 chars.

## Measurement — 61% of our descriptions are being cut

| | |
|---|---|
| Skills measured | 145 |
| Cap | 250 |
| Median length | **278** |
| Longest | 662 (`gitops-transform`) |
| Over the cap | **89 (61%)** |

Worst offenders: `gitops-transform` 662, `argocd-multicloud` 595,
`adversarial-review` 581, `librefang-wasm-skill` 561, `gitops-bootstrap` 547,
`upload-to-bossfang` 537.

The median sitting above the cap means truncation is the norm here, not an edge
case.

## What this means

**No curation needed.** The Codex remedy — `config/codex-catalog.txt` selecting a
subset — solves a shared-budget problem Kimi does not have. Shipping all 145 is
correct and costs other skills nothing.

**But 89 skills lose their tail.** A description that reads
"... Use when X, Y, or Z" and gets cut at 250 loses exactly the triggering
guidance the model selects on. This is a real, measurable quality loss, distinct
from the budget question that was asked.

## Recommendation (NOT done here — out of scope)

Two options, both changes of their own:

1. **Front-load the trigger.** Rewrite the 89 over-cap descriptions so the first
   250 characters carry the selection-relevant content. No new field, works on
   every harness.
2. **Use `whenToUse`.** Kimi emits it untruncated. Harness-specific, and only
   helps here.

Option 1 is portable and is what the pack's own `description` conventions
already imply. Neither is in this change's scope: rewriting 89 descriptions is
user-visible content editing that deserves its own review.

## OQ-3 status

**CLOSED.** Asked at assess, carried unowned through analyze and spec, owned by
this change, now answered with a measurement and a method.
