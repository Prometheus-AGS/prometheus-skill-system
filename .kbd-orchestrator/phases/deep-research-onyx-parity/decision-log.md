# Decision log — deep-research-onyx-parity

### 2026-09-08 — Analyze decisions (kbd-analyze, after two adversarial rounds)

| Decision | Verdict | Status | Provenance |
|---|---|---|---|
| D-01 Concurrency primitives | `tokio` Semaphore + JoinSet + time::timeout, already a dependency; no new crate | resolved | Tier 1: Cargo.toml:25 |
| D-02 Cap enforcement | In code, not prompt. Onyx states ≤3 four times in prose with no code cap; we enforce structurally | resolved | Tier 1: orchestration_layer.py:88,116,181,197 |
| D-03 Dispatch order | Harness subagents first, process workers second; native/UAR rejected | resolved | assessment V2 |
| D-03a Strategy A provisional | Parallel in-session dispatch is NOT established in-repo; gated by a smoke test in the first thread change | resolved | adversarial round 2 |
| D-04 Workspace split | Deferred, not a precondition | resolved | Tier 1: no new dep needed |
| D-05 Merge key | Canonical URL + existing content-addressed claim hash | resolved | change-rah-010 |
| D-06 Reflection artifact | Durable reflections.md, not an ephemeral ack | resolved | Tier 1: dr_mock_tools.py:112 |
| D-07 Budgets | Onyx constants as starting values, all source-verified, tunable per depth | resolved | Tier 1: dr_loop.py:80,85,97,100; research_agent.py:88,91,93 |
| D-08 Onyx prompt text | Reimplement mechanisms; no verbatim prompt reuse without a licence check (NOASSERTION) | resolved | Tier 1: GitHub licence field |
| D-09 D-A/D-B ownership | Fix in this phase; D-A blocks dispatch strategy B | resolved | assessment §3 |
| D-10 G5 benchmark | Adopt deep_research_bench criteria/prompts/queries, Apache-2.0, pinned 469cce54; build FACT + verified-claim ratio | resolved | adversarial round 1 |
| D-11 G3 report split | Build; no adoptable alternative and the invariant is specific to our claim ids | resolved | adversarial round 1 |
| D-11b D-B is a C-01 event | Regeneration with byte-identical proof + drift validators in the same change | resolved | adversarial round 2 |
| D-12 C-03 on D-B | Not triggered by regeneration alone; triggered by a version bump or manifest edit | resolved | assess handoff deferral |

Open for spec: depth-ceiling enforcement point, per-depth budget defaults, dossier token cap (10K vs ~5K), bench composition and whether Onyx is run locally, `--max-parallel 1` scope.
