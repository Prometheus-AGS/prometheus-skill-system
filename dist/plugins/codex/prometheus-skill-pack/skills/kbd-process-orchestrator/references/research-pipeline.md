# Research pipeline (shared source of truth)

The tiered procedure both `/kbd-analyze` and the iterative-evolver's Analyze
phase follow. Codifies the global "Research & Reuse" rule as a machine
procedure: search before building, prefer battle-tested libraries over
hand-rolled code, gather evidence before concluding.

## Tiers (run in order; stop early when a tier answers the question)

| Tier | Source | How | Purpose |
|------|--------|-----|---------|
| 1 | GitHub code/repo search | `gh search repos`, `gh search code` (CLI); `mcp__github__search_*` fallback | Existing frameworks, skeletons, 80%-solutions to fork/port/wrap |
| 2 | Library docs | Context7 / docfork MCP (`resolve-library-id` → `get-library-docs`) | Confirm API fit and version constraints for tier-1 candidates |
| 3 | Package registries | `npm view <pkg> --json`, `cargo search`, PyPI JSON API via Bash | Maintenance health: last release, downloads, license, open issues |
| 4 | Broad web | firecrawl_search (primary) / tavily | Stack comparisons, architecture patterns — only when tiers 1–3 are insufficient |

## Budget (hard caps — terminate, don't spin)

Every analysis records a `research_budget` block in `library-candidates.json`:

- `max_queries_per_tier` (default **8**)
- `max_minutes` (default **20**)
- `queries_used` — incremented as you go

On reaching a cap, **stop** and emit what was found with lower confidence and a
note in `analysis.md` that the budget bounded the search. Never loop a tier to
chase completeness past its cap. (Mirrors the proven 2-rejection sycophancy-gate
cap: bounded effort beats unbounded thoroughness.)

## Two modes

- **Stack specified** (project.json / constraint manifest names the stack):
  research candidates *within* that stack, ranked per assessment gap.
- **Stack discovery** (greenfield / unspecified): first a stack-discovery pass
  (tier 4 + tier 1 by problem domain) producing `stack-recommendation.md` with
  2–3 scored options; then candidate research against the recommended stack.
  **Contested choice** = score gap < 15% between the top two options →
  escalate via `pmpo-elicit` (when available) rather than silently picking;
  record the decision and alternatives in `decision-log.md`.

## Evidence format

Each candidate carries an `evidence[]` array of `{tier, source_url, claim}` so
every adopt/adapt/reference/reject verdict is traceable to where it came from.
`build_required[]` records gaps with no adoptable candidate — these flow to the
Plan stage as build tasks (and, in a later phase, to capability gaps).
