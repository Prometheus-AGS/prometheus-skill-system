# change-ldd-04-cache-identity-accounting

Title: Honor embedding model identity and consistent query-cache accounting
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: surreal-memory
Gaps: G4, G5
Depends on: none (start after the change-ldd-03-published-memory-pins task 1 proposal as a priority edge; approval wait does not block independent cache source work)
Reuse: cand-003, cand-004
scope:
  - crates/surreal-memory/src/embeddings/mod.rs
  - crates/surreal-memory/src/embeddings/openai.rs
  - crates/surreal-memory/src/embeddings/cohere.rs
  - crates/surreal-memory/src/embeddings/candle.rs
  - crates/surreal-memory/src/palace/embedding.rs
  - crates/surreal-memory/src/storage/surreal.rs
  - src/executor.rs
  - executors/mlx/Sources/SurrealMemoryMLXExecutorCore/Protocol.swift
  - executors/mlx/Sources/SurrealMemoryMLXExecutor/main.swift
  - src/operations.rs
  - openapi/surreal-memory-v2.openapi.json
  - docs/query-embedding-cache.md

## Required behavior

1. Honor the previous phase normative key requirement: provider/model/configuration namespace plus dimensions and the existing normalized query. The current cache is per-instance, so document this as contract hardening, not a reproduced cross-model leak. Keep the namespace immutable for the owning cache lifetime; model/config replacement must construct a new identity/cache.

2. Expose a backward-compatible optional cache namespace on EmbeddingService, defaulting to unknown. Built-in local, OpenAI, Cohere, FastEmbed and supervised-executor implementations report a stable nonsecret provider/model/configuration identity; supervised identity must describe the actual worker model. Unknown/dynamically unidentifiable providers bypass cache instead of using dimensions as identity. Preserve the public MemoryStorage trait.

3. Retain quick_cache 0.6.21 and existing normalization/capacity policy. A miss increments exactly once when an embedding attempt begins, including an attempt that fails or is subsequently cancelled. A hit increments only after successful reuse/coalesced success without invoking the producer. Capacity=0 and unknown namespace cause one miss per attempted query embedding and zero hits. Failed values are never cached. Coalesced retries may cause multiple actual attempts; document counters rather than claiming hits+misses always equals successful requests.

4. Preserve the write-path invariant: create/update/duplicate detection must not read, populate or increment query-cache counters. Preserve existing stats field names/types; update their descriptions in OpenAPI/docs without a speculative new telemetry framework. Replace .surreal().ok() at the durable-operation stats path with explicit supported-storage enforcement/error propagation; do not label the downcast failure a transport error.

5. Keep dependency changes minimal. Change 03 owns approved final release/version decisions including this cache work; do not retag an existing release or leave this work out of the final pin graph.

## Binding execution contract

These are specifications, not acceptance results. All tasks start pending. Production tasks have no per-task test command; the parent phase completes every production edit before test authoring/execution. Change `change-ldd-12-integration-rollout` owns the consolidated final local gate and C-01 generated reconciliation. No generated certification, independent review or release success is implied by finishing a source task.

Use worktrees under `/Users/gqadonis/Projects/prometheus/worktrees/` from the respective repository's `origin/main`; incorporate prerequisite source changes without requiring an early push/merge. Preserve existing main-checkout dirt. Never modify `worktrees/deploy-main`, `deploy/main`, installed plugin caches or real home state during implementation/tests. Owner merges all PRs and explicitly approves versions.toml, tags and real Codex/Claude changes. Check machine-wide Cargo/rustc contention before each eventual build; only local full integration evidence is accepted. Exit 2 is BLOCKED.

Adversarial review is pending until the completed-production boundary under canonical AGENTS.md/CLAUDE.md. No skill suggestion authorizes earlier review/test execution. Existing historical gate claims do not certify this phase. Native task registration into the canonical runtime belongs to Plan; task IDs below are stable authored IDs.

## Recalled lessons

Previous cache specification required model ID; reflection revealed dimensions-only implementation. Recalled write-duplicate detection polluted the query cache: preserve the repaired separation. Real source shows .surreal() is a downcast, not network I/O.

See `verification.md` for acceptance boundaries and `tasks.json` for the explicit pending task list.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-04-cache-identity-accounting` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
