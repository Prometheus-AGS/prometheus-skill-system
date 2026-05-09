---
id: BDD-009
title: pk-codegraph runtime coverage (Playwright trace ingestion)
status: planned
priority: P1
estimated_effort: 1w
agent_role: rust-codegraph
depends_on: [BDD-008]
unblocks: [BDD-010]
related: []
created_from_conversation_turn: 5-6
---

# BDD-009 — pk-codegraph runtime coverage

## Problem

BDD-008's static extraction gives you "this step references this testid" and "this testid is defined by this component." But step `When I click the {string}` is a parameterized template — at static-analysis time you can't know which scenarios it serves. You also can't know which API endpoints fire during a scenario, which network routes matter, or which JS files actually executed. Static analysis bottoms out before reaching the precise scenario-to-code mapping needed for selective test execution.

## Evidence

Run a scenario in SSR with `recordVideo: true` and `trace: 'on'`. Observe the `tests/reports/<scenario>.zip` trace file. Inside is rich JSON — network requests, DOM events, console messages, JS coverage if enabled.

That data is the ground truth for "what did this scenario actually exercise."

## Why it matters

Without runtime coverage:
- BDD-010's impact-set hash is approximate; it may include too many files (over-running) or miss some (correctness gap).
- BDD-008's `codegraph_find_tests_for_files` falls back to static heuristics, which under-cover scenarios with dynamic step parameters.

With runtime coverage:
- Each scenario has a precise list of files and routes it exercised in the most recent passing run.
- The impact-set computation becomes exact for files-that-changed.
- A code change can be mapped to the exact scenarios that need re-running.

## Proposed fix

Extend `pk-codegraph` with a runtime-coverage ingester:

**Trace capture.** Update SSR's Playwright config to enable JS coverage capture during BDD runs:

```typescript
// In tests/support/world.ts (or playwright config)
await page.coverage.startJSCoverage();
// ... scenario runs ...
const coverage = await page.coverage.stopJSCoverage();
```

The trace zip already includes network and DOM events. Add JS coverage when running with `VIDEO=true` (the same flag that enables video recording).

**Trace ingestion.** A new module `pk-codegraph::runtime` (Rust) reads trace zips:

1. Parses the JSON event streams.
2. Extracts: scenario_id, files-with-coverage, network endpoints hit.
3. Maps endpoints back to API handlers (Next.js routing structure is known statically).
4. Maps API handlers transitively to imported modules (via static graph from BDD-008).
5. Writes `exercises_scenario` edges in Surreal: `Scenario → File`.

**Triggering ingestion.** After every successful video-proof run, a hook ingests the new traces. Old traces are kept for 30 days; old `exercises_scenario` edges expire with them (or get refreshed by re-runs).

## Trade-offs and risks

- **Cost: JS coverage capture adds 20-30% per-scenario runtime overhead.** Acceptable for video-proof runs (which are not on the critical PR path); too expensive for `pnpm test:bdd:ui` watch mode. Configurable.
- **Storage: trace zips are 100KB-1MB each.** With 250 scenarios, that's 25MB-250MB per full run. Don't store the raw zips in Surreal; extract the relevant signal (~10-50KB per scenario) and discard.
- **Correctness: coverage is per-test-run.** If a scenario's coverage was last captured in commit X but the scenario hasn't run since, and then code changes in files the scenario *used to* exercise, the impact-set computation may miss it. Mitigation: the `validated_against_commit` field on scenarios (used in BDD-010) requires re-running if the commit drifts too far.
- **Cross-platform coverage isn't free.** Playwright supports JS coverage on Chromium; on Firefox/WebKit support is limited. Default to Chromium for video proof; document the constraint.

## Acceptance criteria

- [ ] Playwright config captures JS coverage during video runs.
- [ ] After a successful video-proof run, traces are ingested into Surreal.
- [ ] `Scenario → File` edges in Surreal represent the runtime-observed coverage.
- [ ] `codegraph_find_tests_for_files` (from BDD-008) uses runtime data when available, static fallback otherwise.
- [ ] Old traces and edges are pruned after 30 days.
- [ ] Performance: ingestion of 250 traces completes in <2 minutes.
- [ ] Smoke test: a synthetic code change in one file → query returns the expected set of exercising scenarios.

## Implementation steps

1. Update Playwright config to enable JS coverage in video mode.
2. Write the trace-zip parser in Rust (use `zip` crate, `serde_json` for events).
3. Implement endpoint → handler mapping (Next.js routing is regular).
4. Write the Surreal edge writer.
5. Hook ingestion into the post-video-proof Stop chain.
6. Test on a real video-proof run.
7. Add pruning.
8. Document.

## Dependencies

BDD-008 (the static graph must exist; runtime adds edges to it).

## Open questions

- Should ingestion run after every scenario or batch at end-of-run? End-of-run is simpler and bulk-write is more efficient.
- Source maps for production-mode coverage. SSR runs in dev for tests, so source mapping is not critical, but coverage data references compiled file paths. Mitigation: ingest as-is, normalize paths via the static graph.
- Should runtime data feed back into the static graph (e.g. discovering `imports` edges that ts-morph missed)? Possibly, but cautiously — runtime data is per-scenario and doesn't generalize as cleanly as static.
