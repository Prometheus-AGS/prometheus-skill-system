# Assessment: phase-team-learning-hardening

- Assessed: 2026-10-04, against `origin/main` at `e421715` (merge of #145), inspected in a detached scratch worktree. The local main checkout is on `codex/delivery-cadence-recovery` and is 163 commits behind, so it was not used as evidence.
- Machine state: pk/worker 1.11.0, surreal-memory-server 1.10.0, plugin generation `c250d53e` from `e421715`. Cadence iteration 7 finished `success`; the cadence cron is deleted.
- Phase progress: 0 of 0 changes. No changes are registered yet.
- Model preflight: `degraded`, with `distinct_models: 0` and no providers detected. The adversarial vet will probably run harness-native, so the judge may share the producer's model family.

## Summary

The phase has nine goal items. Eight are **NOT MET**, and none of them has partial code on main. One is **PARTIAL**: G3b, recall quality, was better this time; see its row. Every item is a follow-up to a manual workaround or a known gap recorded in the previous reflection. None needs new architecture.

| # | Goal item | Status | Evidence on main |
|---|---|---|---|
| G1a | Priority-ranked MEMORY.md partitioning | NOT MET | `scripts/memory-index-partition.py:11`: "file order is preserved". `plan()` sorts moved lines by `line` (`:124`) and has no rank key. |
| G1b | Codex `generate_memories=false` applied by the installer and checked by the doctor | NOT MET (agent level only) | The agent-team-creator exporter already sets `memories.generate_memories=false` on each generated Codex **agent** (`runtime/src/adapters-local.mts:26`, diagnostic at `:81`). The **user-level** `~/.codex/config.toml` setting is neither applied nor checked: there is no hit in `scripts/`, `tools/prometheus-cli` or `cowork-management`, and outside tests it appears only in scratch configs (`test-team-awareness.sh:522`, `test-subagent-delivery.sh:290`). The doctor has no `memories` check. The main thread still generates memories unless the operator sets it by hand. |
| G1c | Versioned, fail-loud cadence refresh procedure | NOT MET | `.prometheus/cadence/procedures/refresh-skill-pack.sh` is git-ignored (`.gitignore:94` `.prometheus/*`), and a stray `.bak-200117` sits next to it. Its state.json parity fix and its submodule fix exist only on this machine. |
| G2a | `memory-envelope.integration` on a scratch server | NOT MET | `runtime/test-src/memory-envelope.integration.mts:13` defaults to `http://127.0.0.1:23001`, the live service. |
| G2b | Less repeated query embedding in B4 recall | NOT MET | `learning_recall.py:224-230` builds up to 5 scope queries (role, lead, project, user, global). `surreal_candidates` (`:286-300`) runs them **sequentially**, and each is a `POST /api/v1/search` (`:272`) with the same text. *Unverified:* whether the server embeds the query again on each call; server code and latency were not inspected, so analyze must confirm this. All of it runs inside the 2.8 s overall deadline and the 3.5 s watchdog (`subagentstart-learning.sh:29,53`). A timeout logs `timedOut:true` and delivers nothing. |
| G2c | Ledger/tasks reconciliation check | NOT MET | `kbd-apply.sh:681-683`: `mark-done` calls only `b_mark_done`, with no `sync_progress` and no warning. Nothing in `delivery-cadence/scripts` compares `tasks.json` with `progress.json`. |
| G3a | Cortex mirror verified against real Cortex | NOT MET | `test-cortex-mirror.sh:4` uses "a stub MCP stdio server". Real Cortex **2.0.3** is installed on this machine (`~/.claude/plugins/cache/cortex/cortex/2.0.3`), but it is uninitialised: SessionStart prints "Run /cortex:setup". |
| G3b | Recall quality in `prior-context.md` | PARTIAL | This phase's `prior-context.md` recalled 10 lessons. At least 8 apply directly: mark-done ledger drift, deploy worktrees, submodule update, the real-service test rule, polling after issue create, paired emitter/validator, the post-merge head check, and reentrant locks. The 4 "pk knowledge" entries were off-topic (KnowMe, MCP 2026-07-28, avatars, Actix gateway). Nothing measures relevance; this is one sample. |
| G3c | `rebase-regenerate` helper | NOT MET | No helper exists. The generators that would be chained are `scripts/generate-harness-adapters.js` and `generate-skill-system-distribution.js`, and the `npm run build:codex` mirror. |

## Recalled lessons that shape this assessment

- "`kbd-apply mark-done` updates the task flag but does **not** synchronize the canonical KBD ledger." This is G2c's root cause, confirmed at `kbd-apply.sh:681-683`. The fix belongs in the driver, either a warning or a sync, not only in a cadence check.
- "`delivery-cadence` freezes every source named in `ready`; do **not** point those sources at the actively edited main checkout." and "run `git submodule update` before invoking `update-skill-pack.sh`". Both encode fixes that exist today only in the unversioned procedure (G1c). Versioning it is how these lessons become enforced.
- "Avoid reentrant lock-taking CLIs and default fallbacks." This is the `--auto` parity bug. G1c's "fail-loud" requirement means a versioned procedure must exit non-zero instead of defaulting N=1.
- "For every external integration, run at least one test against the real service". This defines G3a's acceptance: a stub-only Cortex test does not close it.
- "Verify main includes PR final head after merge". This applies to every PR this phase merges. The plan should make it a step at each change boundary.

## Knowledge gaps (from `prior-context.md`)

1. `memory-index-partition.py` keeps file order; the live partition needed a manual priority reorder. Maps to G1a.
2. Codex memory generation is disabled by hand only; a fresh machine will regrow `memory_summary.md`. Maps to G1b.
3. `refresh-skill-pack.sh` is local and unversioned. Maps to G1c.
4. `memory-envelope.integration.mjs` publishes to the live `:23001`; it flaked twice at load ~250. Maps to G2a.
5. The D3 Cortex mirror has been tested only against a stub; real Cortex 2.0.3 has no tag field. Maps to G3a.
6. SubagentStart delivery is skipped when recall misses the 3.5 s watchdog; B4 embeds the same query several times. Maps to G2b.
7. `kbd-apply mark-done` leaves the canonical ledger stale and nothing warns. Maps to G2c.

These are engineering gaps the phase closes, not topics to study. `/learn-goal` is available for any of them, for example `/learn-goal "surreal-memory search API and embedding reuse"` before G2b, or `/learn-goal "Cortex 2.0.3 MCP schema"` before G3a. None has been run.

## Additional findings (not in the seeded goals)

1. **The Codex memories directory still holds 180 KB of generated memory.** `~/.codex/memories/MEMORY.md` (182,777 B, last modified 14:03 local) and `raw_memories.md` (97,336 B) remain. Only `memory_summary.md` was archived. Nothing has grown since the setting was applied at 19:31Z, so generation is off. Open question for analyze: does Codex still *read* these files while `use_memories` keeps its default? G1b should decide whether the installer archives them too and sets `use_memories`.
2. **The live MEMORY.md index is back at 4,088 B,** just under the 4 KB budget it was partitioned to (3,974 B) earlier today. Two memory files were added since. Without an automatic, ranked partition (G1a), the budget will be exceeded within a session or two.
3. **The learning queue holds 2 records in `memory/accepted`,** plus a `quarantine-20261003-accepted-stuck` manifest from yesterday. The #118 `learning.queue` doctor check reports these as in-flight or stale. Worth one doctor run during execute; not a goal.
4. **Untagged pk entries bypass project scoping.** `learning_recall.py:403-404` (`pk_allowed`) returns `True` for any untagged entry, so legacy and doc ingests from other projects pass the filter. This is the likely cause of the off-topic "pk knowledge" block in G3b, and it belongs in G3b's scope.
5. **Every `shared/` and `skills/` edit triggers regeneration.** Tracked generated mirrors under `dist/plugins/{claude,codex}/` and the hook bundles must be regenerated in the same change (constraint C-01; `check:distribution`). That applies to G1a, G2b, G2c and G3c. It is also why G3c matters: these mirrors caused most of last phase's rebase conflicts.
6. **The main checkout sits on a stale branch,** `codex/delivery-cadence-recovery`, 163 behind main. Assessment and execution should use worktrees off `origin/main`, as last phase did.

## Open questions for analyze / plan

- G2b: does surreal-memory 1.10.0's `/api/v1/search` accept a precomputed query embedding, or a multi-scope query in one call? If it does, B4 can embed once. If not, the options are parallel requests or a server-side change in `surreal-memory-server`, which would be a cross-repo change.
- G3a: is initialising real Cortex (`/cortex:setup`) on this machine acceptable as the test environment, or must the test install a scratch Cortex? The repository's test isolation rule (scratch HOME) points to a scratch instance started from the installed 2.0.3 package.
- G1c: where does a versioned procedure live? Options are `skills/process/delivery-cadence/procedures/` (shipped) or `scripts/cadence/` (repo-only). The `.prometheus/cadence/` copy then becomes generated or symlinked.
- G2c: should the guard be a warning in `kbd-apply mark-done`, an automatic sync, or a reconciliation check at the cadence or reflect boundary? Possibly all three.
- G3b: what is a measurable acceptance criterion? For example: on a fixed set of phase goals, at least N of the top-K recalled lessons are judged applicable, and the "pk knowledge" block applies the same project filter as "Lessons".

## Adversarial review

The review ran harness-native: the gateway had no distinct judge model (`JUDGE_MODEL_COLLISION`), and isolation was `same-family-harness-native`. Verdict **PASS**: 0 critical, 3 warnings, 3 suggestions (`review/assess/findings.json`). All three warnings were folded in above:
- G1b's agent-level setting;
- G2b's re-embedding claim, now marked unverified;
- `dist/` regeneration (finding 5).

The suggestions are also folded in: corrected line citations and the untagged-pk finding (finding 4). The reviewer noted that `reviewer-mandate-artifact.md` was not at the path given to it (the mandates live under `assets/`) and that it reviewed against `output-contract.md`. That is a gap in the review's own setup, so treat this as a weaker review.
