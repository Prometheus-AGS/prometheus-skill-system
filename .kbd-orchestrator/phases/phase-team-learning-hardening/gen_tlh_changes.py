#!/usr/bin/env python3
"""Generate the native-kbd change set for phase phase-team-learning-hardening.

Each change gets spec.md, tasks.json, verification.md and verify.sh, written under
.kbd-orchestrator/changes/<id>/. Idempotent: rerunning rewrites the same bytes.
"""
import json, os, textwrap

KBD = "/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
PHASE = "phase-team-learning-hardening"
WT = "/Users/gqadonis/Projects/prometheus/worktrees"
DESIGN = ".kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md"

REPOS = {
    "pk": dict(name="prometheus-knowledge-rs", base="1bbaecc", clone="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge"),
    "sm": dict(name="surreal-memory-server", base="0af8ae1", clone="/Users/gqadonis/Projects/prometheus/surreal-memory-server"),
    "sp": dict(name="prometheus-skill-system", base="e421715", clone="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack"),
    "mini": dict(name="prometheus-skills-mini", base="56cbf12", clone="/Users/gqadonis/Projects/prometheus/prometheus-skills-mini"),
}

# Gate prerequisites: a gate that cannot run reports BLOCKED (exit 2), never a pass.
REQ_SM = 'curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }'
REQ_PK = 'command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }'
REQ_SURREALDB = 'nc -z 127.0.0.1 "${SURREALDB_PORT:-28000}" || { echo "BLOCKED: SurrealDB 3.3.0 not listening" >&2; exit 2; }'
NO_CARGO_RACE = 'if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi'
PK_V10 = 'pk --version 2>/dev/null | grep -Eq "1\\.(1[0-9]|[2-9][0-9])\\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; }'
PK_V11 = 'pk --version 2>/dev/null | grep -Eq "1\\.(1[1-9]|[2-9][0-9])\\." || { echo "BLOCKED: pk >= 1.11.0 required (E1 not installed)" >&2; exit 2; }'
MINI_NO_NEW_FAILURES = ('node --test --test-reporter=tap 2>&1 | grep -E "^not ok" | sed "s/^not ok [0-9]* - //" | sort > "${TMPDIR:-/tmp}/tli-mini-fail.$$"; '
    'comm -23 "${TMPDIR:-/tmp}/tli-mini-fail.$$" "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt" > "${TMPDIR:-/tmp}/tli-mini-new.$$"; '
    'if [ -s "${TMPDIR:-/tmp}/tli-mini-new.$$" ]; then echo "new test failures beyond the recorded mini baseline:" >&2; cat "${TMPDIR:-/tmp}/tli-mini-new.$$" >&2; exit 1; fi')
SP_REGEN = "node scripts/generate-harness-adapters.js >/dev/null && node scripts/generate-skill-system-distribution.js >/dev/null && git diff --exit-code -- hooks shared/harnesses/generated shared/scripts/generated dist"
SP_ADAPTERS = "npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex"

C = []
def change(**k): C.append(k)

L_MARKDONE = '"`kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger." Applied: task closure in this change uses `begin-task`/`end-task`.'
L_REAL = '"For every external integration, run at least one test against the real service before claiming the work is done." Applied as an acceptance criterion.'
L_LOCK = '"Do not invoke a CLI or subprocess that takes a lock from inside code that already holds the same lock. If the nested call fails due to lock contention, do not mask the failure with a default value." Applied: no default N.'
L_DEPLOY = '"`delivery-cadence` freezes every source named in `ready`; do **not** point those sources at the actively edited main checkout." and "run `git submodule update` before invoking `update-skill-pack.sh`".'
L_PAIRED = '"For each generated format, maintain exactly one authoritative emitter and one authoritative validator." Applied: regenerate with the repository emitters, then run the paired validators.'
L_HEAD = '"After a pull request merges, verify that `main` contains the PR\'s **final head commit**." Applied at merge.'
L_POLL = '"Avoid reentrant lock-taking CLIs and default fallbacks." Applies to the reconcile check invoked inside the cadence checkpoint: it reads files, never the cadence CLI.'

SCRATCH_LIB = "shared/scripts/tests/lib/scratch-surreal.sh"

# ---------------------------------------------------------------- G1: install-time automation
change(id="change-tlh-01-ranked-memory-partition", code="01", repo="sp",
  title="memory-index-partition.py ranks entries (current phase, feedback/GLOBAL, project newest-first, archives) before moving any to the learning store",
  depends=[], design="§G1a",
  why="`scripts/memory-index-partition.py` preserves file order (line 11) and moves the last lines first, so the live partition needed a manual priority reorder (assessment G1a). The live index is already back at 4,088 B of its 4 KB budget.",
  what=[
    "Add a rank key per index bullet: 0 = a `project` entry whose slug or title matches the active phase (read `.kbd-orchestrator/current-waypoint.json` `phase`, else `position-reminder.txt` `Position:`; no phase = rank 0 empty); 1 = `feedback` entries and any entry whose title contains `GLOBAL`; 2 = other `project` entries, newer first by the `YYYYMMDD` or `YYYY-MM-DD` in the file name, undated last; 3 = `archive-*` entries. The trailing \"moved to the learning store\" footer line is not ranked.",
    "Order all bullets by priority: rank 0, then rank 1 (file order), then rank 2 newest first (undated last, file order among equals), then rank 3 (file order). Move bullets to the learning store from the END of that order (archives first, then the oldest rank-2 entries, then rank 1, never rank 0 unless it alone exceeds the budget) until the index fits `--limit`. Kept bullets are written in that priority order. The footer line is never moved and is always written last.",
    "`--dry-run` prints the rank of every line and which would move; output stays deterministic for identical input.",
    "Header docstring describes the ranking; the old \"file order is preserved\" rule is removed.",
  ],
  scope=["scripts/memory-index-partition.py", "scripts/tests/test-memory-partition.sh", "dist/plugins/**"],
  tasks=[("Implement rank classification and rank-ordered moving and writing in memory-index-partition.py", ["scripts/memory-index-partition.py"]),
         ("Extend test-memory-partition.sh with a ranked fixture (active phase entry last in file, archives first) and an idempotence run", ["scripts/tests/test-memory-partition.sh"])],
  accept=[
    "A fixture index whose active-phase entry is the last line and whose first lines are `archive-*` keeps the active-phase entry and moves the archives first.",
    "`feedback`/GLOBAL entries are kept ahead of older `project` entries when the budget forces a choice, and the kept rank-2 entries appear newest first (asserted on the output order).",
    "Running the partition twice on its own output moves nothing the second time (idempotent) and the index is <= `--limit` bytes.",
    "Scratch HOME only; the operator's real MEMORY.md is never read or written by the test.",
  ],
  lessons=[],
  verify=["/bin/bash scripts/tests/test-memory-partition.sh", "npm run check:distribution"])

change(id="change-tlh-02-codex-memories-install-and-doctor", code="02", repo="sp",
  title="The installer sets Codex `[memories] generate_memories = false` and archives memory_summary.md; the doctor checks it",
  depends=[], design="§G1b",
  why="Codex memory generation is disabled only in the operator's hand-edited `~/.codex/config.toml`; a fresh machine regrows `memory_summary.md`, which Codex injects into every thread (assessment G1b). The agent-team exporter already sets the option per generated Codex agent (`adapters-local.mts:26`) but not for the main thread.",
  what=[
    "New `shared/scripts/codex-memories-config.sh` (bash 3.2): idempotently inserts or updates `generate_memories = false` under a `[memories]` table in `${CODEX_HOME:-$HOME/.codex}/config.toml` with a line-level edit that preserves every other line and comment; writes a timestamped backup first; re-parses the result with `python3 -c 'import tomllib'` and restores the backup on a parse failure; archives an existing `memories/memory_summary.md` to `memories-archive/memory_summary-<UTC>.md`; leaves `MEMORY.md`, `raw_memories.md` and every other file in place. `--check` reports state as JSON without writing.",
    "Both installers apply it: `scripts/install-system.js` (the `prometheus setup` / `install.sh` path, which runs its own Codex step and does not call install-skills-flat) spawns the script after its Codex plugin step, and `install_to_codex()` in `scripts/install-skills-flat.sh` calls it through a guarded helper `scripts/lib/install-codex-memories.sh`. Every failure prints a warning and the install completes (installer-must-not-abort). Absent Codex (no `~/.codex`) is silent. `CODEX_MEMORIES_SCRIPT` overrides the script path, for fault injection in tests only.",
    "`prometheus doctor` gains an optional check `codex.memories`: Green when `generate_memories` is `false` and no `memory_summary.md` exists; Yellow otherwise, with the exact repair command; skipped (Green, note) when Codex is not installed. It never counts toward failed checks.",
    "Docs: `docs/guide/06-memory-and-learning.md` and `site/docs/operations/doctors-and-mac-certification.md` describe the setting, the archive and the check.",
  ],
  scope=["shared/scripts/codex-memories-config.sh", "scripts/install-skills-flat.sh", "scripts/lib/install-codex-memories.sh", "scripts/install-system.js", "shared/scripts/tests/test-codex-memories-config.sh",
         "tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs", "tools/prometheus-cli/crates/prometheus-cli/tests/doctor.rs",
         "docs/guide/06-memory-and-learning.md", "site/docs/operations/doctors-and-mac-certification.md", "dist/plugins/**"],
  tasks=[("Write codex-memories-config.sh (line-level edit, backup, re-parse/restore, summary archive, --check JSON)", ["shared/scripts/codex-memories-config.sh"]),
         ("Call it from install-system.js and, through scripts/lib/install-codex-memories.sh, from install_to_codex(), with every failure path guarded", ["scripts/install-system.js", "scripts/lib/install-codex-memories.sh", "scripts/install-skills-flat.sh"]),
         ("Add the optional codex.memories doctor check and its integration test cases", ["tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs", "tools/prometheus-cli/crates/prometheus-cli/tests/doctor.rs"]),
         ("Add test-codex-memories-config.sh (scratch CODEX_HOME: comments preserved, idempotent, corrupt-TOML restore, summary archived, codex debug prompt-input carries no memories text when codex is present; a failing CODEX_MEMORIES_SCRIPT leaves both installer entry points exiting 0 with a warning) and update the docs", ["shared/scripts/tests/test-codex-memories-config.sh", "docs/guide/06-memory-and-learning.md", "site/docs/operations/doctors-and-mac-certification.md"])],
  accept=[
    "On a scratch `CODEX_HOME` whose config.toml has comments and other tables, the script adds `[memories] generate_memories = false`, keeps every other line byte-identical, and a second run changes nothing.",
    "A config.toml that already has `[memories] generate_memories = true` is changed to `false` in place, not duplicated.",
    "When the edited file would not parse, the original is restored and the script exits non-zero; the installer still completes with a warning.",
    "An existing `memories/memory_summary.md` is moved to `memories-archive/`; `MEMORY.md` and `raw_memories.md` are untouched.",
    "With codex on PATH, `codex debug prompt-input` under the scratch `CODEX_HOME` (no auth.json at all: no copy and no symlink, so the operator's credentials are unreachable) contains no text from a seeded `memories/MEMORY.md`. Without codex, or if prompt-input refuses to run unauthenticated, that step reports BLOCKED (exit 2), never pass.",
    "Test order: all deterministic cases run first; the prompt-input probe runs last as an optional sub-case that prints SKIP when codex is absent or refuses unauthenticated, and only `REQUIRE_CODEX_PROBE=1` turns that SKIP into exit 2. The doctor check is the standing guard for the inert-files conclusion.",
    "The install-system.js step is exported as `applyCodexMemories({home})` and exercised in isolation with `node -e` under a scratch HOME.",
    "With `CODEX_MEMORIES_SCRIPT` pointing at a script that exits 1, the guarded helper sourced from install-skills-flat.sh and the install-system.js Codex memories step (run under a scratch HOME) both return 0 and print a warning.",
    "`prometheus doctor --json` reports `codex.memories` Yellow with the repair command for an unset config, Green after the script runs, and the check never increments failed.",
  ],
  lessons=["\"Install Codex memory settings by hand only\" was knowledge gap 2 in `prior-context.md`; this change closes it."],
  verify=[NO_CARGO_RACE, "/bin/bash shared/scripts/tests/test-codex-memories-config.sh",
          "cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test doctor", "npm run check:distribution"])

change(id="change-tlh-03-versioned-cadence-refresh-procedure", code="03", repo="sp",
  title="Ship the skill-pack refresh procedure in delivery-cadence: profile inputs, state.json parity, fail-loud, submodule update",
  depends=[], design="§G1c",
  why="`.prometheus/cadence/procedures/refresh-skill-pack.sh` is git-ignored (`.gitignore:94`) and carries this machine's only copy of four recorded fixes: deploy-worktree freezing, state.json parity, submodule update after fast-forward, and the surreal-memory kickstart (assessment G1c).",
  what=[
    "Add `skills/process/delivery-cadence/scripts/refresh-skill-pack.sh` (under the skill's existing `scripts/` directory, which the validators and distribution already carry; bash 3.2, `set -euo pipefail`) with inputs from flags or env: `--deploy <worktree>`, `--state <state.json>`, `--mode full|verify|auto`, `--services <launchd labels, comma-separated>`.",
    "`--mode auto` reads the iteration number from `state.json` directly (never the cadence CLI, which holds the lock inside a checkpoint); odd = full, even = verify. A missing, unreadable or non-numeric iteration exits 2 with a message and never defaults.",
    "Full mode: fast-forward the deploy worktree to `origin/main` (refuse a dirty or diverged worktree, exit 1), `git submodule update --init --recursive`, run `scripts/update-skill-pack.sh --force` and `scripts/install-binaries.sh` from the deploy worktree, `launchctl kickstart -k gui/$UID/<label>` for each service, then print a JSON summary (source commit, pk/worker/surreal-memory versions, health, plugin generation). Verify mode prints the same summary without changing anything.",
    "For tests only, and only when `REFRESH_TEST_MODE=1`, `REFRESH_UPDATE_CMD`, `REFRESH_INSTALL_CMD` and `REFRESH_KICKSTART_CMD` override the update, install and kickstart commands (defaults: the real scripts and `launchctl`).",
    "`references/profile.md` documents the procedure and how a profile points a checkpoint at it. The local `.prometheus/cadence/procedures/refresh-skill-pack.sh` becomes a shim that `exec`s `$HOME/.claude/skills/delivery-cadence/scripts/refresh-skill-pack.sh`, the installed flat path. The shim is committed as an example under `examples/`; the local file stays untracked.",
  ],
  scope=["skills/process/delivery-cadence/scripts/refresh-skill-pack.sh", "skills/process/delivery-cadence/references/profile.md",
         "skills/process/delivery-cadence/examples/refresh-skill-pack-shim.sh", "skills/process/delivery-cadence/SKILL.md",
         "shared/scripts/tests/test-cadence-refresh-procedure.sh", "dist/plugins/**"],
  tasks=[("Write scripts/refresh-skill-pack.sh with profile inputs, state.json parity, fail-loud exits and the JSON summary", ["skills/process/delivery-cadence/scripts/refresh-skill-pack.sh"]),
         ("Document it in references/profile.md and SKILL.md and add the shim example", ["skills/process/delivery-cadence/references/profile.md", "skills/process/delivery-cadence/SKILL.md", "skills/process/delivery-cadence/examples/refresh-skill-pack-shim.sh"]),
         ("Add test-cadence-refresh-procedure.sh (scratch git repo with a submodule as the deploy worktree; parity from state.json odd/even; missing/garbage iteration exits 2; dirty worktree exits 1; verify mode changes nothing (HEAD, submodule commit and a stub-call log unchanged); the JSON summary has sourceCommit, versions and health keys; runs under /bin/bash; local submodule via `-c protocol.file.allow=always`)", ["shared/scripts/tests/test-cadence-refresh-procedure.sh"])],
  accept=[
    "With `state.json` iteration 7 `--mode auto` selects full; iteration 8 selects verify; a missing state file, a non-numeric iteration, or an unreadable file exits 2 and does no work.",
    "Full mode on a scratch deploy worktree fast-forwards it and leaves its submodule at the recorded commit (clean `git status`); a dirty or diverged worktree exits 1 before any install step.",
    "The procedure runs under `/bin/bash` (3.2) in the test.",
    "Verify mode leaves HEAD, the submodule commit and the stub-call log unchanged, and its JSON summary carries `sourceCommit`, `versions` and `health` keys.",
    "In the test, install and kickstart steps are replaced by recording stubs through `REFRESH_UPDATE_CMD`/`REFRESH_INSTALL_CMD`/`REFRESH_KICKSTART_CMD` env overrides; the real machine is never refreshed by the gate.",
  ],
  lessons=[L_DEPLOY, L_LOCK],
  verify=["/bin/bash shared/scripts/tests/test-cadence-refresh-procedure.sh", "npm run check:distribution"])

# ---------------------------------------------------------------- G2: test isolation and load
change(id="change-tlh-04-scratch-surreal-for-envelope-test", code="04", repo="sp",
  title="Shared scratch surreal-memory test library; memory-envelope.integration runs against it, never the live :23001",
  depends=[], design="§G2a",
  why="`runtime/test-src/memory-envelope.integration.mts:13` defaults to the live `http://127.0.0.1:23001` service; it flaked twice at load ~250 and writes test records into the operator's store (assessment G2a).",
  what=[
    f"Extract the scratch-server start/stop steps of `shared/scripts/tests/test-kbd-memory-loop.sh` into `{SCRATCH_LIB}` (bash 3.2): `scratch_surreal_start` picks a free port, creates a temp data dir, starts `surreal-memory-server` (binary from `TLI_SM_BIN` or PATH, >= 1.10.0, with the local embedding executor), waits for `/health`, exports `SURREAL_MEMORY_URL`; `scratch_surreal_stop` kills it and removes the dir; a missing binary or old version calls `blocked` (exit 2).",
    "`test-kbd-memory-loop.sh` and `test-subagent-delivery.sh` source the library instead of their inline copies (behaviour unchanged).",
    "New `skills/process/agent-team-creator/tests/run-memory-envelope.sh` starts a scratch server via the library, runs the compiled `memory-envelope.integration.mjs`, stops the server.",
    "`memory-envelope.integration.mts` drops the `:23001` default: without `SURREAL_MEMORY_URL` it exits 2 (BLOCKED); rebuild `tests/*.mjs` with `npm run build:tests`.",
  ],
  scope=[SCRATCH_LIB, "shared/scripts/tests/test-kbd-memory-loop.sh", "shared/scripts/tests/test-subagent-delivery.sh",
         "skills/process/agent-team-creator/runtime/test-src/memory-envelope.integration.mts", "skills/process/agent-team-creator/tests/memory-envelope.integration.mjs",
         "skills/process/agent-team-creator/tests/run-memory-envelope.sh", "dist/plugins/**"],
  tasks=[(f"Extract {SCRATCH_LIB} and switch test-kbd-memory-loop.sh and test-subagent-delivery.sh to it", [SCRATCH_LIB, "shared/scripts/tests/test-kbd-memory-loop.sh", "shared/scripts/tests/test-subagent-delivery.sh"]),
         ("Remove the :23001 default from memory-envelope.integration.mts, rebuild the .mjs, and add run-memory-envelope.sh", ["skills/process/agent-team-creator/runtime/test-src/memory-envelope.integration.mts", "skills/process/agent-team-creator/tests/memory-envelope.integration.mjs", "skills/process/agent-team-creator/tests/run-memory-envelope.sh"])],
  accept=[
    "`run-memory-envelope.sh` passes. It refuses (exit 1) if `SURREAL_MEMORY_URL` points at port 23001. It reads every record the test wrote back from the scratch server. After `scratch_surreal_stop` it checks that no process still holds the scratch data dir (`pgrep -f <dir>`), exiting 1 otherwise. All of this is checked inside the script the gate runs. The live service is never contacted.",
    "`node tests/memory-envelope.integration.mjs` with `SURREAL_MEMORY_URL` unset exits 2.",
    "`test-kbd-memory-loop.sh` and `test-subagent-delivery.sh --harness none` still pass using the shared library. `scratch_surreal_stop` itself verifies that no process survives on its data dir and fails otherwise, so every caller gets the check.",
  ],
  lessons=[],
  verify=["command -v surreal-memory-server >/dev/null || { echo 'BLOCKED: surreal-memory-server not on PATH' >&2; exit 2; }",
          "/bin/bash skills/process/agent-team-creator/tests/run-memory-envelope.sh",
          "if grep -q 23001 skills/process/agent-team-creator/tests/memory-envelope.integration.mjs; then echo 'compiled .mjs still references :23001 (rebuild with npm run build:tests)' >&2; exit 1; fi", "( cd skills/process/agent-team-creator/runtime && unset SURREAL_MEMORY_URL; node ../tests/memory-envelope.integration.mjs; test $? -eq 2 )",
          "/bin/bash shared/scripts/tests/test-kbd-memory-loop.sh", "/bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness none",
          "npm run check:distribution"])

change(id="change-tlh-05-query-embedding-cache", code="05", repo="sm",
  title="surreal-memory caches query embeddings (quick_cache, query path only) after measuring single-embedding latency under load; release 1.10.1",
  depends=[], design="§G2b",
  why="Every `POST /api/v1/search` embeds its query again (`search_memories` -> `embed_text` -> `embedding_service.embed`, no cache); one SubagentStart recall sends 4 searches (5 for the lead) with identical text in series and misses its 2.8 s deadline under load (assessment G2b, analysis §G2b).",
  what=[
    "Measure first: `tests/query_embedding_latency.rs` (integration, real executor, `#[ignore]` by default, run explicitly) records p50/p95 latency of one query embedding idle and with 8 concurrent writers; the result is committed to `docs/perf/query-embedding-latency.md`, which includes machine-readable lines `IDLE_P50_SECONDS=`, `IDLE_P95_SECONDS=` and `LOADED_P95_SECONDS=`. If idle p95 > 2.0 s the change stops and reports instead of implementing (the cache cannot rescue it).",
    "Add `quick_cache = \"0.6\"` (the version already in Cargo.lock through surrealdb-core; no second copy) and a bounded query-embedding cache in `SurrealStorage`: key = (embedding model id, NFC-normalised whitespace-collapsed query text), capacity from `SURREAL_MEMORY_QUERY_EMBED_CACHE` (default 512, 0 disables). Use `get_value_or_guard_async` so concurrent identical misses embed once. Only `search_memories` (and therefore hybrid search) uses it; the write path never reads or fills it.",
    "Expose cache hits/misses as a `query_embed_cache` object in `GET /api/v2/operations/stats` (`src/operations.rs`), proven by a root-crate integration test `tests/query_embed_cache_stats.rs` that drives the real router.",
    "Bump the release version to 1.10.1 in the same three files release PR #45 changed for 1.10.0: `Cargo.toml`, `Cargo.lock` and `openapi/surreal-memory-v2.openapi.json` (the repository has no CHANGELOG). The tag push is operator-approved and is not part of the gate.",
  ],
  scope=["Cargo.toml", "Cargo.lock", "crates/surreal-memory/Cargo.toml", "crates/surreal-memory/src/storage/surreal.rs",
         "crates/surreal-memory/tests/query_embedding_cache.rs", "crates/surreal-memory/tests/query_embedding_latency.rs",
         "docs/perf/query-embedding-latency.md", "openapi/surreal-memory-v2.openapi.json", "src/operations.rs", "tests/query_embed_cache_stats.rs"],
  tasks=[("Write the ignored latency integration test, run it on this machine, and commit docs/perf/query-embedding-latency.md (stop and report if idle p95 > 2.0 s)", ["crates/surreal-memory/tests/query_embedding_latency.rs", "docs/perf/query-embedding-latency.md"]),
         ("Add the bounded query-path embedding cache with miss de-duplication and hit/miss stats", ["Cargo.toml", "Cargo.lock", "crates/surreal-memory/Cargo.toml", "crates/surreal-memory/src/storage/surreal.rs", "src/operations.rs"]),
         ("Add the query_embedding_cache and root query_embed_cache_stats integration tests and bump to 1.10.1 (Cargo.toml, Cargo.lock, openapi)", ["crates/surreal-memory/tests/query_embedding_cache.rs", "tests/query_embed_cache_stats.rs", "Cargo.toml", "Cargo.lock", "openapi/surreal-memory-v2.openapi.json"])],
  accept=[
    "Latency evidence is committed before the cache code, with idle and loaded p50/p95.",
    "Through the real storage with a counting embedding service, 4 searches with the same text embed once; 4 concurrent identical searches embed once; a different text embeds again; capacity 0 embeds every time.",
    "Writing a memory never reads or fills the query cache (count unchanged by `add_memory`).",
    "`Cargo.lock` contains exactly one `quick_cache` entry.",
    "`GET /api/v2/operations/stats` includes `query_embed_cache` with hits and misses (asserted by the root-crate test `tests/query_embed_cache_stats.rs` through the real router).",
    "Existing `search_correctness` integration test still passes.",
  ],
  lessons=[L_REAL],
  verify=[NO_CARGO_RACE, "awk -F= '/^IDLE_P95_SECONDS=/{found=1; if ($2+0 > 2.0) bad=1} END{exit !(found && !bad)}' docs/perf/query-embedding-latency.md || { echo 'IDLE_P95_SECONDS missing or > 2.0 (stop rule)' >&2; exit 1; }", "cargo test -p surreal-memory --test query_embedding_cache", "cargo test --test query_embed_cache_stats", "cargo test -p surreal-memory --test search_correctness",
          "test \"$(grep -c '^name = \"quick_cache\"' Cargo.lock)\" -eq 1", "grep -Eqi 'idle.*p50' docs/perf/query-embedding-latency.md && grep -Eqi 'idle.*p95' docs/perf/query-embedding-latency.md && grep -Eqi 'loaded.*p95' docs/perf/query-embedding-latency.md || { echo 'latency evidence lacks idle p50/p95 and loaded p95' >&2; exit 1; }",
          "perf=$(git log --diff-filter=A --format=%H -- docs/perf/query-embedding-latency.md | tail -1); cache=$(git log -S get_value_or_guard_async --format=%H -- crates/surreal-memory/src/storage/surreal.rs | tail -1); test -n \"$perf\" && test -n \"$cache\" && git merge-base --is-ancestor \"$perf\" \"$cache\" && test \"$perf\" != \"$cache\" || { echo 'latency evidence must be committed before the cache code' >&2; exit 1; }",
          "grep -Eq '^version = \"1\\.10\\.1\"' Cargo.toml && grep -q '\"version\": \"1.10.1\"' openapi/surreal-memory-v2.openapi.json && grep -A1 '^name = \"surreal-memory-server\"' Cargo.lock | grep -q '1.10.1' || { echo '1.10.1 bump incomplete (Cargo.toml, openapi, Cargo.lock)' >&2; exit 1; }"])

change(id="change-tlh-06-ledger-reconciliation", code="06", repo="sp",
  title="kbd-apply mark-done syncs the ledger; kbd-apply reconcile detects and repairs task/ledger drift; cadence and reflect run it",
  depends=["change-tlh-03-versioned-cadence-refresh-procedure"], design="§G2c",
  why="`kbd-apply.sh:681-683`: `mark-done` flips the backend flag only, so last phase's ledger showed 13 of 25 changes while every `tasks.json` said done, and nothing warned (assessment G2c).",
  what=[
    "`mark-done <change> <id>` calls `sync_progress` after `b_mark_done` (same as `end-task`, without firing hooks) and prints that hooks were not fired.",
    "New `reconcile [<phase>] [--repair] [--json]`: for every change in the phase plan, compare the backend's done flags (native-kbd `tasks.json`, openspec `tasks.md`) with the canonical runtime ledger; print one line per drifted task and exit 1 on drift, 0 when clean. `--repair` replays each drifted task through the `begin-task`/`end-task` path and re-checks.",
    "Inputs `reconcile` reads: the phase's change list from the canonical runtime (`prometheus kbd --path <root> status --json`, the active phase's changes), each change's backend task file, and the ledger projection `.kbd-orchestrator/phases/<phase>/progress.json` (`changes[].tasks`). It never writes the ledger except through `--repair`'s begin-task/end-task path.",
    "`/kbd-reflect` SKILL.md step 4 runs `kbd-apply reconcile` before reading progress.json and stops on drift with the repair command.",
    "The delivery-cadence refresh procedure (change 03) gains `--kbd-root <repo>` and `--reconcile-phase <phase|auto>` (auto = `phase` from `<kbd-root>/.kbd-orchestrator/current-waypoint.json`). The committed shim example passes `--kbd-root` and `--reconcile-phase auto`, so every cadence iteration launched through the shim runs the check. The procedure that runs `kbd-apply reconcile` read-only in verify and full modes and reports drift in its JSON summary (it reads files; it never calls the cadence CLI).",
  ],
  scope=["skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh", "skills/process/kbd-process-orchestrator/skills/kbd-apply/SKILL.md",
         "skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md", "skills/process/delivery-cadence/scripts/refresh-skill-pack.sh",
         "shared/scripts/tests/test-kbd-apply-reconcile.sh", "shared/scripts/tests/test-cadence-refresh-procedure.sh", "skills/process/delivery-cadence/examples/refresh-skill-pack-shim.sh", "dist/plugins/**"],
  tasks=[("Make mark-done sync progress and add the reconcile subcommand with --repair and --json", ["skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh", "skills/process/kbd-process-orchestrator/skills/kbd-apply/SKILL.md"]),
         ("Wire reconcile into kbd-reflect step 4, the refresh procedure's --kbd-root/--reconcile-phase and the shim example, and add a --reconcile-phase case (stub kbd-apply on PATH, JSON summary has a reconcile field, a cadence.mjs tripwire on PATH is never called) to test-cadence-refresh-procedure.sh", ["skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md", "skills/process/delivery-cadence/scripts/refresh-skill-pack.sh", "skills/process/delivery-cadence/examples/refresh-skill-pack-shim.sh", "shared/scripts/tests/test-cadence-refresh-procedure.sh"]),
         ("Add test-kbd-apply-reconcile.sh (scratch KBD project with the real runtime: drift created by flipping tasks.json, detect exit 1, --repair to clean exit 0, mark-done leaves no drift)", ["shared/scripts/tests/test-kbd-apply-reconcile.sh"])],
  accept=[
    "In a scratch KBD project, a task flipped done in `tasks.json` without the ledger is reported by `reconcile` (exit 1, names change and task); `--repair` makes the next `reconcile` exit 0 and progress.json counts it.",
    "After `mark-done`, `reconcile` exits 0.",
    "The refresh procedure with `--reconcile-phase` includes a `reconcile` field in its JSON summary and never invokes `cadence.mjs`; the shim example passes `--kbd-root` and `--reconcile-phase auto`.",
    "The test runs the installed `prometheus kbd` runtime with scratch HOME and a scratch project root (`mktemp -d`, initialised by `prometheus kbd init` or the runtime's documented bootstrap). It never runs against this repository's `.kbd-orchestrator` or the operator's `~/.prometheus`.",
  ],
  lessons=[L_MARKDONE, L_POLL],
  verify=["test -f skills/process/delivery-cadence/scripts/refresh-skill-pack.sh || { echo 'BLOCKED: dependency change-tlh-03 not in this base (rebase onto main after it merges)' >&2; exit 2; }", "command -v prometheus >/dev/null || { echo 'BLOCKED: prometheus CLI not on PATH' >&2; exit 2; }",
          "/bin/bash shared/scripts/tests/test-kbd-apply-reconcile.sh", "/bin/bash shared/scripts/tests/test-cadence-refresh-procedure.sh", "npm run check:distribution"])

# ---------------------------------------------------------------- G3: real-world verification
change(id="change-tlh-07-cortex-mirror-real-service", code="07", repo="sp",
  title="Verify the D3 Cortex mirror against the real Cortex 2.0.3 MCP server on a scratch CORTEX_DATA_DIR",
  depends=[], design="§G3a",
  why="`test-cortex-mirror.sh` exercises the mirror only against a stub MCP server (line 4); real Cortex 2.0.3 is installed and honours `CORTEX_DATA_DIR`, so a real-service test is possible without touching `~/.cortex` (assessment G3a, analysis §G3a).",
  what=[
    "Add a `real` case to `shared/scripts/tests/test-cortex-mirror.sh`: locate the installed Cortex under the real HOME *before* switching HOME, then pass it explicitly through the existing `PROMETHEUS_CORTEX_MCP` override (`learning_write.py::cortex_command`), because the default glob under a scratch HOME finds nothing; require its `node_modules/@xenova/transformers/.cache` model directory to exist (else BLOCKED, exit 2 — never download into the plugin cache); run with `CORTEX_DATA_DIR` and HOME pointing at scratch directories; write a lesson through `learning_write.py` with the mirror enabled; then call `cortex_recall` on the real server over stdio JSON-RPC by `projectId` and assert the lesson text and its role tag (in `context`) come back.",
    "Keep the stub cases (absent Cortex silent; misbehaving server does not block the write).",
    "If the real case exposes a mirror defect (payload shape, projectId mapping), fix it in `shared/scripts/lib/learning_write.py`.",
    "Document in `site/docs/memory/cortex-mirror.md`, registered in `site/sidebars.js` under the memory items, that the mirror is verified against Cortex 2.0.3. The test records the Cortex version it ran against and fails if it is not 2.0.3, unless `CORTEX_EXPECTED_VERSION` overrides it.",
  ],
  scope=["shared/scripts/tests/test-cortex-mirror.sh", "shared/scripts/lib/learning_write.py", "site/docs/memory/cortex-mirror.md", "site/sidebars.js", "dist/plugins/**"],
  tasks=[("Add the real-Cortex case to test-cortex-mirror.sh (scratch CORTEX_DATA_DIR and HOME, model-cache precondition, recall by projectId, role tag in context)", ["shared/scripts/tests/test-cortex-mirror.sh"]),
         ("Fix any mirror defect the real case exposes in learning_write.py and document the verified version", ["shared/scripts/lib/learning_write.py", "site/docs/memory/cortex-mirror.md", "site/sidebars.js"])],
  accept=[
    "The real case passes against Cortex 2.0.3: a lesson written through `learning_write.py` is returned by real `cortex_recall` for its projectId with the role text present.",
    "`~/.cortex` is unchanged by the test (mtime and size of `memory.db` before == after) and nothing is written under `~/.claude/plugins/cache`.",
    "With the model cache absent (simulated by pointing discovery at a copy without it), the case exits 2, not 0.",
    "The stub cases still pass.",
    "The `memory.db` and plugin-cache checks and the absent-model case are assertions inside `test-cortex-mirror.sh`, which the gate runs, so the gate proves them.",
  ],
  lessons=[L_REAL],
  verify=["ls -d ~/.claude/plugins/cache/cortex/cortex/*/dist/mcp-server.js >/dev/null 2>&1 || { echo 'BLOCKED: Cortex not installed' >&2; exit 2; }",
          "/bin/bash shared/scripts/tests/test-cortex-mirror.sh", "npm run check:distribution"])

change(id="change-tlh-08-recall-scoping-and-eval", code="08", repo="sp",
  title="Recall admits untagged pk entries only for the current project or on a lexical match; a fixed evaluation fixture gates recall quality",
  depends=["change-tlh-04-scratch-surreal-for-envelope-test"], design="§G3b",
  why="`pk_allowed` (`learning_recall.py:403-404`) admits every untagged pk entry, so `prior-context.md`'s \"pk knowledge\" block filled with other projects' legacy ingests (KnowMe, avatars, Actix) while the lesson block was relevant (assessment G3b).",
  what=[
    "`pk_allowed`: an untagged entry is admitted when its pk scope is `project`, meaning it came from the current repository's own pk KB root. The off-topic entries observed are `[shared:...]` and `[global:...]` scopes, not `project`. Otherwise it is admitted only when `lexical_similarity(query, title + excerpt)` >= `PK_UNTAGGED_MIN_SIMILARITY`; tagged entries keep today's rules.",
    "Scope definitions for pk entries: *project* = returned by pk for the scratch repo's own KB (the test runs with cwd = a scratch repo root containing `.prometheus/project.json` and HOME = scratch, so pk resolves only scratch roots); *role* = carries a `role:<current role>` tag. *Foreign* = any shared/global-scope entry whose project tag or source names another project.",
    "The threshold is set from the evaluation fixture, not guessed: pick the smallest value at which the fixture's foreign-project entries are all excluded, record it as the constant with a comment naming the fixture.",
    "Add `shared/scripts/tests/test-recall-quality.sh`: a scratch surreal-memory (change 04 library) and a scratch pk KB seeded with project lessons, foreign-project untagged entries and global feedback; six fixed phase-goal queries split into 3 tuning and 3 held-out; the threshold is chosen on the tuning queries only, and the assertions run on the held-out ones; assert for each that >= 3 of the top 5 recalled items carry the current project or role scope and none is a foreign-project entry; print the per-query table.",
  ],
  scope=["shared/scripts/lib/learning_recall.py", "shared/scripts/tests/test-recall-quality.sh", "shared/scripts/tests/fixtures/recall-quality/**", "dist/plugins/**"],
  tasks=[("Restrict untagged pk admission in pk_allowed with the PK_UNTAGGED_MIN_SIMILARITY constant", ["shared/scripts/lib/learning_recall.py"]),
         ("Add the recall-quality fixture and test-recall-quality.sh, then set the threshold from it", ["shared/scripts/tests/test-recall-quality.sh", "shared/scripts/tests/fixtures/recall-quality/**"])],
  accept=[
    "For each of the three held-out fixture queries, >= 3 of the top 5 items are current-project or role scope and 0 are foreign-project untagged entries.",
    "The fixture contains a current-project untagged pk entry relevant to a held-out query, and the test asserts it appears in that query's recalled items.",
    "`test-kbd-memory-loop.sh` and `test-subagent-delivery.sh --harness none` still pass.",
    "Uses scratch HOME, a scratch surreal-memory and a scratch pk root; the real stores are never queried.",
  ],
  lessons=[],
  verify=["test -f shared/scripts/tests/lib/scratch-surreal.sh || { echo 'BLOCKED: dependency change-tlh-04 not in this base (rebase onto main after it merges)' >&2; exit 2; }", "command -v surreal-memory-server >/dev/null || { echo 'BLOCKED: surreal-memory-server not on PATH' >&2; exit 2; }", REQ_PK,
          "/bin/bash shared/scripts/tests/test-recall-quality.sh", "/bin/bash shared/scripts/tests/test-kbd-memory-loop.sh",
          "/bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness none", "npm run check:distribution"])

change(id="change-tlh-09-rebase-regenerate", code="09", repo="sp",
  title="scripts/rebase-regenerate.sh resolves generated-only rebase/merge conflicts by regenerating, then validates",
  depends=[], design="§G3c",
  why="Six PRs last phase needed rebases, mostly in generated files (hook bundles, harness manifests, `dist/**`), each resolved by hand-running the generators (assessment G3c, previous reflection Delta 2).",
  what=[
    "`scripts/generated-paths.mjs` prints the authoritative generated-path set derived from the generators' declared outputs: the `skill-system.json` target matrix used by `generate-skill-system-distribution.js` (which is also what `build:codex` runs), and the output list of `generate-harness-adapters.js`, exported by each generator as a function or `--list-outputs` flag. `check:distribution` and the new script share it.",
    "`scripts/rebase-regenerate.sh` (bash 3.2): requires an in-progress rebase or merge; lists conflicted paths; exits 1 naming them if any is not in the generated set; otherwise resolves each to either side (content discarded), runs `node scripts/generate-harness-adapters.js` and `node scripts/generate-skill-system-distribution.js` (the same script `build:codex` runs), then `npm run check:distribution`, `npm run validate:harness-adapters`, `npm run validate:codex`; on success stages the generated paths and prints the `git rebase --continue` / `git commit` command (it never continues itself).",
    "`docs/CONTRIBUTING.md` documents it; an optional `.gitattributes` merge-driver registration is described but not installed.",
  ],
  scope=["scripts/generated-paths.mjs", "scripts/rebase-regenerate.sh", "scripts/generate-skill-system-distribution.js", "scripts/generate-harness-adapters.js", "scripts/tests/test-rebase-regenerate.sh", "docs/CONTRIBUTING.md"],
  tasks=[("Add scripts/generated-paths.mjs (with output-list exports from both generators) and use it from the distribution check", ["scripts/generated-paths.mjs", "scripts/generate-skill-system-distribution.js", "scripts/generate-harness-adapters.js"]),
         ("Write scripts/rebase-regenerate.sh and document it in CONTRIBUTING.md", ["scripts/rebase-regenerate.sh", "docs/CONTRIBUTING.md"]),
         ("Add test-rebase-regenerate.sh (temp repo built from the worktree's current files including uncommitted ones (copied excluding .git and node_modules, then `git init` + commit), `node_modules` symlinked: two branches editing one skill's SKILL.md so dist/ conflicts; rebase; helper resolves and validators pass; a real source conflict is refused with exit 1)", ["scripts/tests/test-rebase-regenerate.sh"])],
  accept=[
    "In a temporary clone, a rebase whose only conflicts are generated paths is resolved by the helper, the validators pass, and `git diff --cached` contains only generated paths.",
    "A rebase with one conflicted source file (non-generated) exits 1 naming that file and changes nothing.",
    "Outside a rebase or merge the helper exits 2 with a message.",
    "`check:distribution` still passes and uses the shared generated-path set.",
  ],
  lessons=[L_PAIRED],
  verify=["node scripts/generated-paths.mjs | grep -q '^dist/' || { echo 'generated-paths lists no dist paths' >&2; exit 1; }", "grep -q generated-paths scripts/generate-skill-system-distribution.js || { echo 'distribution check does not use generated-paths' >&2; exit 1; }", "/bin/bash scripts/tests/test-rebase-regenerate.sh", "npm run check:distribution", "npm run validate:harness-adapters", "npm run validate:codex"])
# ---------------------------------------------------------------- writers
def worktree(ch):
    return f"{WT}/tlh-{ch['code'].lower()}"

def write_change(ch):
    d = os.path.join(KBD, "changes", ch["id"]); os.makedirs(d, exist_ok=True)
    repo = REPOS[ch["repo"]]
    deps = ", ".join(f"`{x}`" for x in ch["depends"]) or "none"
    base = (f"a worktree branch off `origin/main` of {repo['name']} at or after `{repo['base']}`, at `{worktree(ch)}` "
            f"(created with `git -C {repo['clone']} worktree add`). Delivered through a pull request; the user merges.")
    if ch["code"] in ("A5b",):
        base = "no code branch: an operator-authored commit on prometheus-skills-mini `main`."
    spec = [f"# {ch['id']}", "", f"**Title:** {ch['title']}", f"**Plan PR:** {ch['code']}", f"**Repository:** `{repo['name']}`",
            f"**Phase:** {PHASE}", f"**Depends on:** {deps}", "**Backend:** native-kbd", f"**Base branch:** {base}",
            f"**Analysis section:** `{DESIGN}` {ch['design']}", "", "## Why", "", ch["why"], "", "## What Changes", ""]
    spec += [f"- {w}" for w in ch["what"]]
    spec += ["", "## Scope", ""] + [f"- `{s}`" for s in ch["scope"]]
    spec += ["", "## Constraints", "",
             "- Implement the whole change, then run the gate once (implementation-first, integration-only).",
             "- One cargo/rustc build on the machine at a time.",
             "- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).",
             "- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.",
             "- Hooks exit 0 and print nothing when a dependency is absent.",
             "- The last task regenerates generated outputs (`node scripts/generate-harness-adapters.js && node scripts/generate-skill-system-distribution.js`, which is also `build:codex`) before the gate runs `check:distribution`.",
             "- A change with a Depends-on starts from `origin/main` after that dependency has merged; its verify.sh exits 2 (BLOCKED) when the dependency is absent from the base.",
             "- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.",
             "- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.", ""]
    if ch["repo"] != "sp":
        spec = [l for l in spec if "generate-harness-adapters.js &&" not in l and "Generated hooks/dist" not in l and "Generated outputs (`dist/plugins" not in l]
    spec += ["## Plan reference", "", f"Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/{PHASE}/plan.md` (rows keyed by `{ch['id']}` and backend task ID).", ""]
    if ch.get("lessons"):
        spec += ["## Recalled lessons (from prior-context.md)", ""] + [f"- {l}" for l in ch["lessons"]] + [""]
    open(os.path.join(d, "spec.md"), "w").write("\n".join(spec))

    tasks = {"changeId": ch["id"], "schemaVersion": "1", "tasks": []}
    prior = {}
    prior_path = os.path.join(d, "tasks.json")
    if os.path.exists(prior_path):  # never reset completion state recorded by kbd-apply
        for old in json.load(open(prior_path)).get("tasks", []):
            prior[old["id"]] = old
    n = len(ch["tasks"])
    for i, (title, files) in enumerate(ch["tasks"], 1):
        old = prior.get(str(i), {})
        tasks["tasks"].append({"id": str(i), "title": f"[{ch['code']}] {title}", "done": bool(old.get("done")), "doneAt": old.get("doneAt"), "doneBy": old.get("doneBy"),
                               "files": files, "verify": (f"bash {d}/verify.sh" if i == n else None), "notes": old.get("notes")})
    open(os.path.join(d, "tasks.json"), "w").write(json.dumps(tasks, indent=2) + "\n")

    ver = [f"# Verification — {ch['id']}", "", f"Repository: `{repo['name']}`", f"Depends on: {deps}", "", "## Acceptance criteria", ""]
    ver += [f"- {a}" if not a.startswith("  ") else f"  - {a.strip()}" for a in ch["accept"]]
    ver += ["", "## Verify commands", "", "Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).", "", "```verify"]
    ver += ch["verify"] + ["```", ""]
    open(os.path.join(d, "verification.md"), "w").write("\n".join(ver))

    env = "TLH_ROOT"
    root_default = worktree(ch) if ch["code"] != "A5b" else repo["clone"]
    sh = ["#!/usr/bin/env bash", f"# Generated from verification.md for {ch['id']}. Run by `kbd-apply verify` via tasks.json.",
          "set -euo pipefail", f'TLI_KBD="{KBD}"', f'ROOT="${{{env}:-{root_default}}}"',
          'cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }',
          f'git merge-base --is-ancestor {repo["base"]} HEAD || {{ echo "base does not contain {repo["base"]}" >&2; exit 1; }}' if ch["code"] != "A5b" else "true"]
    def unit(cmd):
        if "trap " in cmd or "srv=" in cmd or cmd.startswith("TLI_") or cmd.startswith("export "):
            return cmd
        return "( " + cmd + " ) || exit $?"
    sh += [unit(v) for v in ch["verify"]] + [f'echo "verify OK: {ch["id"]}"', ""]
    p = os.path.join(d, "verify.sh"); open(p, "w").write("\n".join(sh)); os.chmod(p, 0o755)

for ch in C:
    dist_scope = [s for s in ch["scope"] if s.startswith("dist/") or s.startswith("shared/harnesses/generated") or s.startswith("shared/scripts/generated") or s in ("hooks/hooks.json", "hooks/codex-hooks.json")]
    if ch["repo"] == "mini" and ch["code"] != "A5b" and "dist/plugins/**" not in ch["scope"]:
        ch["scope"].append("dist/plugins/**"); dist_scope.append("dist/plugins/**")
    listed = {f for _, fs in ch["tasks"] for f in fs}
    missing = [s for s in dist_scope if s not in listed]
    if missing:
        title, files = ch["tasks"][-1]
        ch["tasks"][-1] = (title, files + missing)
    write_change(ch)
ids = [c["id"] for c in C]
assert len(ids) == len(set(ids))
titles = [t for c in C for t, _ in c["tasks"]]
print(len(C), "changes;", sum(len(c["tasks"]) for c in C), "tasks")
index = [{"id": c["id"], "code": c["code"], "repo": REPOS[c["repo"]]["name"], "depends": c["depends"], "title": c["title"],
          "tasks": len(c["tasks"])} for c in C]
# Changes whose definitions are no longer in this script keep their files and
# their index entry (read back from spec.md), so the index never loses a change.
import re as _re, glob as _glob
known = {c["id"] for c in C}
for spec_path in sorted(_glob.glob(os.path.join(KBD, "changes", "change-tlh-*", "spec.md"))):
    cid = os.path.basename(os.path.dirname(spec_path))
    if cid in known:
        continue
    spec = open(spec_path).read()
    field = lambda name: (_re.search(rf"^\*\*{name}:\*\* (.+)$", spec, _re.M) or [None, ""])[1].strip()
    deps = _re.findall(r"`(change-tlh-[a-z0-9-]+)`", field("Depends on"))
    repo = field("Repository").strip("`")
    tasks_n = len(json.load(open(os.path.join(os.path.dirname(spec_path), "tasks.json")))["tasks"])
    index.append({"id": cid, "code": field("Plan PR"), "repo": repo, "depends": deps, "title": field("Title"), "tasks": tasks_n})
json.dump(index, open(os.path.join(KBD, "phases", PHASE, "change-index.json"), "w"), indent=2)
