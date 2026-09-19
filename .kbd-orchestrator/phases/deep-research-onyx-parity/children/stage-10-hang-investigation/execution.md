EXECUTION: deep-research-onyx-parity › stage-10-hang-investigation
Project: prometheus-skill-pack
Date: 2026-09-12
Selected backend: native-tool (opencode, SELF) driving native-kbd changes via /kbd-apply
Dispatched to: SELF — this session walks each change task-by-task with kbd-apply begin-task/end-task
Backend rationale: changes are native-kbd (drt series precedent; backend_detect routes by change-id shape), the work is process-tree capture and careful gated edits best done in-session, and /kbd-apply owns the per-task hook/progress loop. OpenSpec exists at the repo root but is not the backend for this phase's changes.
Backend entrypoint: kbd-apply.sh begin-task <change> <id> <i> <n> "<title>" → implement one task → end-task
OpenSpec available: YES (repo root) — NOT selected; native-kbd owns this phase's changes
Source plan: .kbd-orchestrator/phases/deep-research-onyx-parity/children/stage-10-hang-investigation/plan.md

EXECUTION SCOPE

- change-drt-008-hang-capture-harness: instrumented repeat-run capture harness (goals 1–2)
- change-drt-009-stage10-hang-fix-and-certification: Goal-4 gated fix + N-run certification (goals 3–4)
- change-drt-011-phase-close-distribution-regen: C-01 distribution reconciliation

DISPATCH CONTRACTS

- change-drt-008-hang-capture-harness → SELF
  Entry: kbd-apply begin-task change-drt-008-hang-capture-harness 1 1 3 "<title>"; spec at .kbd-orchestrator/changes/change-drt-008-hang-capture-harness/
  Model class: small (plan annotation: Low complexity)
  Concrete model: SELF (glm-5.3, frontier generator — executing directly; class annotation kept for routing auditability)
  Model rationale: 3 tasks, single layer, no new abstractions — cheapest viable class; self-execution chosen for process-capture discipline
  Progress file: .kbd-orchestrator/phases/deep-research-onyx-parity/children/stage-10-hang-investigation/progress.json
  Handoff: kbd-apply end-task syncs progress.json + waypoint per task

- change-drt-009-stage10-hang-fix-and-certification → SELF
  Entry: kbd-apply begin-task change-drt-009-stage10-hang-fix-and-certification 1 1 3 "<title>"; spec at .kbd-orchestrator/changes/change-drt-009-stage10-hang-fix-and-certification/
  Model class: medium (plan annotation: Medium complexity)
  Concrete model: SELF (glm-5.3); the Goal-4 gate's judge dispatches to MiniMax-M3 via liter-llm (distinct from producer) — the gate's distinctness is machine-checked, not assumed
  Model rationale: crosses research-tests ↔ adversarial-review tooling boundary; judgment-heavy branch discipline
  Progress file: as above
  Handoff: as above; task 1 BLOCKED semantics per spec (no distinct judge ⇒ BLOCKED, no fix)

- change-drt-011-phase-close-distribution-regen → SELF
  Entry: kbd-apply begin-task change-drt-011-phase-close-distribution-regen 1 1 1 "<title>"; spec at .kbd-orchestrator/changes/change-drt-011-phase-close-distribution-regen/
  Model class: small
  Concrete model: SELF (glm-5.3)
  Model rationale: deterministic generator run + checks
  Progress file: as above
  Handoff: as above

APPROVAL GATES

- drt-009 task 1: adversarial review of the diagnosis with judge distinct from producer (machine-checked: review/diagnosis/findings.json cross_model_check + sycophancy PASS record). Same-model fallback does NOT satisfy Goal 4.
- Per-change artifact-refiner QA (/refine-validate) then adversarial-review --mode diff before archive (see plan handoff and plan.md Unresolved section for inherited guidance: p=0 → Clopper-Pearson bound; zero-hang → BLOCKED + one K-doubling; K-loop full-K-when-clean, capture-then-continue-when-hung).

FALLBACK CONDITIONS

- If in-session execution cannot keep scope bounded (scope-guard warnings on writes outside the widened child scope), fall back to per-task dispatch through a fresh subagent per task with this execution.md as the handoff note.
- If the liter-llm judge gateway cannot produce a distinct-judge review for drt-009 task 1, that task records BLOCKED (not fallback-to-same-model).

VERIFICATION REQUIREMENTS

- Per-change verify strings in tasks.json (all failable by design)
- drt-008: bash -n + /bin/bash -n (C-05), test -x, HANG-CAPTURE.md greps, no driver-contract.sh modification, no orphaned driver processes
- drt-009: distinct-judge jq check, branch: line, ≥10 machine-countable `| run` rows, full suite count
- drt-011: npm run check:distribution && validate:codex && check:skills-index
- All local; no hosted CI (AGENTS.md)

PROGRESS LEDGER

- [PENDING] change-drt-008-hang-capture-harness — SELF
- [PENDING] change-drt-009-stage10-hang-fix-and-certification — SELF
- [PENDING] change-drt-011-phase-close-distribution-regen — SELF

OUTPUTS

- skills/research/deep-research/tests/hang/{run-hang-capture.sh, HANG-CAPTURE.md, DIAGNOSIS.md, FIX-CERTIFICATION.md} (per change)
- Conditionally: extracted .py companions beside build-review-packet.sh
- dist/plugins/** regeneration

BLOCKERS

- NONE (inherited plan-review CRITICALs are dispositioned guidance in plan.md's Unresolved section, not blockers)

REFLECTION HANDOFF

- HANG-CAPTURE.md (blocked command, stack, measured rate, dating) and FIX-CERTIFICATION.md (branch, N, streak table, suite count); review records under review/; the plan stage's unresolved findings and how each was dispositioned at execute time.

EXECUTION READY
