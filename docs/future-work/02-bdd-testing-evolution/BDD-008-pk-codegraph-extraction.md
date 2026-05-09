---
id: BDD-008
title: pk-codegraph extraction (ts-morph + Surreal)
status: ready
priority: P0
estimated_effort: 1-2w
agent_role: rust-codegraph
depends_on: []
unblocks: [BDD-009, BDD-010, BDD-013]
related: [SP-019]
created_from_conversation_turn: 5-6
---

# BDD-008 — pk-codegraph extraction

This is the **foundation for selective test execution and test-impact analysis**. Without it, BDD-010 (impact-set hash test runner) is impossible, and BDD-013 (user-story-to-feature contract) is much harder. Plan for 1-2 weeks of focused work.

## Problem

There is no live, queryable representation of code dependencies in the SSR project. Static documents like `react-architecture.md` describe patterns abstractly. CLAUDE.md rules exist but are not data. Nothing answers questions like "if `useAcquisitions` changes, which scenarios in tests/features/ are affected?"

For selective test execution and test-impact analysis, that question must be answerable. It currently cannot be.

## Evidence

1. Read `react-architecture.md` and `signalr-zustand-architecture.md` — note the second is stale (no SignalR; it's SSE). Documentation drifts; data does not.
2. There is no codegraph tool, no extraction step, no Surreal entries describing imports/calls/defines edges.

## Why it matters

The downstream value:
- **BDD-010 (impact-set hash):** "Which files transitively contribute to this scenario's behavior?" — answered by graph traversal.
- **BDD-013 (story-to-feature contract):** "Which features tag-link to this OpenSpec change-id?" — answered by indexed lookup.
- **Refactoring safety:** "If I rename this hook, which tests reference it via testids that components controlled by this hook produce?" — answered by combining static + runtime mapping (BDD-009 adds the runtime layer).

P0 because everything else in this category waits for it.

## Proposed fix

A new Rust crate **`pk-codegraph`** in the `prometheus-knowledge` workspace, with a Node-based extractor companion:

**Architecture:**

```
[ts-morph extractor (Node)]      [pk-codegraph (Rust)]
   walks src/, tests/         →     ingests JSON
   emits JSON of nodes/edges        persists to Surreal
                                    exposes MCP tools
```

**Node types in the graph:**

- `File` — every .ts, .tsx, .feature, .steps.ts file.
- `Component` — React components (function or class).
- `Hook` — custom hooks (functions returning state/effects).
- `Store` — Zustand stores.
- `ApiFn` — exported functions in `src/lib/api/`.
- `Type` — TypeScript types/interfaces.
- `TestId` — `data-testid` attribute values (with the component that defines them).
- `FeatureFile` — Cucumber `.feature` file.
- `Scenario` — single scenario within a feature file.
- `StepDef` — step definition pattern in `.steps.ts`.

**Edge types:**

- `imports` — File → File.
- `calls` — Function → Function.
- `defines` — File → Component/Hook/Store/etc.
- `references_testid` — StepDef → TestId.
- `exercises_scenario` — Scenario → (TestIds, ApiFns) — populated by BDD-009 from runtime traces.

**Per-commit namespacing:**

Each commit's graph lives in a separate Surreal namespace keyed by the commit SHA: `prometheus.codegraph_<sha>`. This avoids mutating-graph-during-PR-review anti-patterns. Stale namespaces are pruned after 30 days.

**Extractor pipeline:**

1. CI step on every push to a feature branch: extract codegraph, ingest into Surreal at `codegraph_<sha>`.
2. CI step on PR open/update: extract codegraph for the PR HEAD; compare against the merge-base SHA's graph to compute the "affected files" set.

**MCP tools exposed by `pk-codegraph`:**

- `codegraph_extract` — trigger an extraction run.
- `codegraph_query_affected` — given a set of changed files, return the transitive set of nodes affected.
- `codegraph_find_tests_for_files` — given files, return scenarios that exercise any of them (requires runtime data from BDD-009 for full fidelity; falls back to static heuristics).

## Trade-offs and risks

- **Maintenance.** The graph must stay current. Per-commit namespacing addresses correctness during PR review but adds Surreal storage overhead. 30-day pruning keeps it bounded.
- **ts-morph performance.** On a large monorepo, full extraction takes 20-60s. Acceptable for CI; not for every keystroke. Watch-mode optimization (incremental extraction) is a future improvement.
- **Test-to-code mapping is inherently imperfect from static analysis alone.** The static graph gives you "this step references this testid → testid is defined by this component → component imports these hooks → hooks call these api-fns." That's good but indirect. Runtime mapping (BDD-009) adds the actual "scenario X executed these files" layer; the two combined give high fidelity.
- **Rust crate adds maintenance burden.** True. The alternative (Node-only) loses Surreal-Rust integration and forces a different storage path. The crate is worth it.

## Acceptance criteria

- [ ] `pk-codegraph` crate exists in `prometheus-knowledge/` workspace, builds with `cargo build`.
- [ ] Node extractor at `pk-codegraph/extractor-node/` walks src/ and tests/, emits JSON with full node and edge set.
- [ ] Rust crate ingests JSON, persists to Surreal at `codegraph_<sha>` namespace.
- [ ] MCP tools `codegraph_extract`, `codegraph_query_affected`, `codegraph_find_tests_for_files` registered.
- [ ] CI workflow runs extraction on push and on PR.
- [ ] On a sample PR (synthetic) that changes one hook, `codegraph_query_affected` returns the expected set of components and step refs.
- [ ] Performance: full extraction on SSR-sized repo completes in <60s.
- [ ] 30-day pruning policy implemented.

## Implementation steps

1. Scaffold the Rust crate with workspace integration.
2. Define the Surreal schema for codegraph nodes and edges.
3. Build the Node extractor using ts-morph. Emit JSON.
4. Implement the Rust ingester (read JSON, persist to Surreal).
5. Implement the MCP tool surface.
6. Add CI workflow.
7. Test on synthetic small repo first; then on SSR.
8. Implement pruning.
9. Document in `pk-codegraph/README.md`.

## Dependencies

None hard. Recommended after SP-008 so the codegraph store is per-project from day one.

## Open questions

- **Watch mode for local development.** Re-extraction on every save is too expensive. Incremental extraction (recompute only changed files' subgraph) is a follow-up; not blocking the first version.
- **Cross-project codegraph.** If multiple projects share the same `prometheus-knowledge` install, do their codegraphs co-exist? Yes, via project-root scoping in the Surreal namespace name (`codegraph_<project>_<sha>`).
- **Trace ingestion details belong in BDD-009.** This task is static extraction only; runtime mapping is the next task.
