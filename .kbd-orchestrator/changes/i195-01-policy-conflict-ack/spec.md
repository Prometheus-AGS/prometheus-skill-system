# Governing-policy conflicts block dependent work until acknowledged

Phase: issue-195-closeout · Change: i195-01-policy-conflict-ack

## Why

Issue #195 acceptance 2 requires a governing-policy conflict to be surfaced before dependent execution. Today `method-context.mjs` lists conflicts but `status` (line 128) and `--require-resolved` (line 182) ignore them, so a recovered session can proceed past the conflict. This repository's own CLAUDE.md:102 / AGENTS.md:104 compiler-check exception triggers it once VLBV is selected.

## What changes

- Reuse the existing signed decision surface: an acknowledgement is a decision whose summary is `KBD-POLICY-ACK {"phaseId","path","sha256","issue","note"}`, recorded with `prometheus kbd decision record` and replaced through `--supersedes` (same pattern as `KBD-METHOD`, method-context.mjs:70-80). No new event kind, CLI command or schema.
- A conflict is `acknowledged` only when exactly one current (non-superseded) ack matches phaseId, path and the file's current SHA-256. A changed policy file therefore re-raises the conflict.
- Any unacknowledged conflict sets `status: "CONFLICT"` (blockers still win: BLOCKED > CONFLICT > RESOLVED_REFERENCES); `--require-resolved` exits 2 for both BLOCKED and CONFLICT. Each conflict entry gains `acknowledged` and `ackDecisionId`.
- reanchor.sh prints a blocking line naming each unacknowledged conflict; kbd-handoff.sh includes `policyConflicts` in its method summary. references/method-context.md documents the payload, the recording command and the trust model.
- Operator-owned policy files are never edited. Control is described honestly: canonical Decision state carries no actor, so acknowledgement authority equals the existing KBD-METHOD trust model (instruction-level; not a new guarantee).

## Scope

- `skills/process/kbd-process-orchestrator/shared/method-context.mjs`
- `skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh`
- `skills/process/kbd-process-orchestrator/skills/kbd-handoff/kbd-handoff.sh`
- `skills/process/kbd-process-orchestrator/references/method-context.md`

## Constraints

- Recalled lesson: "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — close tasks only with kbd-apply begin-task/end-task.
- Generated outputs: i195-02 task 2 is the named C-01 reconciliation owner (twice-run, byte-identical hashes, drift validators); i195-01 records that owner in its persisted execution evidence (execution.md and the end-task note) and claims no distribution certification.
- Review before archive: i195-03 task 3 runs the independent cumulative-diff review whose receipt covers i195-01 and i195-02; neither archives before that receipt (or a recorded owner waiver).
- Implementation first (CLAUDE.md, highest precedence): no tests, builds or checks before every planned production change in the phase is complete; acceptance is gathered only at the final boundary owned by i195-03.
- Exclusions: #184 exit semantics, #185 trusted tool admission, the operator-authority trust model, a generic shell fence, and any edit to CLAUDE.md/AGENTS.md policy text.
