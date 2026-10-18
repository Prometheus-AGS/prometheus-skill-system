# SPEC: phase-readiness-contract

Date: 2026-10-18. Source-bound to assessment.md + analysis.md (same directory). Issues: #186 (vertical slice, primary), #184 (canonical accepted-contract foundation only), #185 (interface dependencies recorded, NOT implemented).

## What is built

A pre-execution readiness path for KBD phases, all shell/JSON/Markdown (zero Rust, zero Cargo):

1. **Adopted contract artifacts** — `references/schemas/adopted-contract.schema.json` (NEW) + `references/templates/adopted-contract.template.md` (NEW) under `skills/process/kbd-process-orchestrator/`.
2. **Adoption writer** — `shared/scripts/kbd-contract-adopt.sh` (NEW): validates contract against schema, computes SHA-256, records `prometheus kbd decision record` (immutable, plan-revision-bound, `--supersedes` for amendments) carrying path+digest+adoption source, installs contract into the phase dir. Canonical writer only; never hand-writes journal/waypoint.
3. **Readiness checker** — `shared/scripts/kbd-phase-readiness-check.sh` (NEW, read-only against canonical state): validates identity (project/run/phase/plan revision), adoption (decision digest == contract digest), scenario coverage (every scenario ID mapped to a registered task), fixture/permission readiness declarations, budgets present, boundary packet complete. Emits exactly `READY` or `BLOCKED` + minimal missing list. Never writes success evidence.
4. **Semantic handoff gate** — extend `references/schemas/handoff.schema.json` (optional `identity` object, backward compatible) and `shared/lib/stage-gate.sh` (schema validation replacing bare `jq empty`; identity match enforced when present; Plan→Execute boundary REQUIRES identity + adoption).
5. **Upstream skill/prompt edits** (minimal diffs): `prompts/assess.md` + `skills/kbd-assess/SKILL.md` (adopted-outcome elicitation), `skills/kbd-analyze/SKILL.md` (readiness mapping + READY/BLOCKED recommendation), `skills/kbd-spec/SKILL.md` (finite scenario matrix), `prompts/plan.md` + `skills/kbd-plan/SKILL.md` (contract identity, coverage map, worker routes, budgets, single boundary packet), `skills/kbd-execute/SKILL.md` + `skills/kbd-apply/SKILL.md` (pre-dispatch readiness gate; READY required; never fabricate PASS), `references/templates/acceptance.template.md` (stable scenario IDs, fixture/permission readiness, status vocabulary, evidence-invalidation keys).
6. **Boundary integration evidence** — `tests/integration/kbd-readiness-gate.sh` (NEW, executable): runs the real scripts + real `prometheus` CLI + real filesystem; scratch fixture trees under `$TMPDIR` for negative controls; the phase's own contract adoption is the real-path dogfood. Run ONCE after all production diffs complete.

## Frozen finite scenario matrix (stable IDs — frozen before implementation per operator rule 2)

| ID | Name | Oracle | Negative control |
|---|---|---|---|
| RC-01 | ADOPT-READY: adopt this phase's own contract → READY | canonical decision exists with matching digest+revision; checker prints READY, exit 0 | checker run pre-adoption prints BLOCKED(adoption) |
| RC-02 | BLOCKED-MISSING: fixture phase without adoption | checker prints BLOCKED listing `adoption` as missing, exit 2 | same fixture WITH adoption → not BLOCKED on adoption |
| RC-03 | REJECT-STALE: fixture with tampered digest / superseded plan revision | rejected, minimal missing list names `contractDigest`/`planRevision`; no evidence written | untouched fixture passes identity+digest check |
| RC-04 | REJECT-IDENTITY: fixture with wrong phaseId/runId | rejected naming `identity`; canonical state untouched | correct identity passes |
| RC-05 | REJECT-MALFORMED: empty/malformed handoff at Plan→Execute | stage gate exits ≠0, no handoff/evidence written | well-formed handoff passes |
| RC-06 | AMEND: amendment decision (`--supersedes`, new digest) | checker accepts new digest, rejects old; `prometheus kbd audit` shows both decisions, history intact | old digest still rejected after amendment |
| RC-07 | SKILL-PACKAGING + INSTALL-PARITY (delivery qualification, NOT a product oracle): modified skills pass repo strict validation AND this phase's changed delivery surface keeps source/generated/installed parity | `npm run validate:strict skills/process/kbd-process-orchestrator` exit 0; generated payload `node scripts/generate-skill-system-distribution.js` reflects the changed sources; REAL installer (`node scripts/install.js --scope user`) with `HOME`/`XDG_DATA_HOME` redirected to scratch installs payloads whose `stage-gate.sh` digest == source digest and whose kbd-execute SKILL.md carries the readiness-gate marker | intentionally broken frontmatter fails validation (dry-run on scratch copy, never committed); scratch install of PRE-CHANGE source lacks the readiness-gate marker (proves the check detects drift, not echoes). Failures here are packaging/verification failures, never cascaded into product-failure claims for RC-01..06 |

Status vocabulary per scenario run: `PASS` / `FAIL` (product: real oracle unmet) / `BLOCKED` (verification: validator/env/fixture unavailable) / `NOT_EXERCISED`. Every scenario records attempted/observed/evidence. Unknown ≠ success; unknown ≠ observed failure.

Digest normalization (frozen): `contractDigest = sha256( jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' adopted-contract.json )` — UTF-8, sorted keys, compact. `identity.contractDigest` is excluded to avoid self-referential recursion; `identity.planRevision` is excluded because the canonical `Decision` already binds `plan_revision` at record time — the writer stamps the field at adoption for readability; the checker compares digest (same normalization) AND decision.plan_revision vs current plan revision (staleness). Writer and checker use the identical normalization command.

Negative-control isolation (frozen): scratch `HOME`, `XDG_DATA_HOME`, scratch canonical project dir (own `.kbd-orchestrator` + `.prometheus/project.json` fixture), scratch harness/skill roots, scratch memory roots. The test NEVER reads or writes live `~/Library/Application Support/prometheus`, live `~/.opencode`, `~/.claude`, or surreal-memory for negative controls; only RC-01/RC-06 adoption writes touch the REAL canonical runtime (that IS the production path being verified).

## Nine-item methodology mapping (operator rules → artifacts)

| # | Rule | Artifact / field |
|---|---|---|
| 1 | Capability inventory → honest release claim | assessment.md (inventory) + contract `releaseClaim` (operation, supported envs, capabilities, UNKNOWNs); plan/release text never uses task counts as capability |
| 2 | Freeze adopted criteria IDs/oracle/exclusions/decision owner pre-production | contract `scenarioMatrix` (stable IDs above), `exclusions`, `decisionOwner`, `adoptionSource`; amendments only via new canonical decision (rule 8) |
| 3 | Minimal sufficient end-to-end evidence per required claim | verification.md per-scenario: production entry, real collaborators, exact command/argv/exit/observed/limitations, fixture identity; no unit/mock/schema-only evidence |
| 4 | product failure vs verification failure vs BLOCKED vs NOT_EXERCISED | status vocabulary (above) in contract `statusVocabulary` + verification.md; readiness checker distinguishes missing-evidence from failed-oracle |
| 5 | Functional vs resilience scope separated; safety checks non-waivable | contract `functionalScope` vs `resilienceScope` (resilience overflow → backlog list); `requiredSafetyChecks` (no second ledger, no historical rewrite, no sandbox expansion, fixture isolation) cannot be dropped by budget exhaustion |
| 6 | Fine-grained evidence invalidation | contract `evidenceInvalidation`: keyed by source digest / plan revision / fixture identity; only affected evidence invalidated; history preserved; no full-repo re-review loops |
| 7 | Bounded budgets + exhaustion semantics | contract `budgets`: timeMinutes=90, toolCalls=180, repairAttempts=2, noProgressLimit=2, tokenBudget=UNKNOWN(fallback: time+tool bounds); exhaustion → checkpoint + BLOCKED, never silent PASS |
| 8 | Scope change requires explicit operator version-adoption | amendment path = new `decision record --supersedes` referencing operator instruction; suggestions never auto-enter task totals or next phase |
| 9 | Boundary verification only after complete production | `boundaryPacket.postImplementationOnly=true`; single integration run at phase boundary; no per-task test loops; budget stop permits saving docs/state without running product tests |

## Functional scope (this phase)

RC-01..RC-07 above; the six production file groups above; canonical registration of this phase's changes/tasks via `prometheus kbd change/task register`.

## Resilience scope (deferred backlog — NOT in this phase)

Host-side enforcement of budgets (#185); automatic evidence re-validation on source drift; install/gate parity for harness targets this phase does NOT change (e.g. Cursor/Devin payload shapes); UAR-specific matrices (explicitly not generic-skill gates). Generated/installed payload parity FOR THE FILES THIS PHASE CHANGES is functional scope, delivered inside RC-07's qualification and RC-01..06's scratch-install execution, not deferrable. Only never-adopted optional items may defer; adopted requirements never move to backlog by budget overflow — budget exhaustion yields checkpoint+BLOCKED instead.

## Exclusions (frozen)

Mini port; #185 implementation; #184 full phase-exit/budget runtime; #172/#176 reconcile rewrite; global tool fence; plugin migration; new services/credentials; push/merge/release; hosted CI as evidence.

## Required safety checks (non-waivable)

Canonical writer only for runtime state; no journal/waypoint hand-edits; negative controls write nothing to real canonical state; fixtures in `$TMPDIR` with no live ports/services; preserve foreign dirty state (`.prometheus/knowledge`, `tools/surreal-memory-server` in main checkout) untouched.

## Budgets (frozen)

time=90min total (consumed at spec time ≈35min), toolCalls=180 (consumed ≈33), repairAttempts≤2, noProgress≤2, token=UNKNOWN+fallback. Worker route: main agent only, kimi-code-plan-global/k3-256k (operator-confirmed subscription, Extra Usage disabled); no subagents (single authorized route).

## Decision owner

Operator (gqadonis). Material scope/acceptance changes require explicit operator version-adoption.
