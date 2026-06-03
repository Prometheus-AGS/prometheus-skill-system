EXECUTION: kbd-execute-spec-wrapping-and-nesting-2026-06-03
Project: prometheus-skill-system
Date: 2026-06-03
Selected backend: native-tool (SELF — Claude Code)
Dispatched to: SELF
Backend rationale: This phase repairs the broken /opsx:apply execute seam (F1).
  Driving the changes through OpenSpec's apply would dogfood the broken path to
  fix the broken path. Native KBD + self-execution keeps the loop under KBD
  control. The work is doc + shell edits to the orchestrator skill itself.
Backend entrypoint: direct edits + shell test scripts under
  skills/process/kbd-process-orchestrator/
OpenSpec available: YES (but intentionally not used as the apply backend here)
Source plan: .kbd-orchestrator/phases/kbd-execute-spec-wrapping-and-nesting-2026-06-03/plan.md

EXECUTION SCOPE (commit order)

- change-005-hooks-robustness: harden hooks.sh resolver + memory-log.sh; tests
- change-001-spec-backend-interface: SpecBackend interface + OpenSpec adapter doc
- change-002-kbd-apply-driver: new kbd-apply skill (the core F1 fix)
- change-003-rewrite-execute-dispatch: route kbd-execute → kbd-apply; kill false claim
- change-004-per-turn-position-reporter: plain-text guarantee + Stop-hook recipe
- change-006-child-loop-wrapping: child loops use kbd-apply; full-chain reporting
- change-008-verify-and-integration-test: e2e proof + validations
- change-007-speckit-adapter: thin Spec Kit adapter (LAST)

DISPATCH CONTRACTS
All changes → SELF (Claude Code). No external tool handoff this phase.
Model class per change recorded in plan.md / progress.json.

APPROVAL GATES
- NONE (orchestrator-internal changes; user reviews at reflect)

FALLBACK CONDITIONS
- If a shell fix cannot be made shell-portable, document the bash-only
  constraint rather than silently shipping a zsh-incompatible guard.

VERIFICATION REQUIREMENTS
- bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-hooks.sh
- bash all other shared/lib/tests/*.sh
- npm run validate:strict for the new kbd-apply skill

PROGRESS LEDGER
- [DONE] change-005-hooks-robustness — SELF
- [DONE] change-001-spec-backend-interface — SELF
- [DONE] change-002-kbd-apply-driver — SELF
- [DONE] change-003-rewrite-execute-dispatch — SELF
- [DONE] change-004-per-turn-position-reporter — SELF
- [DONE] change-006-child-loop-wrapping — SELF
- [DONE] change-008-verify-and-integration-test — SELF
- [DONE] change-007-speckit-adapter — SELF

OUTPUTS
- Edited: shared/lib/hooks.sh, shared/lib/memory-log.sh
- New: skills/kbd-apply/{SKILL.md,kbd-apply.sh}
- New: references/spec-backend-interface.md, references/per-turn-position-hook.md
- Edited: skills/kbd-execute/SKILL.md, prompts/execute.md, orchestrator SKILL.md
- New/edited tests under shared/lib/tests/

BLOCKERS
- NONE

REFLECTION HANDOFF
- Which findings were fully fixed in code vs documented-only.
- The environmental xtrace discovery (root trigger of the jq errors).
- Honest status of the per-turn guarantee (Stop-hook recipe is documented;
  whether it was wired into settings is a user decision).

EXECUTION READY
