ASSESSMENT: issue-195-closeout
Project: prometheus-skill-pack (project 6ac090a4-3656-4d83-8eb6-2891508196d5)
Date: 2026-10-27
Codebase baseline: main @ 6bb66d5c (includes #197 VLBV propagation, #200 impeccable v4.8.0, #202 zeespec replay fix); installed prometheus 1.13.2 built from identical kbd-runtime/CLI source.
Cross-tool progress: none — new phase, 0 of 0 changes registered.

## Scope and provenance

Follow-up to `issues-194-195-196` (contract `c6c2db6d…`, plan revision 19), whose repair budget (2 attempts) is consumed and which remains OPEN. Goals A–D come from the owner-approved `/plan` (2026-10-27, "yes"), recorded in `goals.md`. Issue #195 is OPEN. This assessment is read-only: no source edits, builds or tests were run for it.

IMPLEMENTATION STATUS
- Method-context helper (`skills/process/kbd-process-orchestrator/shared/method-context.mjs`): DONE for discovery; PARTIAL for issue acceptance 2 — conflicts are detected but never change `status` (line 128: `status: blockers.length ? 'BLOCKED' : 'RESOLVED_REFERENCES'`), so `--require-resolved` (line 182) exits 2 only on blockers, never on a conflict.
- Conflict detection scope: PARTIAL — runs only when `method === 'VLBV'` (line 100) and scans only `AGENTS.md`, `CLAUDE.md`, `.kbd-orchestrator/constraints.md`. This repository's own `CLAUDE.md:102` / `AGENTS.md:104` ("Use an intermediate package-scoped compiler check only when an observed compiler error blocks…") match the regex, so the conflict WILL fire here once VLBV is selected. Today the phase has no method decision, so `policyConflicts` is `[]` and status is `BLOCKED` for "adopted contract unavailable".
- Recovery surfaces (reanchor.sh, kbd-handoff/pause/resume, agent-team `phase-policy.mts` / `handoff-provenance.mts`): DONE — carry method/contract references; reanchor already prints `policyConflicts` from the derived snapshot.
- Selected-method section in stage skills: DONE but duplicated — `## Selected-method recovery and authority` appears in 11 stage SKILL.md files (kbd-assess, child-exit, execute, handoff, init, new-child, next-child, pause, plan, resume, spec). Two byte variants exist (8 vs 3), differing only by one trailing blank line — no semantic drift yet. A canonical target already exists: `skills/process/kbd-process-orchestrator/references/method-context.md`.
- VER-001 / VER-003: STUB — `scripts/tests/lib/kbd-194-196-sessions.mjs:314-316` registers them as `stub(...)` that always yield NOT_EXERCISED. They were never implemented, not merely unrun.
- POL-001/002/003, SCN-001/002: DONE in the prior phase (attempts 13, 14, 16 PASS, n=1 each, Claude Code only).
- PKG-001, IDR-001..003: DONE (attempt-17 ship recheck PASS).

CROSS-TOOL PROGRESS
- NONE — no cross-tool activity recorded for this phase.

SPEC GAP SUMMARY
- Gap A (acceptance 2): a governing-policy conflict is advisory only. Needs a status that blocks dependent execution and an operator acknowledgement (with provenance) that clears it, without rewriting operator rules.
- Gap B (acceptance 5, drift risk): 11 hand-copied sections; replace with a one-line pointer to `references/method-context.md` (or a generated include) and regenerate distributions.
- Gap C (acceptance 4 and 6): VER-001 (real repair/resume operation via `/value-scenario-verify` with oracle-sensitivity challenge) and VER-003 (canonical closure, six distinct claims, no automatic successor) have no driver implementation.
- Gap D (acceptance 5, surfaces): Codex and OpenCode never exercised. Both CLIs are installed now (codex-cli 0.160.1, opencode v2.2.0); scratch logins do not exist.
- Gap E (environment): scratch session homes `~/.prometheus-scratch/kbd194/{claude,codex}` no longer exist, and the driver (`kbd-194-196-sessions.mjs`) defines no OpenCode scratch home at all. Every session scenario (POL-*, SCN-*, VER-*) needs the operator to log in to a fresh scratch Claude config; Codex and OpenCode attempts each need their own operator login in a scratch config home (`CODEX_HOME`, and an OpenCode config/data root under `~/.prometheus-scratch/`) so an attempt cannot read or write the operator's real configuration. The driver never types credentials.
- Gap G (cross-model review feasibility, Goal D): liter-llm is installed and its gateway responds (`:4000` with key, `:8181` openai-proxy). Observed during this stage's own review: (1) `models.toml` sets `judge` and `critic` to the same model (MiniMax-M3) with no `backup`, so `dispatch-judge.sh` refuses (`JUDGE_MODEL_COLLISION`, exit 4); (2) `gpt-5.5` through openai-proxy fails with HTTP 400 ("not supported when using Codex with a ChatGPT account"); (3) `glm-5.3` succeeded via per-run env override (`PROMETHEUS_KBD_JUDGE_*`); (4) the producer's canonical identity is not configured, so findings record `cross_model_check: unverified-producer-unknown`. A verified-distinct review therefore needs either a `backup` role or distinct judge/critic in `models.toml` (owner-owned config) plus producer identity, or acceptance of the env-override route with the unverified label.
- Gap F (governance): prior phase OPEN with exhausted repair budget; it needs an honest typed disposition (partial) referencing this phase rather than silent abandonment.

BUILD HEALTH
- build check: UNKNOWN for this phase (none run; production-first policy). Context: #202 gate passed (kbd-runtime replay tests 3/3, `prometheus-cli --test kbd` 8/8).
- known violations: NONE observed in scope.
- test coverage: PARTIAL — 8 of 12 adopted scenarios PASS from prior phase; VER-001/VER-003 stubs; Codex/OpenCode NOT_EXERCISED.

CONSTRAINT CHECK
- AGENTS.md / CLAUDE.md violations: NONE. Note: their compiler-check exception is itself the policy conflict Gap A must surface (not rewrite).
- constraints.md violations: NONE observed.

GOAL PROGRESS
- Goal 1 (follow-up framing): MET — phase created at revision 2426 with goals recorded.
- Goal A (conflict blocks): NOT MET — see Gap A.
- Goal B (single reference): NOT MET — 11 copies.
- Goal C (VER-001/VER-003, rerun POL-002/PKG-001): NOT MET — stubs; scratch login absent.
- Goal D (Codex/OpenCode, cross-model review): NOT MET — not attempted; preflight `degraded` (one distinct non-Claude model, MiniMax-M3, as judge and critic; no backup).
- Goal E (typed closure, #195 evidence comment): NOT MET — pending all above.

## Recalled lessons that apply

- "A CRITICAL shipped despite all per-change gates passing … `install-skills-flat.sh` never applied the Codex memory setting" — PKG-001 and Gap D must check the installer path actually dispatched per surface, not only the source tree.
- "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — close tasks only through `begin-task`/`end-task`; verify `prometheus kbd status` before reflect.
- "Treat One-Build Guard BLOCKED Phase Gates as Rerunnable" — a BLOCKED from the one-build guard is not a pass and not a failure; rerun after the competing build ends.
- "Use rebase-regenerate.sh for Generated-Only Rebase Conflicts" — Gap B regenerates `dist/` mirrors; resolve generated-only conflicts with that script.

## Knowledge gaps (from prior-context; offer `/learn-goal <topic>`, not run)

- `scripts/memory-index-partition.py` keeps file order instead of ranking entries.
- Codex memory generation disabled by hand; installer neither sets nor checks it.
- `.prometheus/cadence/procedures/refresh-skill-pack.sh` is local and unversioned.
- `agent-team-creator/tests/memory-envelope.integration.mjs` publishes to live `:23001`.
- D3 Cortex mirror tested only against a stub server.
- SubagentStart delivery skipped when recall misses the 3.5 s watchdog.
- `kbd-apply mark-done` leaves the canonical ledger stale with no warning.

None of these block this phase.

## Additional observations (out of scope, recorded)

- `refresh-native-plugin-installs.sh:254-258` compares the Codex source path without resolving symlinks; fails when invoked through `~/Projects` (symlink to `/Volumes/T7`). No issue filed.
- `prometheus-exec` reproducible-build hash mismatch on rustc 1.99.1; sources unchanged. No issue filed.
- A stale empty `.git/index.lock` appeared twice this session with no git process running; source unidentified.

## Adopted-outcome contract inputs (elicitation)

| Input | Value |
|---|---|
| Useful user operation | A KBD operator in a fresh or delegated session recovers the selected method and contract, and is stopped before dependent execution when governing policy conflicts with it, until the operator acknowledges the conflict. |
| Observed baseline | Conflicts listed but non-blocking; 11 duplicated sections; VER-001/003 stubs; Codex/OpenCode UNKNOWN; scratch logins absent; prior lifecycle budget consumption UNKNOWN (preserved, not reset). |
| Desired observable result | `method-context --require-resolved` exits 2 on an unacknowledged conflict and 0 after a recorded acknowledgement; one canonical section; VER-001/VER-003 produce PASS/FAIL observations (not NOT_EXERCISED); Codex/OpenCode each PASS or honestly UNSUPPORTED/NOT_EXERCISED; #195 evidence comment. |
| Supported environments | macOS (darwin), Node v26, Claude Code; Codex 0.160.1 and OpenCode 2.2.0 best-effort. |
| Required safety checks | Scratch isolation (`HOME`/`CODEX_HOME`/OpenCode config+data roots/`PROMETHEUS_PLUGIN_ROOT`/data dir scratch; memory URLs at a dead port); no credentials typed by agents; no writes to real journals; protected-test certification at final boundary. |
| Exclusions | #184 exit semantics, #185 trusted tool admission, operator-authority trust model, generic shell fence, rewriting `CLAUDE.md`/`AGENTS.md` policy text. |
| Risk acceptance | **UNKNOWN — owner input needed** (e.g., accept n=1 session samples; accept Codex/OpenCode NOT_EXERCISED if logins are unavailable). |
| Budgets | **UNKNOWN — owner input needed** (prior defaults: 180 min, 240 tool calls, 2 repair attempts, no-progress limit 2). |
| Decision owner | Repository owner (Travis James). |

## Value scenario discovery (proposals, not adopted)

Reuse unchanged from the prior contract: POL-002 (rerun, since Gap A changes its behavior), PKG-001 (rerun after Gap B regeneration), VER-001, VER-003. New candidates:

- **CON-001** — Unacknowledged conflict blocks: VLBV selected with this repo's `CLAUDE.md` compiler exception → status `CONFLICT`, `--require-resolved` exit 2, reanchor prints it. Reason now: acceptance 2. Likelihood: certain in this repo.
- **CON-002** — Acknowledgement clears without rewriting policy: a signed operator acknowledgement → status resolves; `CLAUDE.md` bytes unchanged; a changed `CLAUDE.md` hash re-raises the conflict. Reason now: avoids a permanently blocked repo.
- **REF-001** — Single-source section: all stage skills resolve the same canonical text in source, generated `dist/`, and the installed generation. Reason now: acceptance 5.

Deferred: multi-sample (n>1) reliability, OpenCode hook semantics beyond observation.

## Issue #195 acceptance → goal mapping

| #195 acceptance (issue text) | Prior evidence | This phase |
|---|---|---|
| 1. Fresh and delegated sessions recover the same method/contract/value/exclusions/stage authority | POL-001 PASS (attempt 13, n=1, Claude) | none required |
| 2. No premature generated-code checks on planning/task/hour/stale-command/reviewer triggers; governing-policy conflict surfaced **before dependent execution** | POL-002 PASS (attempt 16); conflict advisory only | Goal A, CON-001/002, POL-002 rerun |
| 3. Compaction/resume preserves supersession, freshness, budgets, unresolved effects | POL-003 PASS (attempt 16) | none required |
| 4. Completed phase reaches separately authorized real-path verification; authored/measured/demonstrated/release kept distinct | VER-001 stub | Goal C (VER-001) |
| 5. Source, generated and installed surfaces preserve semantics; versions/identities and unsupported surfaces reported | PKG-001 PASS (Claude payload); Codex/OpenCode not exercised | Goals B and D, REF-001, PKG-001 rerun |
| 6. Closure produces no automatic successor; optional findings do not expand the contract | VER-003 stub | Goal C (VER-003), closure |

## Unresolved calibration inputs

1. Risk acceptance and budgets (above).
2. Acknowledgement mechanism: a signed canonical decision (`KBD-POLICY-ACK`) via existing decision events (recommended: no new event kind) vs a file marker.
3. Whether the operator will log in to fresh scratch Claude (required for VER/POL), Codex and OpenCode homes (optional) at the boundary.
4. Cross-model review route (Gap G): fix `models.toml` (distinct judge/critic or a `backup`, plus producer identity) vs accept the per-run override labelled `unverified-producer-unknown`.

## Adversarial review of this assessment

Judge glm-5.3 via liter-llm gateway (per-run override; `cross_model_check: unverified-producer-unknown`); verdict PASS, 0 CRITICAL / 2 WARNING / 1 SUGGESTION; anti-theater gate PASS (score 0.0). All three findings were incorporated above (Gap E OpenCode isolation, Gap G, acceptance mapping). Findings: `review/assess/findings.json`.

ASSESSMENT COMPLETE
