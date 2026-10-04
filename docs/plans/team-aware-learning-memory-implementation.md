# Team-aware learning and memory — implementation plan

This plan replaces PRs 4–14 of the session plan (`yes-fix-it-in-logical-mccarthy.md`). It builds them on `docs/design/team-aware-learning-memory.md`. PRs 1–3 (Codex hooks, the 1.9.0 pin refresh, kbd-open into the repo) are merged and are not repeated.

## Rules that apply to every PR

- **Worktrees.** Each PR gets its own worktree and branch off `origin/main`.
- **Gates.** Implement the whole PR first, then run its integration gate. Only integration evidence counts.
- **Cargo.** One cargo build on the machine at a time.
- **Generated files.** Regenerate them (adapters, distribution, Codex plugin, services manifest); never hand-edit them.
- **Absence.** Every hook exits 0 and prints nothing when stores, pk, or teams are absent.
- **Merging.** The user merges. Agents open PRs and report.

## Ordered PR list

### PR A1 — pk: score every entry in a scope

- **Repo:** prometheus-knowledge-rs
- **Files:** `crates/pk-librarian/src/context.rs`, `crates/pk-store/src/scope.rs`, `crates/pk-cli/tests/context.rs`
- **Design section:** 8 (U1), 4
- **Depends on:** —
- **Integration gate:** `cargo test -p pk-cli --test context`. Seed 200 entries in one scope, with the matching entry sorting last by id. `pk context` returns it.

### PR A2 — pk: worker project/team/role and snapshot commit

- **Repo:** prometheus-knowledge-rs
- **Files:** `crates/pk-learning-worker/src/main.rs` (`LearningJob`, `enqueue_memory`, `process_job`), `crates/pk-learning-worker/tests/worker.rs`
- **Design section:** 6, 8 (U2)
- **Depends on:** A1
- **Integration gate:** `cargo test -p pk-learning-worker --test worker`. A job carrying `projectId`, `teamId` and `roleId` produces a surreal-memory outbox op with `user_id = projectId` and `agent_id = <team>/<role>`, and `pk context` returns the session entry.

### PR A3 — pk: tag filter and settable type

- **Repo:** prometheus-knowledge-rs
- **Files:** `crates/pk-cli/src/context.rs`, `crates/pk-cli/src/ingest.rs`, `crates/pk-store/src/entry.rs`, `crates/pk-cli/tests/tags.rs`
- **Design section:** 2, 8 (U3)
- **Depends on:** A1
- **Integration gate:** `cargo test -p pk-cli --test tags`. Entries ingested with `--type Lesson --tag role:a` and `--tag role:b`. `pk context --tag role:a` returns only the first.

### PR A4 — surreal-memory: lean search and filterable metadata

- **Repo:** surreal-memory-server
- **Files:** `src/api/search.rs`, `src/api/memory.rs`, `src/store/memory.rs`, `tests/api_search.rs`
- **Design section:** 8 (U4, U5, U6), 2
- **Depends on:** —
- **Integration gate:** `cargo test --test api_search` against a real SurrealDB 3.3.0:
  - search responses carry no embeddings by default;
  - a `categories` filter returns only matching records;
  - the v2 re-key operation moves `agent_id = null` records to `@project`.

### PR A5 — pin bump to v1.10.0 in both skills repos

- **Repo:** prometheus-skill-system and prometheus-skills-mini
- **Files:**
  - skill-pack: `tools/prometheus-knowledge`, `tools/surreal-memory-server` gitlinks, `skill-system.json`, `tools/prometheus-cli/Cargo.toml`, `tools/forge-rs/Cargo.toml`, `config/release-version-matrix.json`, installation docs;
  - mini: `versions.toml`, the same gitlinks.
- **Design section:** 8
- **Depends on:** A1, A2, A3, A4
- **Integration gate:** `bash scripts/install-binaries.sh` from a clean worktree produces `pk 1.10.0` and `surreal-memory-server 1.10.0` with no downgrade. The `skill-system.js` import check passes. The mini versions check passes.

### PR B1 — identity resolver and project-id unification

- **Repo:** prometheus-skill-system
- **Files:**
  - new: `shared/scripts/lib/project_id.py`, `shared/scripts/lib/project-id.sh`, `shared/scripts/lib/agent_identity.py`;
  - callers: `shared/scripts/memory-bridge.sh`, `shared/scripts/kbd-memory-recall.sh`, `shared/scripts/evaluate-session.sh`, `shared/scripts/enqueue-learning-job.py`;
  - test: `shared/scripts/tests/test-identity.sh`.
- **Design section:** 1
- **Depends on:** A5
- **Integration gate:** `bash shared/scripts/tests/test-identity.sh`. It covers:
  - the four project-id precedence levels in a real git repo with two worktrees (same id);
  - a SubagentStart payload with `agent_type: prometheus-skill-pack:backend-dev` resolves to role `backend-dev`;
  - a Codex payload with `backend_dev` resolves to `backend-dev`;
  - an unknown name falls back to `owns` glob matching.

### PR B2 — SubagentStop matcher namespacing

- **Repo:** prometheus-skill-system
- **Files:** `hooks/hook-contract.json`, regenerated `hooks/hooks.json` and `hooks/codex-hooks.json`, `agents/iterative-evolver-*.md` (renames), `scripts/tests/hook-dispatch.test.mjs`
- **Design section:** 1
- **Depends on:** B1
- **Integration gate:**
  - `npm run validate:harness-adapters` and `node --test scripts/tests/hook-dispatch.test.mjs`;
  - a SubagentStop for a team role named `planner` does not fire the iterative-evolver planner hook;
  - `iterative-evolver-planner` still does.

### PR B3 — envelope write library

- **Repo:** prometheus-skill-system
- **Files:** `shared/scripts/lib/learning_write.py`, `shared/schemas/learning-envelope.schema.json` (from the design PR), `shared/scripts/memory-bridge.sh`, `shared/scripts/tests/test-learning-write.sh`
- **Design section:** 2, 6
- **Depends on:** B1
- **Integration gate:** `bash shared/scripts/tests/test-learning-write.sh` against a live surreal-memory and real pk:
  - each visibility level is stored under its `agent_id`;
  - every stored envelope validates against the schema;
  - a repeated write with the same `contentHash` stores once;
  - with surreal-memory stopped, the op lands in the outbox and the script exits 0.

### PR B4 — per-agent recall library and KBD memory loop

- **Repo:** prometheus-skill-system
- **Files:**
  - `shared/scripts/lib/learning_recall.py`;
  - rewritten `shared/scripts/kbd-memory-recall.sh`;
  - `shared/scripts/memory-writeback.sh`;
  - `skills/process/kbd-process-orchestrator/prompts/reflect.md`;
  - `shared/scripts/kbd-stage-writeback.sh`;
  - kbd-* stage SKILL.md files;
  - `shared/scripts/tests/test-kbd-memory-loop.sh`.
- **Design section:** 4, 6
- **Depends on:** B3
- **Integration gate:** `bash shared/scripts/tests/test-kbd-memory-loop.sh`:
  1. write a reflection with a unique token;
  2. run the worker's `run-once`;
  3. run `kbd-next-phase`;
  4. recall for the assess stage.

  `prior-context.md` cites the token from surreal-memory and from pk, it stays within budget, and the delivery log records its bytes.

### PR B5 — SubagentStart delivery hook (alpha gate)

- **Repo:** prometheus-skill-system
- **Files:** `shared/scripts/subagentstart-learning.sh`, `hooks/hook-contract.json` (group `subagentstart-learning`, matcher `*`, timeout 5), regenerated hooks, `shared/scripts/tests/test-subagent-delivery.sh`
- **Design section:** 5, 4
- **Depends on:** B4, B2
- **Integration gate:** the **alpha** gate, described under "Per-agent integration gates" below. It runs in Claude Code and in Codex.

### PR B6 — file-memory tier reduction

- **Repo:** prometheus-skill-system
- **Files:** `scripts/memory-index-partition.py`, `skills/process/agent-team-creator/runtime/src/export-claude.mts` (`memory: local`), `skills/process/agent-team-creator/runtime/src/export-codex.mts` (`generate_memories=false`, `_` names), `docs/guide/memory-tiers.md`
- **Design section:** 5 (file tier), 10
- **Depends on:** B5
- **Integration gate:**
  - after partitioning a copy of a 14 KB index, `MEMORY.md` is ≤ 4 KB and every removed line is reachable through SubagentStart recall for its role;
  - exported Codex agents have names matching `^[a-z0-9_]+$`;
  - `codex exec` accepts them.

### PR B7 — cross-agent awareness: routing, digest, lead view (beta gate)

- **Repo:** prometheus-skill-system
- **Files:** `shared/scripts/lib/learning_route.py`, `shared/scripts/lib/learning_write.py`, `shared/scripts/lib/learning_recall.py`, `shared/scripts/tests/test-team-awareness.sh`
- **Design section:** 7
- **Depends on:** B5
- **Integration gate:** the **beta** gate, described under "Per-agent integration gates" below. It runs in both harnesses.

### PR C1 — user/global scopes and promotion

- **Repo:** prometheus-knowledge-rs and prometheus-skill-system
- **Files:**
  - pk: `crates/pk-learning-worker/src/promotion.rs`, `crates/pk-cli/src/candidates.rs`;
  - skill-pack: `memory-writeback.sh` marker routing, `kbd-open.sh` candidates section, `skills/process/kbd-reflect/SKILL.md`.
- **Design section:** 3
- **Depends on:** B7
- **Integration gate:**
  - `cargo test -p pk-learning-worker --test promotion`: the same lesson in two projects yields one candidate with 2 evidence entries;
  - `pk candidates accept` writes the pk shared snapshot and queues a `@global` op;
  - a third project's subagent receives the lesson under its global quota.

### PR C2 — Feynman gap routing

- **Repo:** prometheus-skill-system
- **Files:** `shared/scripts/karpathy-hook-dispatch.sh`, `shared/scripts/lib/learning_recall.py` (gap flag), `skills/learn/learn-goal/SKILL.md`, `skills/learn/feynman-loop/SKILL.md`, `shared/scripts/tests/test-prompt-gap.sh`
- **Design section:** 4
- **Depends on:** B4
- **Integration gate:** `bash shared/scripts/tests/test-prompt-gap.sh` with real pk:
  - a problem prompt with an empty KB emits one gap line, and a repeat emits none;
  - a gap recorded inside a subagent carries its role in `gaps.jsonl`;
  - after a covering ingest, no gap line is emitted.

### PR C3 — skill discovery from conversations

- **Repo:** prometheus-knowledge-rs and prometheus-skill-system
- **Files:**
  - pk: `crates/pk-learning-worker/src/transcript.rs`, `crates/pk-learning-worker/src/skill_candidates.rs`;
  - skill-pack: `kbd-open.sh`, the reflect template `## Codify as Skill?`, `propose-skill-update.sh` header.
- **Design section:** 6
- **Depends on:** C1
- **Integration gate:**
  - `cargo test -p pk-learning-worker --test skill_discovery`: three similar report transcripts across two projects yield one new-skill candidate, and a `Skill(kbd-plan)` followed by corrections yields one update candidate attributed to the role that ran it;
  - `kbd-open` lists both.

### PR C4 — agent teams across repos with issue-based requests

- **Repo:** prometheus-skill-system
- **Files:** `skills/process/agent-team-creator/schemas/team.schema.json` (`card`), `runtime/src/registry.mts`, `runtime/src/cli.mts`, `runtime/test-src/teams.integration.mts`
- **Design section:** 7 (lead view as intake), 1 (role ids)
- **Depends on:** B7
- **Integration gate:** `teams.integration.mts` with real git, pk and gh against `PROMETHEUS_TEAM_TEST_REPO`:
  - a same-repo request creates an intake task;
  - a cross-repo request creates a labelled issue that `team-intake` imports exactly once;
  - the intake role's next SubagentStart includes the request digest line.

### PR D1 — prometheus-skills-mini file-tier port

- **Repo:** prometheus-skills-mini
- **Files:** `skills/shared/learning-envelope.schema.json`, `scripts/agent-identity.mjs`, `scripts/subagentstart-learning.mjs`, `scripts/kbd-memory-recall.mjs`, `skills/carried-payload.test.mjs`
- **Design section:** 9
- **Depends on:** B7
- **Integration gate:** `node --test skills/carried-payload.test.mjs`:
  - the carried files match the skill-pack copies byte for byte;
  - the hook exits 0 and prints nothing with an empty HOME;
  - a fixture team's role receives only its file-tier lessons.

### PR D2 — prometheus-skills-mini hook timeouts in seconds

- **Repo:** prometheus-skills-mini
- **Files:** `hooks/hooks.json`, `hooks/codex-hooks.json`, their generator, `scripts/tests/hooks.test.mjs`
- **Design section:** 5 (hook safety)
- **Depends on:** — (mirror of skill-pack PR #124)
- **Integration gate:** `node --test scripts/tests/hooks.test.mjs`. Every hook timeout is 1–600 seconds and is identical across both harnesses.

### PR D3 — CLAUDE.md memory chain and Cortex mirror

- **Repo:** prometheus-skill-system
- **Files:** `CLAUDE.md` (memory section), `AGENTS.md`, `shared/scripts/lib/learning_write.py` (optional Cortex mirror), `docs/guide/memory-tiers.md`
- **Design section:** 5 (file tier), 6 (Cortex mirror)
- **Depends on:** B6
- **Integration gate:**
  - with Cortex installed, one attributed write appears in `cortex_recall` filtered by project and role tag;
  - with Cortex absent, the write path exits 0 with no warning;
  - the CLAUDE.md memory instructions name the per-role delivery and no longer tell subagents to read the full index.

## Order

```
A1 → A2, A3 ; A4 (parallel) → A5 → B1 → B2, B3 → B4 → B5 (alpha) → B6, B7 (beta)
  → C1 → C3 ; C2 after B4 ; C4 after B7 ; D1 after B7 ; D2 any time ; D3 after B6
```

The Rust PRs (A1–A4, and the pk halves of C1 and C3) never build concurrently.

## Per-agent integration gates

Both gates use a fixture team `tlm-fixture` with two roles:
- `api-dev`, which owns `src/api/**`;
- `ui-dev`, which owns `src/ui/**`.

The fixture runs in a scratch git repo, with a scratch HOME and CODEX_HOME. The real plugin generation is never switched. The gates run once in **Claude Code** (`claude -p` with the pack loaded via `--plugin-dir`) and once in **Codex** (`codex exec --dangerously-bypass-hook-trust < /dev/null`, with the project trusted at its canonical path).

### Alpha gate — targeted delivery (PR B5)

**Setup.** Before cycle 1, the write library stores:
- a private `api-dev` lesson with token `ALPHA-API`;
- a private `ui-dev` lesson with token `ALPHA-UI`;
- 30 KB of unrelated project-scope lessons, as a flood control.

**Run.** The main thread spawns `api-dev` and `ui-dev`. Each subagent is asked to echo any token it was given.

**Assertions:**
- `api-dev` reports `ALPHA-API` and not `ALPHA-UI`; `ui-dev` reports the reverse. This gives 0 leaks.
- The delivery log shows SubagentStart bytes ≤ 8,000 characters for Claude Code and ≤ 2,000 tokens (estimated as ≤ 7,000 characters) for Codex.
- The total per-subagent learning bytes are ≤ 12 KB.
- **Codex is UNVERIFIABLE when the project layer does not load.** In that case, Codex runs the fallback path instead: role capture from the `spawn_agent` PreToolUse arguments, correlated by `agent_id`. The gate records which path it exercised and never reports a pass for a path it did not run.

### Beta gate — attribution and awareness across cycles (PR B7)

**Cycle 1.** `api-dev` edits `src/api/handler.ts` and `src/ui/client.ts`. Its SubagentStop writes:
- a private lesson `BETA-PRIV`, with paths `src/api/handler.ts`;
- a routed lesson `BETA-ROUTED`, with paths `src/ui/client.ts`.

**Cycle 2.** The main thread spawns both roles again.

**Assertions:**
- `api-dev` receives `BETA-PRIV`. Its own lessons come back to it.
- `ui-dev` receives `BETA-ROUTED`, through path routing, and only the digest line for `BETA-PRIV`, not its text.
- The main thread, as lead, receives the digest lines and no private text.
- Every stored record validates against the envelope schema, and its `agent_id` matches the design table.
- Bytes per agent are reported per channel, compared against the baseline (14 KB Claude, 10.8 KB Codex), and the reduction is stated.
