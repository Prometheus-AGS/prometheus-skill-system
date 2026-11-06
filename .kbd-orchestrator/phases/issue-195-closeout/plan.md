PLAN: issue-195-closeout
Project: prometheus-skill-pack (6ac090a4-3656-4d83-8eb6-2891508196d5)
Date: 2026-10-27
OpenSpec available: NO for this phase (backend native-kbd per project.json specBackend)
Changes to implement: 3

## Owner decisions adopted at this stage (AskUserQuestion, 2026-10-27)

| Input | Answer |
|---|---|
| Risk acceptance | n=1 samples accepted; Codex/OpenCode may be NOT_EXERCISED; Claude session scenarios must run (operator scratch Claude login is a boundary prerequisite) |
| Budgets | 180 min / 240 tool calls / 2 repair attempts / no-progress 2; tokens UNKNOWN |
| Acknowledgement | Signed `KBD-POLICY-ACK` decision bound to the policy file SHA-256 |
| Review route | Labelled per-run judge override (glm-5.3 via liter-llm), recorded `unverified-producer-unknown` |
| Goal B form (2026-10-30) | Owner approved the plan "one identical core in all 11 stage skills" plus the single reference file and an equality check (`check:selected-method`) — the adapted form is accepted; the gap is removed as drift (byte-identical by check), not only guarded |
| Closeout scope (2026-10-30) | Owner approved full closeout with evidence for #195 and #196, the scratch Claude login, and posting the acceptance-to-evidence mappings and closing #195/#196 |

## Recalled lessons that change ordering, scope or risk

- "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — every task closes through `kbd-apply begin-task` / `end-task`; reflect checks `prometheus kbd status` against `tasks.json`.
- "A CRITICAL shipped despite all per-change gates passing … the installer never applied the Codex memory setting" — REF-001 and PKG-001 observe an installer-dispatched scratch generation, not source text.
- "Use rebase-regenerate.sh for Generated-Only Rebase Conflicts" — i195-02 regenerates `dist/**`; resolve generated-only conflicts with that script.
- "Treat One-Build Guard BLOCKED Phase Gates as Rerunnable" — a one-build-guard BLOCKED at the boundary is rerun after the competing build, not counted as PASS or FAIL.

CHANGE LIST (ordered)
1. i195-01-policy-conflict-ack: governing-policy conflicts block dependent work until a hash-bound acknowledgement
   - Scope: orchestrator helper (JS), bootstrap hook (bash), handoff script (bash), reference doc
   - Depends on: NONE
   - Recommended agent: Claude Code lead (sequential)
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: frontier
   - Customer value: HIGH
   - Library: `kbd-decision-policy-ack` (adopt) — reuses the KBD-METHOD signed-decision pattern; no new event kind
   - C-01: i195-01 records i195-02-single-method-reference as its named reconciliation change in execution.md and its end-task note, and is not distribution-certified until i195-02's reconciliation receipts land.
   - Details: `method-context.mjs` resolves current `KBD-POLICY-ACK` decisions, adds `CONFLICT` status and exit 2 under `--require-resolved`; `reanchor.sh` and `kbd-handoff.sh` show unacknowledged conflicts; `references/method-context.md` documents the payload and trust model.

2. i195-02-single-method-reference: one identical selected-method core in all 11 stage skills
   - Scope: 11 stage SKILL.md files, new check script, package.json, generated dist/** and SKILLS.md
   - Depends on: i195-01-policy-conflict-ack (the core names CONFLICT and KBD-POLICY-ACK)
   - Recommended agent: Claude Code lead (sequential)
   - Est. complexity: S
   - Complexity score: Low
   - Model class: medium
   - Customer value: MEDIUM
   - Library: `method-context-reference` (adapt)
   - Details: replace 11 copies with the canonical core from the spec; add `check:selected-method`; regenerate and commit tracked outputs. Named C-01 reconciliation owner for both 01 and 02 source edits. Goal B satisfied in the owner-accepted adapted form (single reference + identical inline core + equality check).
   - C-01 reconciliation (task 2, before phase certification): run `npm run build:distribution` and `npm run generate:skills-index` twice, one at a time; record the SHA-256 of every tracked generated output after each run in `evidence/reconciliation.json`; the two hash sets must be byte-identical; then run `check:distribution`, `check:skills-index` and `validate:harness-adapters`. A mismatch is a BLOCKED change, not a warning. The boundary packet requires `evidence/reconciliation.json` as a receipt.

3. i195-03-delivery-qualification: author and run the closeout evidence; close honestly
   - Scope: session driver and new scenario library (test code), evidence, review, closure
   - Depends on: i195-01-policy-conflict-ack, i195-02-single-method-reference (all production complete)
   - Recommended agent: Claude Code lead (sequential); independent review by non-Claude judge
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Library: `kbd-194-196-session-driver` (adopt), `opencode-headless` (adapt), `liter-llm-env-override-judge` (adopt); build_required items: driver PROMETHEUS_PLUGIN_ROOT isolation, scenario bodies
   - Details: task 1 authors driver isolation, OpenCode harness with isolation preflight, VER-001/VER-003 bodies and CON/REF/SRF scenarios; task 2 runs the single boundary packet; task 3 cumulative cross-model review, strict validation, distribution check, protected-test certification; task 4 typed closure, partial disposition of issues-194-195-196, post the acceptance-to-evidence mappings on #195 and #196 and close both (posting and closing approved by the owner 2026-10-30; recorded in decision-log.md). OpenCode isolation for SRF-002: scratch XDG_CONFIG_HOME/XDG_DATA_HOME plus OPENCODE_DISABLE_CLAUDE_CODE=1, else UNSUPPORTED/NOT_EXERCISED.

TASK MODEL ASSIGNMENTS

Harness: Claude Code (this session; producer model claude-opus-5-5). Discovery 2026-10-27: Agent tool exposes model overrides sonnet/opus/haiku/fable; liter-llm gateway reachable (`:4000` keyed, `:8181` openai-proxy), glm-5.3 exercised successfully twice this phase via `dispatch-judge.sh`; gpt-5.5 via openai-proxy returned HTTP 400. No workers are launched by this plan.

| Phase path | Change ID | Backend task ID | Requirements | Provider/model | Reasoning effort | Rationale and dated evidence | Harness and route | Worker launch and handoff | Native alternative | Availability and verification | Prerequisites |
|---|---|---|---|---|---|---|---|---|---|---|---|
| issue-195-closeout | i195-01-policy-conflict-ack | 1 | JS logic over signed decisions, hashing, exit semantics; medium uncertainty; 1 file | anthropic/claude-opus-5-5 | session default | Authored the method-context analysis this phase (2026-10-27); same-session context avoids handoff loss | Claude Code native | Lead session in repo root; kbd-apply begin/end-task | claude-sonnet-5-5 | configured and in use | none |
| issue-195-closeout | i195-01-policy-conflict-ack | 2 | bash 3.2-compatible output + doc prose; low uncertainty; 3 files | anthropic/claude-opus-5-5 | session default | Same session; small edits | Claude Code native | Lead session | claude-sonnet-5-5 | configured and in use | none |
| issue-195-closeout | i195-02-single-method-reference | 1 | Mechanical identical edit across 11 files | anthropic/claude-opus-5-5 | session default | Byte identity matters more than model strength; lead keeps one copy of the core | Claude Code native | Lead session | claude-haiku-5-5 | configured and in use | i195-01 done |
| issue-195-closeout | i195-02-single-method-reference | 2 | Small Node check script; run generators (one at a time) | anthropic/claude-opus-5-5 | session default | Generators are deterministic local scripts | Claude Code native | Lead session | claude-sonnet-5-5 | configured and in use | i195-02/1 done |
| issue-195-closeout | i195-03-delivery-qualification | 1 | Test authoring across session driver, closure adapter, OpenCode isolation; high uncertainty | anthropic/claude-opus-5-5 | session default | Requires the full assess/analyze context (isolation gaps found here) | Claude Code native | Lead session | claude-sonnet-5-5 | configured and in use | all production tasks done |
| issue-195-closeout | i195-03-delivery-qualification | 2 | Run the boundary packet once; batch fixes | anthropic/claude-opus-5-5 (driver); claude -p sessions inside the driver use the scratch login's default model | session default | Driver spawns scratch sessions itself | Claude Code native + scratch CLIs | Integration driver from repo root | none | Claude scratch login UNRESOLVED until operator logs in | operator scratch Claude login; optional Codex/OpenCode logins |
| issue-195-closeout | i195-03-delivery-qualification | 3 | Independent cumulative-diff review; must not be Claude-family | zai/glm-5.3 | provider default (reasoning control unknown) | Exercised at assess/analyze/spec 2026-10-27, verdicts parsed, anti-theater gate PASS | liter-llm REST gateway `:4000` via `dispatch-judge.sh` with `PROMETHEUS_KBD_JUDGE_*` override | `dispatch-judge.sh --mode diff`; findings JSON to review/final/ | MiniMax-M3 (configured judge, refused as judge==critic) | verified 2026-10-27; label unverified-producer-unknown | secrets.env loaded; gateway up |
| issue-195-closeout | i195-03-delivery-qualification | 4 | Canonical closure commands; outward-facing posting gated | anthropic/claude-opus-5-5 | session default | Typed CLI commands only | Claude Code native | Lead session | claude-sonnet-5-5 | configured and in use | explicit owner authorization to post on #195 |

EXECUTION ROUND ORDER
Round 0: lead session adopts the contract with `kbd-contract-adopt.sh`, records the KBD-METHOD decision, registers changes/tasks, writes the identity-bound plan handoff (see Adoption record)
Round 1: i195-01-policy-conflict-ack (tasks 1, 2)
Round 2: i195-02-single-method-reference (tasks 1, 2)
Round 3: i195-03-delivery-qualification task 1 (test authoring, only after rounds 1–2 complete)
Round 4: i195-03 task 2 (single boundary run) → task 3 (review/certification) → task 4 (closure)

Sequential throughout: one lead, shared files (i195-02 regenerates outputs for i195-01 edits), and the implementation-first policy forbids interleaving checks.

## Contract binding

- Path: `.kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json`
- Digest: computed by `kbd-contract-adopt.sh` with `jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)'`; stamped into the contract and recorded in the plan handoff identity. See "Adoption record" below.
- Freshness/evidence invalidation: source tree digest, contract digest, fixture manifest SHA-256 per scenario; only affected evidence is invalidated.
- Budgets: 180 min, 240 tool calls, 2 repair attempts, no-progress 2, tokens UNKNOWN; prior lifecycle consumption UNKNOWN and retained.
- Worker routes: lead Claude Code session; judge glm-5.3 via liter-llm (labelled).

## Scenario → task coverage

| Scenario | Owner task | Session? |
|---|---|---|
| CON-001, CON-002 | i195-01-policy-conflict-ack/1 | no |
| CON-003 | i195-01-policy-conflict-ack/2 | no |
| POL-002 | i195-01-policy-conflict-ack/2 | Claude (required) |
| POL-001, POL-003 | i195-02-single-method-reference/1 | Claude (required) |
| REF-001, PKG-001 | i195-02-single-method-reference/2 | no |
| VER-001, VER-003 | i195-03-delivery-qualification/1 | Claude (required) |
| SRF-001 | i195-03-delivery-qualification/1 | Codex (NOT_EXERCISED accepted) |
| SRF-002 | i195-03-delivery-qualification/1 | OpenCode (NOT_EXERCISED accepted) |

Coverage note: the owner task is where the behaviour under each scenario is implemented; i195-03/1 implements the scenario *bodies* specified by the i195-01/i195-02 specs. Every scenario outcome (CON/POL/REF/PKG/VER/SRF) is demonstrated only in the Round 4 boundary packet; Rounds 1–2 run no checks.

## Boundary execution packet (one, post-implementation only)

- Commands (repository root, in order):
  1. `node scripts/check-selected-method-core.mjs`
  2. `node scripts/generate-skill-system-distribution.js --check`
  3. `node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence`
- Expected effects: owned scratch resources only; session scenarios without a scratch login are BLOCKED (Claude) or NOT_EXERCISED (Codex/OpenCode), never PASS.
- Required approvals: this contract's adoption; operator scratch Claude login; separate explicit authorization to post on #195.
- Receipts: evidence/reconciliation.json (C-01 twice-run hashes), evidence/candidate-and-fixtures.json, evidence/attempts.jsonl, evidence/<scenario-id>/, evidence/cleanup.json, review/final/, closure-report.md.
- Cleanup: retain evidence first; remove only manifest-owned resources; never touch real user roots.
- Stop conditions: incomplete production or stale contract; isolation unavailable; uncertain effect or failed cleanup; budget or no-progress limit.

## Adoption record

Round 0 (lead session): `bash shared/scripts/kbd-contract-adopt.sh .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json` stamps `identity.contractDigest`; the digest and canonical planRevision are written into the plan handoff identity (`KBD_HANDOFF_IDENTITY_JSON`). The digest is appended here after adoption.

## Method decision and fixture prerequisites

- Real phase: a signed `KBD-METHOD` decision selecting VLBV, bound to the adopted contract digest, is recorded immediately after adoption under the owner's 2026-10-30 approval of the full closeout. It is a prerequisite of i195-01 task 1's dependent work (method-context resolution), not of authoring.
- Scenario fixtures: CON-001/CON-002/CON-003 run in an owned scratch project whose fixture writes its own synthetic signed `KBD-METHOD` decision (bound to the fixture contract digest) and its own fixture policy file; they never depend on, read or modify the real phase's decisions or operator policy files.
- REF-001 is a no-session scenario: inside the boundary driver, after owned scratch resources exist, it dispatches `scripts/install-plugin-generation.js` from each generated payload (`dist/plugins/{claude,codex}/prometheus-skill-pack`) into a scratch `HOME`/`PROMETHEUS_PLUGIN_ROOT`, then compares the canonical core bytes across source, both payloads and the installed generation.

## Spec artifact verification (2026-10-30)

`.kbd-orchestrator/changes/i195-0{1,2,3}-*/{spec.md,tasks.json,verification.md}` all exist (9 files, generated by `gen_i195_specs.mjs`); the spec handoff listed only i195-01's spec as its output. Registration below uses the backend task IDs from each `tasks.json`.

COMMANDS TO RUN
Changes already authored at Spec under `.kbd-orchestrator/changes/i195-0{1,2,3}-*` (generator `gen_i195_specs.mjs`); register them and their backend task IDs through `prometheus kbd change register` / `task register`.

PLAN COMPLETE
