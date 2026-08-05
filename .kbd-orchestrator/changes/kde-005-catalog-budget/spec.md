# kde-005 — measure the Kimi Desktop catalog/description budget at 145 skills

**Phase:** kimi-desktop-extensibility
**Scope:** measurement only; findings written to the phase directory
**Backend:** native-kbd

## Problem

OQ-3 — "is there a catalog or description budget?" — was carried through the
assess, analyze, **and** spec handoffs with no change owning it. Adversarial
review flagged the orphan. This change owns it.

The question is not hypothetical. Codex has a fixed catalog budget where every
additional skill shortens every other skill's description:

| Codex entries | Avg description | Effect |
|---|---|---|
| ~130 | ~166 chars | model auto-triggers reliably |
| ~200 | ~66 chars | usable |
| ~360 | ~10 chars | broken — skills indistinguishable |

The pack ships **145 skills** to Kimi Desktop in one package. If Kimi budgets
similarly, descriptions may already be truncated, which would silently degrade
skill selection across the whole package.

## Approach

Determine whether Kimi Desktop truncates or drops skill descriptions at 145.

Preferred: find where the daimon renders the skill catalog and inspect what it
produces. Fallback: differential test — install a small package with a
distinctive long description alongside the 145-skill package, and compare how
that description is presented against the same package installed alone.

## Acceptance criteria

1. A written finding stating whether a budget exists and, if so, its observed
   shape (truncation length, entry cap, or none detected).
2. If a budget is found: a recommendation on whether the pack should curate the
   Kimi Desktop set the way `config/codex-catalog.txt` curates Codex.
3. If no budget is detected: the method used, so a future reader knows what was
   actually tested and does not repeat it.
4. `assessment.md` OQ-3 updated from open to the measured verdict.

## Out of scope

- Curating the skill set. If a budget is found, that is a separate change with
  its own review — dropping skills is a user-visible behaviour change.

## Ordering

Independent. May run at any point; touches no file another change edits.

## Note on a negative result

"No budget detected" is a valid and useful outcome, provided criterion 3 records
the method. An untested assumption of safety is what this change exists to
replace.
