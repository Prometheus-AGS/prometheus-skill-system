PLAN: deep-research-onyx-parity › stage-10-hang-investigation
Project: prometheus-skill-pack
Date: 2026-09-12
OpenSpec available: NO (native-kbd) — `openspec/` exists at the repo root, but backend resolution is per-change (`backend_detect` checks the change id's own directory shape first, so an unrelated `openspec/` cannot shadow it), and this phase's changes live under `.kbd-orchestrator/changes/` (drt-001..007 precedent, drt-008/009 authored and adversarially reviewed at the spec stage)
Changes to implement: 3
Producer model: glm-5.3 (opencode session; corrected from the k3 label the handoff-in assumed)

CHANGE LIST (ordered)

1. change-drt-008-hang-capture-harness: instrumented repeat-run capture harness isolating the intermittent stage-10 hang
   - Scope: tests (new `skills/research/deep-research/tests/hang/` only; no existing file edited)
   - Depends on: NONE
   - Recommended agent: Claude Code (main session — live process-tree capture and teardown discipline)
   - Est. complexity: S
   - Complexity score: Low (3 tasks, single layer, no new abstraction; follows the existing suite conventions)
   - Model class: small
   - Customer value: HIGH — the only path to Goal 1's blocked-command trace and Goal 2's dating, and the measured failure rate that sizes change 2's certification
   - Details: loop `driver-contract.sh --scenario full-run` up to K=12 (or first hang) with scoped PID-only cleanup, per-run 90 s timeout (external-wrapper attribution — the suite has no internal timeout), timestamped logs, load-average run records; on timeout capture the ps tree, `sample` stacks, and output-volume observation BEFORE teardown. HANG-CAPTURE.md carries the run table, hangs/K rate, the captured command/stack if hung, and the goal-2 dating entry with a `method:` line. Library: cand-2 (adopt, analysis D2) — the harness reuses the existing driver rather than building any new runner; cand-1's cleanup/independence instrumentation is embedded here.

2. change-drt-009-stage10-hang-fix-and-certification: Goal-4 machine-checked diagnosis review → conditional fix → N-run certification
   - Scope: tests + adversarial-review scripts (conditional) — `tests/hang/DIAGNOSIS.md`, `tests/hang/FIX-CERTIFICATION.md`, conditionally `skills/process/adversarial-review/scripts/build-review-packet.sh` + `.py` companions
   - Depends on: change-drt-008
   - Recommended agent: Claude Code (review orchestration + careful conditional extraction)
   - Est. complexity: M
   - Complexity score: Medium (3 tasks but crosses the research-tests ↔ adversarial-review tooling boundary; judgment-heavy gate and branch discipline; extraction itself has 9× prior art in-tree)
   - Model class: medium
   - Customer value: HIGH — delivers Goals 3 and 4; the proof standard that makes "fixed" mean something for a ~1-in-3 intermittent defect
   - Details: task 1 gates everything on a machine-checked adversarial review of the diagnosis — concretely: `review/diagnosis/findings.json` records `cross_model_check: verified-distinct` (or `judge_model != producer_model`) AND the sycophancy screen's PASS record exists; the same-model fallback does NOT satisfy Goal 4; an unreachable judge ⇒ BLOCKED, no fix. Task 2 applies the fix the reviewed diagnosis warrants within Scope — extraction branch (byte-identical companions), elsewhere branch (in-Scope), or `escalated` when the diagnosis names an out-of-Scope file. Task 3 certifies N = ceil(ln 0.01 / ln(1-p)) ≥ 10 **consecutive** clean full-run passes (machine-countable `| run` rows) plus one full driver-contract suite pass; **streak semantics**: a single unclean run resets the streak to zero — and if a failure pattern suggests run-to-run correlation (shared-state residue the cleanup missed), that is recorded and the certification gate goes BLOCKED rather than being retried away. Library: cand-1 (adopt, D1) and cand-3 (conditional adapt, D3).

3. change-drt-011-phase-close-distribution-regen: C-01 reconciliation — regenerate dist for every skill tree this child touched
   - Scope: build outputs — `dist/plugins/**` regeneration (no source edits)
   - Depends on: change-drt-008, change-drt-009 (all skills/** edits of this child must land first)
   - Recommended agent: OpenCode
   - Est. complexity: S
   - Complexity score: Low (1 task, deterministic generator run + checks)
   - Model class: small
   - Customer value: MEDIUM — phase certification precondition; dist currently still carries the old heredoc `export-package.sh` and lacks this child's new files
   - Details: run the distribution generator twice → byte-identical hashes; `npm run check:distribution`, `npm run validate:codex`, `npm run check:skills-index` all PASS locally; recorded per the C-01 deferral contract that change-drt-009's Constraints named this change as owner.

EXECUTION ROUND ORDER
Round 1: change-drt-008 (no dependencies)
Round 2: change-drt-009 (depends on Round 1)
Round 3: change-drt-011 (depends on Rounds 1–2)
No parallelism: 009's gate consumes 008's evidence, and 011 must follow every skills/** edit it mirrors.

COMMANDS TO RUN (native-kbd; changes 1–2 exist, reviewed at spec stage; 3 is authored at this plan stage)
mkdir -p .kbd-orchestrator/changes/change-drt-011-phase-close-distribution-regen  # spec.md/tasks.json/verification.md below
# Driving (one task per turn): kbd-apply begin-task <change> <id> <i> <n> "<title>" → implement → end-task

SCOPE CUTS AND TRADE-OFFS (explicit, not friction-free)
- **change-drt-010-child-phase-packet-resolution is CUT from this child's plan** (round-1 CRITICAL: it advances none of the child's four goals — it fixes tooling this child's *spec vet* tripped over). It is recorded as a standalone/parent-phase change to run AFTER this child closes: the fix itself is a ~4-line CHANGES_ROOT derivation change in `build-review-packet.sh` (strip `/children/<child>` before the double-dirname); it is sequenced after the child deliberately — the same file is drt-009's conditional-extraction target, and no edit to the suspect file should precede Goal-2 dating. Until it lands, the phase-local symlink bridge at `.kbd-orchestrator/phases/deep-research-onyx-parity/changes/` remains in place and is retired by that change, not this child.
- scoring-graph.sh's pre-existing stall (closed-port `semantic-blocked` scenario) is OUT: it predates this child, reproduces on pre-edit code, and joining it would widen the child's goals without instruction. It still blocks any "all suites green" claim and stays recorded as parent-phase debt.
- Goal-2 dating is bounded by the uncommitted tree: if the implicated code exists only in this phase's uncommitted work, "first appearance" is provable only as "introduced by this phase's uncommitted work", not as a date. The dating method is recorded on a `method:` line in HANG-CAPTURE.md with exactly three legal values — `commit-pass` (a commit boundary exists to diff against), `merge-base` (blame against the branch merge-base, limitations documented), `uncommitted-only` (no committed history can contain the code). The plan does not promise a date the evidence cannot support.
- If drt-008's campaign yields zero hangs in K runs, p is bounded (0/K), not measured (~1/3); drt-009 must then size N from the bound with an `inferred` label — the plan surfaces this rather than assuming the hang will cooperate.
- Strictly serial execution costs wall-clock; it buys clean diagnosis evidence and a conflict-free edit order.
- K=12 sizing rationale: at the observed ~1/3 rate, P(zero hangs in 12 runs | p=1/3) = (2/3)^12 ≈ 0.8%, so K=12 is near-certain to reproduce at least once if the rate holds; it also bounds a clean campaign near 20 minutes. If the true rate is far lower, the campaign bounds it and the escalation below applies.
- The new harness script is bash 3.2 compatible and runs under `/bin/bash` (C-05), `set -euo pipefail`, like every script this phase has shipped.
- drt-009's diagnosis-review evidence path (absolute): `.kbd-orchestrator/phases/deep-research-onyx-parity/children/stage-10-hang-investigation/review/diagnosis/findings.json`.

## Unresolved review findings (round 2 BLOCK accepted at the 2-round cap — inherited by Execute verbatim)

> Judge: MiniMax-M3 vs producer glm-5.3, cross_model_check verified-distinct, sycophancy screen PASS.
> The judge's change-id spellings ("drate-…") are reproduced verbatim; they refer to change-drt-008/009/011.

1. **CRITICAL** — "The certification formula N = ceil(ln 0.01 / ln(1-p)) is mathematically undefined when p=0, and the plan's zero-hang branch does not resolve it."
   Disposition for Execute: when the campaign observes 0 hangs in K runs, size N from the Clopper-Pearson 95% upper bound p_ub = 1 − 0.05^(1/K) (rule-of-three ≈ 3/K) instead of the undefined p̂=0 plug-in, and label p `inferred (0/K, p_ub=…)`. This resolution is guidance inherited through this section; it was not re-vetted (2-round cap reached).
2. **CRITICAL** — "drate-008 violates C-01 because it does not name drate-011 as its reconciliation-change owner, even though drt-008 adds files under a regenerated surface."
   Disposition: the ownership IS recorded — change-drt-008's spec Constraints name the phase-close distribution change (via change-drt-009's Constraints) as the C-01 reconciliation owner. A plan-mode packet carries plan.md, goals, prior handoffs and constraints by design, not the change specs, so this could not be seen from the packet. No change required; cited here so Execute inherits the pointer.
3. **CRITICAL** — "Goal 1 has no defined path when drate-008 observes zero hangs in K runs."
   Disposition for Execute: zero hangs in K runs ⇒ Goal 1 is recorded BLOCKED (rate bounded at 0/K), and the campaign escalates once — K doubled to 24, wall-clock ceiling ~40 min — before the child accepts a bounded-not-measured outcome. A second zero-hang campaign leaves Goal 1 BLOCKED with the evidence boundary stated; it does not invent a diagnosis.

Also carried (WARNINGs, by the gate contract): the K-loop's stop-at-first-hang vs full-K ambiguity (drt-008's harness must state which — resolution for Execute: run ALL K when no hang occurs, since p needs the denominator; when a hang occurs, capture it and CONTINUE the campaign if wall-clock remains, marking post-capture runs as potentially perturbed); the `escalated` branch's destination (defined in drt-009's spec: a new change specced at the PARENT plan stage — outside this child); correlation detected only reactively (accepted — pre-Run correlation controls are the cleanup step; the reactive record is the honest floor); "byte-identical companions" unexplained in plan (defined in drt-009's spec: extracted `.py` diffed empty against the replaced heredoc body).

PLAN COMPLETE
