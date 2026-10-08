# Analysis: phase-team-learning-hardening

- Analyzed: 2026-10-04, from `assessment.md` (assess handoff) and `prior-context.md`.
- Mode: **stack specified**. The stack is the existing repositories: bash 3.2 and Python 3 hook scripts, Node `.mts` for agent-team-creator, Rust for surreal-memory-server and prometheus-cli. No stack discovery was needed.
- Research budget used: tier 1, 1 query (GitHub repo search); tier 2, 0 (local source and the installed packages answered every API question); tier 3, 3 (npm, crates.io ×2). The rest is local code inspection of `surreal-memory-server@777cf72` (origin/main `0af8ae1`, v1.10.0), Cortex 2.0.3 in the plugin cache (read only), and a read-only `codex debug prompt-input` probe on codex-cli 0.158.0. About 15 minutes, inside the 20-minute cap.

## Recalled lessons that bear on these choices

- "For every external integration, run at least one test against the real service". This is why G3a adopts the real Cortex 2.0.3 server, run against a scratch `CORTEX_DATA_DIR`, rather than improving the stub.
- "Avoid reentrant lock-taking CLIs and default fallbacks". G1c's procedure must fail loud; that is decided below.
- "Generated Format Emitters and Validators Must Stay Paired". G3c's helper must regenerate with the repository's authoritative emitters, never hand-merge, and then run the paired validators (`check:distribution`, `validate:harness-adapters`, `validate:codex`).
- "`kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger". G2c changes the driver itself.

## Knowledge gaps (from `prior-context.md`)

The same seven engineering gaps listed in `assessment.md` (G1a, G1b, G1c, G2a, G2b, G3a, G2c). Each one is resolved by a decision below. These are available as optional study topics, and none has been run:
- `/learn-goal "quick_cache LRU semantics"` (G2b);
- `/learn-goal "Cortex 2.0.3 storage and recall"` (G3a);
- `/learn-goal "git custom merge drivers"` (G3c).

## Findings per gap

### G2b: repeated query embedding (answers the assessment's main open question)

**Confirmed.** The server embeds every search query again, with no cache.
- The route is `src/api/search.rs`. Its `SearchBody` has `query`, `user_id`, `agent_id`, `session_id`, `limit` and the two weights. There is **no precomputed-embedding field and no multi-scope form**.
- The handler calls `storage.hybrid_search_memories` (`crates/surreal-memory/src/storage/surreal.rs:2521`). That runs `search_memories`, which calls `self.embed_text(query)`, which calls `embedding_service.embed(text)`, with no cache layer in between.
- The client sends its queries one after another with identical text (`learning_recall.py:223-230`, `:286-300`). A non-lead subagent sends **4 embedding searches** (role, project, user, global) plus 1 `recent` listing; the lead or main thread sends 5. Each search embeds again on the supervised executor. *Not measured:* the latency of a single embedding under load against the 2.8 s deadline. The plan must measure it first, because if one embedding alone exceeds the deadline, no amount of de-duplication helps.

Options:

| Option | Where | Effect | Cost |
|---|---|---|---|
| A. Query-embedding LRU in the server's embedding service, keyed on (model id, normalised text), **query path only** (`search_memories`, never the write path) | surreal-memory-server | 4 or 5 embeddings become 1 for every caller, including the MCP `hybrid_search_memories` and pk | Small Rust change and a server release (1.10.1). Cross-repo. |
| B. Run the client's 5 requests concurrently | skill pack, `learning_recall.py` | Wall time is bounded by the slowest request, if the executor parallelises. It is a supervised child, so it likely serialises. | Small, but probably does not fix the load case. |
| C. Answer the `user` and `global` scopes (top-N shared, `TOP_SHARED`) with lexical `GET /api/v1/memory` (mode `list`) instead of search | skill pack | 4 embeddings become 2 (non-lead) with no server change. It **degrades recall quality** for shared scopes, against G3b, and fetches the whole scope, which the server code warns against. | Recall quality for shared scopes drops to recency plus lexical. |
| D. Multi-scope search API: one embedding, N scope filters | surreal-memory-server and client | 1 embedding and 1 round trip | A new API surface, the largest change. |

**Decision: A, measured first; C rejected.**
- A fixes the cause for every caller. `quick_cache` is **already in surreal-memory's `Cargo.lock`** (0.6.21, transitive through `surrealdb-core`). Pin `quick_cache = "0.6"` to that version so no second copy is added; 0.7.0 is the current crates.io release per `cargo search`.
- Spec must confirm the async-miss behaviour (`get_value_or_guard_async`, so concurrent identical misses embed once) and a size bound.
- C is rejected because it trades the G3b recall-quality goal for latency. It comes back only if the measurement shows A is insufficient.
- B is rejected: it is unproven against a serialising executor.
- D is deferred: more API surface than this phase needs.

### G3a: Cortex mirror against real Cortex

- Real Cortex 2.0.3 (MIT, deps `@xenova/transformers`, `sql.js`, `zod`) honours `CORTEX_DATA_DIR` (`src/config.ts:108`). A test can therefore start the **real** `dist/mcp-server.js` from the installed package against a scratch data directory, without running `/cortex:setup` on the operator's `~/.cortex`.
- The embedding model (`nomic-ai`) is already cached inside the package's `node_modules/@xenova/transformers/.cache`, so the test needs no network.
- **Risk:** if the model cache is missing, xenova writes it into the plugin-cache directory. The test must detect a missing model and exit BLOCKED (2), never download into the cache. This respects the never-edit-a-plugin-cache rule.
- Cortex's `cortex_remember` schema is `content`, optional `context`, and `projectId`. There is no tag field, so the role tag stays in `context`, as D3 assumed. The real-service test must assert that recall by `projectId` returns the mirrored lesson, and that the role text in `context` survives.

**Decision:** adopt real Cortex as the integration target. The stub test stays as the "Cortex absent or misbehaving" case.

### G1b: Codex memories

- The probe ran `codex debug prompt-input` (codex-cli 0.158.0, feature `memories` = stable, enabled).
- With `generate_memories=false` and `memory_summary.md` archived, the assembled prompt contains **no** text from `~/.codex/memories/MEMORY.md` and no reference to `memory_summary`. The leftover 180 KB `MEMORY.md` and `raw_memories.md` are therefore inert: Codex reaches them only through the summary.

**Decision.** The installer must do three things:
1. set `[memories] generate_memories = false` in `~/.codex/config.toml`, merging, never clobbering, with a backup;
2. archive an existing `memory_summary.md` to `~/.codex/memories-archive/`;
3. leave the other memory files in place, since they are inert and are the user's data.

The doctor adds a `codex.memories` check: Yellow when `generate_memories` is not `false` or `memory_summary.md` exists, and optional, so it never fails the install. Turning `use_memories` off is not needed.

### G1a: priority-ranked partition

There is nothing to adopt: it is a 211-line script with domain-specific ranking. **Build.** The rank key, highest first:
1. the current phase's project entries (matched by the active phase slug from `position-reminder.txt` or `current-waypoint.json`);
2. `feedback`/GLOBAL lessons;
3. other `project` entries, newest first;
4. `archive-*` entries.

Within a rank, keep file order. Lines are moved by lowest rank first until the index fits the budget. This is the manual reorder done earlier today, made deterministic.

### G1c: versioned cadence refresh procedure

**Decision:** ship it in the delivery-cadence skill as `skills/process/delivery-cadence/procedures/refresh-skill-pack.sh`, which puts it under C-01 regeneration into `dist/`.
- The project-local `.prometheus/cadence/procedures/refresh-skill-pack.sh` becomes a 3-line shim that `exec`s the installed copy, or the profile points at the installed path directly.
- Required behaviour, each one a recorded fix:
  - `set -euo pipefail`;
  - the iteration number is read from `state.json` and never from the cadence CLI inside a checkpoint; a missing or invalid N **exits non-zero** instead of defaulting to 1;
  - `git submodule update` runs after the fast-forward;
  - `launchctl kickstart -k` for surreal-memory.
- The procedure stays repository-specific: deploy worktree path, service labels. So it takes those as profile inputs rather than hard-coding them.
- Repository-only (`scripts/cadence/`) was the alternative. It was rejected because other projects using delivery-cadence hit the same lessons.

### G2a: memory-envelope integration on a scratch server

There is nothing to adopt. **Adapt** the existing scratch-server harness already used by `shared/scripts/tests/test-kbd-memory-loop.sh` and `test-subagent-delivery.sh`: they start a scratch `surreal-memory-server` on a free port with a temporary data directory and tear it down. The `.mts` test either uses that harness from a small Node helper or is launched by a wrapper that exports `SURREAL_MEMORY_URL`. The default of `:23001` is removed: with no URL the suite starts its own server, or exits BLOCKED (2) when the binary is absent.

### G2c: ledger reconciliation

Three layers, all small:
1. `kbd-apply mark-done` prints a warning naming `end-task`, or syncs. **Decision:** make it call `sync_progress` too; one code path is safer than a warning people ignore.
2. A new `kbd-apply reconcile <phase>` subcommand compares each change's `tasks.json`/`tasks.md` done flags with the canonical ledger, lists the drift, and exits 1 when there is drift. With `--repair` it replays `begin-task`/`end-task`.
3. The cadence iteration and `/kbd-reflect` call `reconcile` (read-only) at their boundary.

### G3b: recall quality

- Root cause found in assess: `pk_allowed` (`learning_recall.py:403-404`) lets every **untagged** pk entry through. The "pk knowledge" block is filled with untagged legacy ingests from other projects (KnowMe, avatars, Actix).
- **Decision:** untagged entries are admitted only when their pk scope is `project` for the current project, or they match lexically above a threshold. Add a fixed evaluation fixture (phase goals plus a seeded store) with an acceptance criterion: at least 3 of the top 5 recalled items carry the project or role scope, and none from a foreign project.
- No library: the scoring is ours.

### G3c: rebase-regenerate helper

- Tier 1 found no general tool. `npm-merge-driver` (2.3.6, ISC) is the reference pattern: a git custom merge driver that regenerates a lockfile on conflict. It is npm-lockfile-specific, so its verdict is **reference**.
- **Decision: build** `scripts/rebase-regenerate.sh`. During a conflicted rebase or merge it:
  1. refuses unless **every** conflicted path is a known generated path. Spec must name the single source of that set, deriving it from the generators' declared outputs (the `skill-system.json` target matrix and the harness-adapter generator's output list), not a hand-written glob. Today it covers `dist/**`, the hook bundles, harness manifests, `.agents/plugins/marketplace.json` and `.codex-plugin/plugin.json`;
  2. resolves those paths to either side (the content is discarded). During a rebase, `--ours` is the upstream; during a merge it is the current branch. Then
  3. reruns `generate-harness-adapters.js`, `generate-skill-system-distribution.js` and `build:codex`;
  4. runs the paired validators;
  5. stages the result.

  Real source conflicts still need a human. A git `merge.<name>.driver` registration is optional on top, since registering it touches `.git/config`; the plain script is enough.

## Open questions

1. G2b: measure a single query embedding under load (the executor's latency) before implementing. Option A is a **cross-repo release** (surreal-memory-server 1.10.1), which needs a version bump, a tag and an operator-approved push. Plan should order the server change first and treat C as the in-repo fallback that ships regardless.
2. G1c: should the shipped procedure stay generic enough for other projects in this phase, or ship skill-pack-specific with profile inputs and generalise later? The recommendation is profile inputs now.
3. G3b's threshold for "matches lexically above a threshold" needs a value. Plan should fix it with the evaluation fixture, not guess it.

## Revisions after adversarial review

The review ran harness-native and same-family (no distinct judge model). Verdict PASS: 0 critical, 7 warnings, 1 suggestion (`review/analyze/findings.json`). Changes made:

- **G2b.** The embedding counts are corrected to 4 for a non-lead and 5 for the lead. A latency measurement is now a precondition. C is rejected as conflicting with G3b. The quick_cache pin and the async-miss behaviour are made explicit (see the G2b section above).
- **G1b, durability.** The "inert" conclusion rests on one probe. The doctor's `codex.memories` check re-verifies on every run that `generate_memories=false` and that no `memory_summary.md` exists, so a regrown summary is caught. Spec adds an integration test with a scratch `CODEX_HOME`: config set, summary archived, and `codex debug prompt-input` contains no memories text.
- **G1b, TOML editing.** No TOML library is adopted. The installer is Node (`scripts/install-system.js`). Candidates such as `smol-toml` and `@iarna/toml` re-serialise and drop the operator's comments. Python `tomllib` reads only, and `toml_edit` would need a Rust binary. **Decision:** an idempotent line-level edit inserts or updates the `[memories]` section, with a timestamped backup. The result is re-parsed with a real TOML parser before the file is replaced; on a parse failure the original is restored and the step warns. This is how the operator's file was edited by hand, and it kept its comments.
- **G3c, git-native options.** `git rerere` replays recorded resolutions. It cannot help here, because regenerated content differs on every conflict. A `.gitattributes` `merge=<driver>` registration needs per-clone `.git/config` setup. It stays an optional, documented layer on top of the script. Both are recorded as `reference` only.
- **G3c, side naming.** Corrected: the resolved content is discarded and regenerated, so the side is irrelevant. The `--ours` and `--theirs` semantics during a rebase are named. The source of the generated-path set is assigned to spec.
- **G2a.** The existing harness's start logic is inline bash. It also needs the embedding executor (MLX) and server ≥ 1.10.0. **Decision:** a shell wrapper (`tests/run-memory-envelope.sh`) starts the scratch server using the same steps as `test-kbd-memory-loop.sh`, exports `SURREAL_MEMORY_URL`, and runs the `.mjs`. The `.mts` drops its `:23001` default and exits BLOCKED (2) when the variable is unset. Spec may first extract the shared start and stop steps into `shared/scripts/tests/lib/scratch-surreal.sh`, so the three tests share one copy.
- **build_required evidence.** The shared "tier 1 and 3 searched" sentence overstates the search: 1 tier-1 and 3 tier-3 queries were run, none targeting G1a, G2c or G3b. Those three are repository logic with no plausible external candidate, so searching for them was judged not worthwhile. That is a judgement, not a search result.
- **Constraints for every new script (from the suggestion):**
  - shell scripts that launchd can run, which includes the G1c procedure, must be bash 3.2 compatible (no `mapfile`, no `declare -A`) and are tested with `/bin/bash`;
  - the G1b installer step follows the installer-must-not-abort rule: every failure path is guarded, and a failure is a warning plus a completed install;
  - G3c runs only by hand, so it may fail loud.
