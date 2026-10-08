# Plan: Codex hooks, Rust pin refresh, and the memory/learning/teams loop

## Context

Several pieces of the Prometheus self-learning stack exist but do not work as a whole. Verified on `origin/main` (2026-10-03):

- **Codex hooks never fire.** PR #54 (`e097e4e`, 2026-08-10) replaced the hand-written `.codex-plugin/plugin.json` (which had `"hooks": "./hooks/codex-hooks.json"`). The generated Codex package ships no hooks, no `scripts/hook-entry.mjs` and no `shared/`.
- **Rust pins are 40–122 commits stale.** Both skills repos pin surreal-memory-server and prometheus-knowledge-rs to Sept commits. `pk-*` Cargo deps lock to April's `79878b4`. Mini's `versions.toml` disagrees with its own submodule commits. The installed binaries are 1.9.0, so running either installer would downgrade them.
- **KBD memory is write-mostly.**
  - Reflect write-back extracts section names the reflect template never emits, so no lessons reach surreal-memory.
  - Recall reads lifecycle metadata, not lessons.
  - User and global memories are never read.
  - The project id differs between the bridge and the worker.
- **Promotion, Feynman routing, skill discovery and cross-team requests exist only as docs or dead code.** The learning worker reads only the final assistant message of a transcript.

Decisions made with the user:
- **Pins:** push surreal-memory `777cf72` and pin both repos to it, plus pk `2759252`; tag both `v1.9.0`.
- **Promotion:** auto-propose, human confirms; `[GLOBAL]`/`[USER]` markers promote immediately.
- **Team requests:** pk shared-scope team registry. Requests inside the same repo become handoffs; requests to another repo, or ones a rule triggers, become GitHub issues.
- **Delivery:** sequenced PRs, each with an integration gate.

Repo rules apply throughout:
- Implement fully, then run integration gates only.
- One cargo build at a time.
- Regenerate generated files; never hand-edit them.
- Each PR's work happens in its own worktree off `origin/main`. The main checkout has other people's uncommitted edits.

---

## PR 1 — Ship Codex hooks (skill-pack)

Worktree: `../worktrees/codex-hooks` on `fix/codex-plugin-hooks` off `origin/main`.

- In `scripts/generate-skill-system-distribution.js`, move the Claude branch's runtime closure into a `copyHookRuntime(root, hooksSource)` helper used by both platforms. The closure is `copyHookTargets`, `shared/`, `copySignedSkillRuntimeFiles`, `scripts/install-plugin-generation.js`, `scripts/lib`, `package.json` and `config/prometheus-exec-component.json`.
  - Claude keeps writing `hooks/hooks.json`.
  - Codex writes `hooks/codex-hooks.json` into the package as `hooks/hooks.json`. Codex discovers that path by convention, as the existing `prometheus-skill-pack@…:hooks/hooks.json` trust entries in `~/.codex/config.toml` show. No `hooks` key goes in `.codex-plugin/plugin.json`, mirroring the Claude rule.
  - Make `copyHookTargets` read whichever hooks file it is given.
- **Plugin-root variable.** Codex sets `PLUGIN_ROOT`, not `CLAUDE_PLUGIN_ROOT`. If the `codex exec` check below shows `${CLAUDE_PLUGIN_ROOT}` is not substituted in `args`, change the Codex emitter in `scripts/generate-harness-adapters.js` to emit `${PLUGIN_ROOT}`. Then rerun `generate-harness-adapters.js` and `validate:harness-adapters`.
- Regenerate with `npm run build:codex`. Extend `scripts/tests/skill-system-distribution.test.mjs` to assert that the Codex package has `hooks/hooks.json`, every file it references, and no `hooks` manifest key.
- Fix the CLAUDE.md "Codex hooks" paragraph: it should say the generated Codex package carries `hooks/hooks.json` by convention, not through a `plugin.json → hooks` key. Update `docs/codex-plugin.md` to match.
- **Gate:**
  - `npm run check:distribution`, `npm run validate:harness-adapters`, `npm run validate:codex`.
  - `codex plugin marketplace add .`, reinstall `prometheus-skill-pack`, then `codex exec --dangerously-bypass-hook-trust "echo hi"`.
  - Confirm a SessionStart hook log or receipt appears under `${PLUGIN_DATA}`, and that `~/.codex/plugins/cache/prometheus-skill-pack/…/hooks/hooks.json` exists.
  - Write a project memory note recording the root cause.

## PR 2 — Refresh surreal-memory and prometheus-knowledge pins (all 4 repos)

1. **Upstream prep.**
   - surreal-memory-server: push `777cf72` to `main`; the uncommitted wiki files stay local. Tag `v1.9.0` at `777cf72`.
   - prometheus-knowledge-rs: tag `v1.9.0` at `2759252`. Push both tags.
2. **skill-pack** (worktree `bump/rust-tools-1.9.0`):
   - Move the gitlinks `tools/surreal-memory-server` → `777cf72` and `tools/prometheus-knowledge` → `2759252`.
   - Update `skill-system.json` `imports[].commit` to match; `scripts/lib/skill-system.js:148` enforces this.
   - `config/release-version-matrix.json` exemptions: 1.8.0 → 1.9.0.
   - `config/defaults.env` `SURREALDB_VERSION` stays 3.3.0, which matches surreal-memory's new `=3.3.0`.
   - Pin `pk-core`/`pk-librarian`/`pk-store` with `tag = "v1.9.0"` in `tools/prometheus-cli/Cargo.toml` and `tools/forge-rs/Cargo.toml`. Run `cargo update -p pk-core -p pk-librarian -p pk-store` in each, sequentially.
   - Update `docs/guide/19-installation.md` and `site/docs/operations/installation-and-upgrades.md` to expect 1.9.0.
3. **mini** (worktree):
   - Same two gitlinks.
   - In `versions.toml`, fix the surreal-memory line (and the drifted liter-llm line to its actual gitlink `12a2fae`).
   - Update `docs/versions-toml.md` and `TOOL_ANALYSIS.md`.
   - Switch `.gitmodules` pk to the HTTPS URL to match skill-pack.
- **Gate:**
  - skill-pack: `node scripts/lib/skill-system.js` install check (submodule HEAD = import commit), then `cargo check -p prometheus-cli` and `cargo check` in forge-rs, sequentially.
  - Both repos: `bash scripts/install-binaries.sh` from a clean worktree must produce `pk 1.9.0` and `surreal-memory-server 1.9.0`, with no downgrade.
  - mini: its versions check (`versions.toml` = gitlinks).

---

## Capability PRs

Repos:
- **SP** = skill-pack
- **PK** = prometheus-knowledge-rs
- **MINI** = skills-mini

### Shared building blocks

- `shared/scripts/lib/project_id.py` (+ `project-id.sh` wrapper):
  - **What it does:** the one project id resolver. It copies the worker's `project_scope()` (`pk-learning-worker/src/main.rs:1537`): `.prometheus/project.json` `projectId`, otherwise the git toplevel, otherwise `project:<sha256(path)>`.
  - **User scope:** it also returns `userScope` = `PROMETHEUS_USER_ID`, otherwise `user:<sha256(global git email)[:16]>`.
  - **Replaces:** the literal default `MEM_PROJECT=prometheus-skill-pack` (`memory-bridge.sh:16`) and the `kbd-memory-recall.sh` and `evaluate-session.sh` variants.
- **Recall** uses SM `POST /api/v1/search {query,user_id,limit}` (`surreal-memory-server/src/api/search.rs:10`). Reuse `kbd_memory_available` and `kbd_memory_url` from `shared/lib/memory.sh` and the bridge's `mem_add_memory`.
- **Candidate queues** go under `~/.prometheus/`: `promotion-candidates/`, `skill-candidates/`, `knowledge-gaps/gaps.jsonl`. Each has `pending/accepted/rejected` and uses the worker's `atomic_json` / `durable_rename`.

### PR 3 (SP) — kbd-open into the repo

- Port `~/.local/bin/kbd-open` to `shared/scripts/kbd-open.sh` (bash 3.2, always exits 0).
  - Replace the slow LLM `pk focus` call with `pk context … --scope project --scope shared --scope global --max-bytes 3000`.
  - Add silent-when-empty sections for promotion candidates, skill candidates and repeated knowledge gaps. They read the directories with python3, so pk isn't required.
- `hook-contract.json`: `sessionstart-kbd-open` gets `target: shared/scripts/kbd-open.sh`.
- Drop the `$HOME/.local/bin/kbd-open` allowlist and branch in `generate-harness-adapters.js` (~line 119). Regenerate the adapters and dist.
- **Gate:** `validate:harness-adapters`, then `node scripts/hook-entry.mjs --hook sessionstart-kbd-open` with an empty temporary HOME and a temporary KBD project. Expect exit 0, an empty stderr, and the phase header present.

### PR 4 (SP) + PR 5 (PK) + PR 6 (MINI) — KBD memory loop works end to end

**SP changes**
- **Reflect template** (`kbd-process-orchestrator/prompts/reflect.md:107-150`):
  - Lead with `## Delta / ## Root Cause / ## Corrective Actions`, which the sycophancy gate already requires.
  - Keep `## Lessons Learned` with optional `[GLOBAL]`/`[USER]` per-line prefixes.
  - Rename to `## Next Phase Seed`.
  - Add `## Codify as Skill?`.
- **`memory-writeback.sh`:**
  - Extract Delta, Root Cause, Corrective Actions, Lessons Learned and Next Phase Seed (legacy `Next Phase Focus` as a fallback).
  - Dedupe with a per-phase hash.
  - Pipe the payload to `pk ingest --scope project --source kbd-reflect:<phase>`.
  - Call `mem_add_memory` with category `kbd-reflection`.
  - Delete the no-op `pk-ingest-on-reflect` hook.
  - `evaluate-session.sh` uses the bridge instead of its direct REST POST.
- **Rewrite `kbd-memory-recall.sh`** to take `<phase> [stage]`. It queries:
  1. SM `/api/v1/search` for `projectId` and for `global`;
  2. `pk context --format json`;
  3. the previous `reflection.md` and pk session entries.
  
  It writes `prior-context.md` with sections for memory lessons, pk knowledge, the previous reflection, prior phases and knowledge gaps.
- **Deterministic recall:**
  - New `shared/scripts/kbd-stage-recall.sh`. It runs from a Claude-only `PreToolUse` hook with matcher `Skill` for `kbd-(assess|analyze|plan|execute|reflect)`, using the contract's per-group `harnesses` filter.
  - For Codex, `karpathy-hook-dispatch.sh` detects `/kbd-<stage>` prompts.
  - A 10-minute freshness stamp prevents repeat runs.
- **Stage writes:** new `shared/scripts/kbd-stage-writeback.sh` as a builtin `*:after` hook for assess, analyze and plan. It reads the `kbd_stage_handoff_write` summary (`shared/lib/stage-gate.sh:220`) and calls `mem_add_memory`.
- **Stage skills:** every `kbd-*` stage SKILL.md starts with "read `prior-context.md`, cite applicable lessons". Reflect also states "which recalled lessons recurred".
- **Next phase:** `kbd-next-phase.sh` (the skill copy at :117-131, plus the copies in `shared/scripts/` and `.codex/`) seeds from `## Next Phase Seed` and appends `## Carry-forward lessons`.
- `enqueue-learning-job.py` adds `projectId` from the resolver.

**PK changes** (`pk-learning-worker/src/main.rs`)
- `LearningJob.project_id` (serde default) is used in `enqueue_memory` (:731).
- After `store.upsert` in `process_job` (:527), call `pk_store::commit_prompt_snapshot`. Without it, karpathy session entries are invisible to `pk context` today.

**MINI changes**
- Port the recall logic into `scripts/kbd-memory-recall.mjs`, plus the carried kbd-* SKILL.md steps and the reflect template.

**Gates**
- PK: `cargo test -p pk-learning-worker --test worker`. A job with `projectId` produces `user_id` = that id, and `pk context` returns the session entry.
- MINI: `node --test skills/carried-payload.test.mjs`.
- SP: new `shared/scripts/tests/test-kbd-memory-loop.sh`, run against live SM, real pk and a worker built from PK:
  1. Write a reflection with a unique token.
  2. Run the worker's `run-once`.
  3. Run `kbd-next-phase`.
  4. Send a PreToolUse `Skill{kbd-assess}` payload.
  5. Assert the token appears in `prior-context.md` from SM and from pk, and that the seed and carry-forward appear in the new `goals.md`.

### PR 7 (SP) + PR 8 (PK) — user/global scopes and promotion

**SP changes**
- **`memory-writeback.sh` routes per line:**
  - `[GLOBAL]` → SM `global` + `pk ingest --scope shared --yes`.
  - `[USER]` → SM `userScope` + pk shared.
  - Unmarked → project.
- **Attach user/global context:**
  - Recall adds the `userScope` search.
  - `kbd-open.sh` adds a bounded 2s SM user/global search plus pending promotion candidates.
  - The per-prompt pk hook already covers shared/global. Accepted promotions are mirrored into pk shared, so prompts stay local and fast.
- **kbd-reflect SKILL.md:** review `pk candidates list --kind promotion`. Accepting or rejecting happens only on human instruction.

**PK changes**
- **New `pk-learning-worker/src/promotion.rs`,** run after `run_once`. It keeps a lesson fingerprint index in `~/.prometheus/learning-index/lessons.jsonl` and proposes a candidate, with evidence, when:
  - a lesson recurs (Jaccard ≥ 0.6) across 2 or more projects;
  - a lesson is tagged `global` but carries no marker; or
  - a lesson names a dependency or CLI and has no repo-relative paths.
- **New command in `pk-cli`:** `pk candidates list|accept|reject --kind promotion|skill`. Accepting:
  - upserts into the shared KB and commits a snapshot;
  - queues an SM `add_memory` for the user or global scope;
  - moves the file to `accepted/`.

**Gates**
- PK: `cargo test -p pk-learning-worker --test promotion`. Seed two projects with the same lesson, run `run-once`, check one candidate with 2 evidence entries, run `pk candidates accept`, and check the shared snapshot and the queued `global` memory op.
- SP: the loop test also checks that a third project's prompt returns the accepted lesson from `[shared:…]`.

### PR 9 (SP) — Feynman/learn gap routing

- **Gap check in the prompt hook** (`karpathy-hook-dispatch.sh`, prompt branch). It calls `pk context --format json` once and renders the existing hook output from it. It flags a gap when all of these hold:
  - pk returns no results, or the top score is below `PROMETHEUS_GAP_MIN_SCORE`;
  - the prompt is problem-shaped;
  - fewer than 2 prompt keywords appear in the project's CLAUDE.md or AGENTS.md.
- **On a gap:** emit one `[prometheus-gap] … consider /learn-goal <topic> or /feynman-loop` line, limited to once per session per topic, and append it to `gaps.jsonl`. With no pk installed, nothing changes.
- **Recall:** writes `## Knowledge gaps` when all sources are empty. kbd-assess and kbd-analyze list them and offer `/learn-goal`. `kbd-open` shows gaps seen 2 or more times.
- **Learning skills** (learn-goal, feynman-loop, learn-kb SKILL.md): the closing step ingests the final explanation into pk (project scope, or shared when the topic is generic) and marks the gap resolved.
- **Gate:** new `shared/scripts/tests/test-prompt-gap.sh` with the real pk:
  - A problem prompt with an empty KB gets one gap line; a repeat gets none.
  - After ingesting a covering doc, no gap line, and the output is identical to `pk context --format hook`.

### PR 10 (PK) + PR 11 (SP) — discovering skills from conversations

**PK changes**
- **New `pk-learning-worker/src/transcript.rs`** parses the whole transcript, not just the final message (`build_session_packet` :565). It extracts:
  - user prompts;
  - artifacts (new Write/Edit files under `docs/`, `reports/` or `*.md`);
  - Bash commands and Skill invocations;
  - corrections made after a skill ran.
- **New `skill_candidates.rs`:**
  - Workflow fingerprints (prompt shingles + tool-sequence 3-grams) go into `learning-index/workflows.jsonl`.
  - **New skill candidate:** a fingerprint seen in 3+ sessions or 2+ projects that no skill covers. It goes into `skill-candidates/pending/` with evidence.
  - **Existing-skill update:** a skill followed by corrections. It goes into `skill-updates/` in the exact format `propose-skill-update.sh` uses.
- **`pk candidates accept --kind skill`** prints the `/pmpo-skill-creator` (or `--update <skill>`) invocation with the evidence path. It never creates the skill on its own.

**SP changes**
- `kbd-open` lists skill candidates.
- The reflect template's `## Codify as Skill?` gets a prompt asking whether a workflow repeated or a report type should become a skill, with evidence. This section is excluded from writeback.
- kbd-reflect reviews skill candidates.
- Fix the `pmpo-skill-creator.md` integration doc.
- Correct the false "Called by evaluate-session.sh" header in `propose-skill-update.sh`; it becomes the manual entry point.

**Gates**
- PK: `cargo test -p pk-learning-worker --test skill_discovery`. Use three similar report-writing transcripts across two projects plus one `Skill(kbd-plan)` followed by corrections. Expect one new-skill candidate and one `kbd-plan` update.
- SP: kbd-open lists both.

### PR 12 (SP) + PR 13 (MINI) — agent teams across components and repos

All in `skills/process/agent-team-creator`.

- **`schemas/team.schema.json`:** optional `card` = `{repo, component, owns[], capabilities[], intake:{intakeRole, label:"team:<id>", rules:[{when, route:"handoff"|"issue"}]}}`. `intakeRole` must exist in `roles`. Mirror it in `runtime/src/types.mts` and `validation.mts`.
- **New `runtime/src/registry.mts`:**
  - **publish:** writes `~/.prometheus/knowledge/shared/teams/<repo>--<id>.json`, the authoritative copy. If pk exists, it also does `pk ingest --scope shared`, so `pk context` can find the team.
  - **discover:** ranks cards by capability overlap and `owns` glob matches.
  - **request:**
    - Same repo and no rule forcing an issue → add a task to the target team's state file through `mutateState`, owned by its intake role. The packet uses the `createHandoff` format (`handoff.mts:17-55`), and `request.sent` / `request.received` events are recorded.
    - Otherwise → `gh issue create --repo <card.repo> --label team:<id>`. If gh is absent, print the packet and the exact command instead.
  - **intake:** imports open labelled issues as intake-role tasks, idempotently. It comments on the issue only with an explicit ack.
- **CLI and instructions:** new `team-publish|team-discover|team-request|team-intake` commands in `runtime/src/cli.mts`. Intake and request guidance goes into the `project-instructions.mts` managed block, plus SKILL.md and reference updates. Rebuild `scripts/*.mjs`.
- **MINI:** port to `skills/agent-team-creator/*`.
- **Gate:** `runtime/test-src/teams.integration.mts` with real git, pk and gh against a sandbox repo set in `PROMETHEUS_TEAM_TEST_REPO` (fails if unset). Steps:
  1. Publish 3 teams across 2 repos.
  2. Discovery ranks the right team first.
  3. A same-repo request creates an intake task.
  4. A cross-repo request creates a labelled issue that `team-intake` imports exactly once.
  5. The issue is closed at the end.
  
  MINI runs `carried-payload`.

### PR 14 (SP + MINI) — final pin bump

PRs 5, 8 and 10 add new prometheus-knowledge commits. Tag `v1.10.0`, then repeat PR 2's pin update in both repos so the skills packs ship the worker and `pk candidates` changes.

---

## Order and dependencies

1 → 2 → 3 → (4 ∥ 5) → 6 → (7 + 8) → 9 → (10 + 11) → 14. PRs 12–13 can run any time after 3.

Only one cargo build at a time across PK PRs and the skill-pack Rust checks. Each PR has its own branch and worktree off `origin/main`. After each PR merges, write project memories, per CLAUDE.md.

## Verification summary

Each PR passes its own gate above before push. At the end, a full manual dogfood:
1. In a scratch project, run `/kbd-reflect`, then `/kbd-next-phase`, then `/kbd-assess`.
2. Confirm `prior-context.md` cites the prior lesson from surreal-memory.
3. Mark a lesson `[GLOBAL]` and confirm a second project's prompt sees it.
4. Confirm a novel problem prompt suggests `/learn-goal`.
5. After three similar report sessions, `kbd-open` lists a skill candidate.
6. A `team-request` across repos opens a labelled issue that the other team's intake imports.
7. Codex: the same SessionStart hook fires under `codex exec`.
