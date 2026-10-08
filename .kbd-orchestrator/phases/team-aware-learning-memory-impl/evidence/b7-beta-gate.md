# B7 beta gate: cross-agent awareness (change-tli-b7-cross-agent-awareness-beta)

Branch `feat/team-awareness` (worktree `/Users/gqadonis/Projects/prometheus/worktrees/tli-b7`), based on `origin/feat/subagentstart-delivery` (B5) with `origin/feat/file-tier-reduction` (B6 work-ahead) merged. pk 1.10.0 first on PATH; scratch surreal-memory 1.10 (embedded, MLX) and the 1.10 learning worker.

## verify.sh (TLI_ROOT=worktree), rc=1

| Step | Result |
|---|---|
| prerequisites (surreal 23001 up, pk >= 1.10, claude + codex) | pass |
| `test-team-awareness.sh --harness both` | `test-team-awareness: 10 passed (harness: both)`, rc 0 |
| `report-learning-delivery.py --require-reduction` (real file tier) | FAIL rc 1: BLOCKED on B6 task 3 (MEMORY.md partition awaiting user approval) |
| `npm run check:distribution` | pass (run separately: verify.sh stops at the previous step under set -e) |

Model runs (both harnesses): `api-dev=BETA-LEAD BETA-PRIV BETA-ROUTED`, `ui-dev=BETA-ROUTED`; the ui-dev child context carried the BETA-PRIV digest line (path + hash, no text).

## Per-agent bytes (SubagentStart channel, delivery.jsonl)

| Harness | Agent | subagentstart | file-tier (scratch) | total | baseline |
|---|---|---|---|---|---|
| claude-code | api-dev | 922 B | 0 | 922 B | 14,336 |
| claude-code | ui-dev | 866 B | 0 | 866 B | 14,336 |
| codex | api-dev | 922 B | 0 | 922 B | 11,059 |
| codex | ui-dev | 866 B | 0 | 866 B | 11,059 |

## --require-reduction against the REAL file tier (not faked, partition not run)

- Claude Code, this project's `MEMORY.md`: 15,580 B, NOT below 14,336 B (a subagent with nothing delivered still loads it).
- Codex `~/.codex/memories/memory_summary.md`: 16,061 B, NOT below 11,059 B.
- Adding the 866-922 B hook payload to either only widens the gap. The step passes once the B6 partition (<= 4,096 B index) is applied to the real MEMORY.md and the Codex memory summary is reduced.
- Other checks in the same step pass: with an empty file tier every agent is below baseline; baselines of 10 B and a 20 KB MEMORY.md both fail as required (asserted inside the gate).

Other regression run: `test-learning-write.sh` 6 passed.

## Follow-ups closed

Branch `feat/team-awareness`, commits `300971a` (paths from SubagentStop), `d78ee4d` (SessionStart main-thread view), `3cc2c56` (tests), `f8d6a87` (regenerated dist). pk 1.10.0 first on PATH; scratch surreal-memory 1.10.0; no cargo run.

**(a) Routing from a live SubagentStop.** `subagentstop-learning.sh` now passes `paths` to `learning_write`: a trailing `paths: a, b` or `(paths: a, b)` on a LESSON/GOTCHA/DECISION line, else the Write/Edit/MultiEdit `file_path` values in the subagent's own transcript (`agent_transcript_path`; Codex `transcript_path`), repo-relative, outside-repo dropped, capped at 20. Cycle 1 now drives one real SubagentStop payload through the generated entry: BETA-PRIV (suffix) stays private, BETA-ROUTED (transcript Write) goes to ui-dev, BETA-LEAD (`(paths: ...)`) goes to lead; the hook stays silent with rc 0.

**(b) Main-thread delivery.** New generated hook `sessionstart-learning` (group `sessionstart:learning`, both harnesses, timeout 5): `--main-thread` recall (`@lead` plus team digest), fenced, 8,000-char Claude / 7,000-char Codex budget, 3.5 s watchdog, plain-text stdout (kbd-open's channel; must not start with `{`), silent for subagent payloads, outside a team, and when nothing is recalled. kbd-open does not recall team lessons, so nothing is injected twice. Hook matrix count 65 -> 67.

Gate output (exact lines):

- `test-team-awareness: 11 passed (harness: both)` rc 0 (previous run: 10 passed)
- `claude output: api-dev=BETA-LEAD BETA-PRIV BETA-ROUTED ui-dev=BETA-ROUTED`; `claude-code main thread: 859 chars, scopes ['tlm-fixture/@lead', 'tlm-fixture/@team']`
- `codex output: api_dev=BETA-LEAD BETA-PRIV BETA-ROUTED ui_dev=BETA-LEAD BETA-ROUTED`; `codex main thread: 859 chars, scopes ['tlm-fixture/@lead', 'tlm-fixture/@team']`; `codex: ui_dev inherited the parent's main-thread view: True`
- `test-learning-write: 6 passed` rc 0
- `test-subagent-delivery: 5 passed (harness: none)` rc 0
- `node scripts/tests/hook-dispatch.test.mjs` rc 0; `npm run validate:harness-adapters` rc 0 (`claude-code=34, codex=33`, bundle f21cc7f3...); `npm run check:distribution` rc 0 (`PASS: 217 canonical skills, payload parity, modes, pins, manifests, and marketplaces`); `npm run validate:codex` rc 0
- `report-learning-delivery.py --require-reduction` (real file tier) rc 1, as expected: `FAIL: codex (any subagent: file tier only): 16061 B is not below the 11059 B baseline` (this worktree's Claude MEMORY.md path is absent, 0 B; the project's own MEMORY.md was 15,580 B before). Still blocked on the B6 partition.

**Finding (needs a decision).** Codex forks the parent thread's history into a spawned child, so the main-thread view the parent receives at SessionStart (including the `@lead` line, BETA-LEAD) is also in every child's context. ui_dev therefore reports BETA-LEAD on Codex; Claude Code does not (ui-dev=BETA-ROUTED). The gate keeps the strict check on what the hooks deliver: ui_dev's own SubagentStart block has neither BETA-LEAD nor BETA-PRIV, the inherited block is the parent's verbatim and carries no role-private text. If the design wants ui-dev never to see lead text on Codex, the Codex main-thread view must drop the `@lead` lines (digest only); that is a design change, not made here.

**Test deviations.** (1) Codex parent-thread assertion "parent receives no injection" became "no role-scoped injection, and the team view is present". (2) `test-subagent-delivery.sh` "store stopped, nothing to deliver" step now also isolates `PROMETHEUS_TEAM_DIGEST_DIR` (it failed identically at `94fa985`: the B7 digest file is a fallback). (3) `test-learning-write.sh` starts its scratch server with `exec` so the cleanup kill reaches it (it left the :23021 orphan). `lsof -iTCP -sTCP:LISTEN` after the final run: only the installed :23001 service.

### Decision applied: Codex main-thread view is digest-only

Codex forks parent history into every child, so `sessionstart-learning.sh` for harness codex now emits only team digest lines (author, paths, hash), never `@lead` lesson text; Claude keeps `@lead` plus digest. Documented in the script header and `docs/guide/memory-tiers.md`. The strict Codex assertions are restored (ui_dev has neither BETA-LEAD nor BETA-PRIV anywhere in its child rollout, inherited history included; the parent view is digest lines only). Commit `git log -1` on `feat/team-awareness` (bundle `aa292bac...`).

- `test-team-awareness: 11 passed (harness: both)` rc 0
- `codex output: api_dev=BETA-LEAD BETA-PRIV BETA-ROUTED ui_dev=BETA-ROUTED`; `codex main thread: 666 chars, scopes ['tlm-fixture/@team']`
- `claude output: api-dev=BETA-LEAD BETA-PRIV BETA-ROUTED ui-dev=BETA-ROUTED`; `claude-code main thread: 859 chars, scopes ['tlm-fixture/@lead', 'tlm-fixture/@team']`
- `test-learning-write: 6 passed`; `test-subagent-delivery: 5 passed (harness: none)`; `hook-dispatch.test.mjs` rc 0; `validate:harness-adapters` rc 0; `check:distribution` rc 0 (`PASS: 217 canonical skills, ...`); `validate:codex` rc 0.
- Only the installed :23001 service listens after the runs.
