# Claude Desktop handoff — issues-194-195-196

Continue the existing approved work in the SAME working tree:
`/Users/gqadonis/Projects/prometheus/prometheus-skill-system`.

The user explicitly asked to hand off to **Claude Desktop**, not Claude Code.
This is a manual advisory checkpoint, not a native harness acceptance receipt.
The current `kbd-handoff` helper supports claude-code/codex/kimi-code/opencode
and refuses open task boundaries; do not rename this as a clean native transfer.
No adopted agent-team state exists; no competing team/task state was created.
Codex stopped implementation. Both source implementation workers hit usage-limit
errors before returning final reports. Re-read files and runtime; do not trust
an unfinished worker message as acceptance evidence.

## Read first

1. Root AGENTS.md, including implementation-first/integration-only and local-only policy.
2. `.kbd-orchestrator/position-reminder.txt`, then live `prometheus kbd --path . status --json`.
3. This packet's JSON snapshot and open obligations.
4. `.kbd-orchestrator/phases/issues-194-195-196/plan.md`, `execution.md`, `backend-task-map.json`, `owner-approval.json`, and `adopted-contract.json`.
5. The four linked OpenSpec changes' proposal/design/spec/tasks/verification.
6. Source files listed in the accompanying production identity manifest; current source is UNBUILT and UNTESTED.

## Approved scope and authority

Issues #194: duplicate change identity diagnosis, prevention and signed/history-preserving repair.
#195: selected VLBV method/contract/authority propagation and recovery.
#196: bounded discovery, author-only JS/Rust BDD and authorized complete-value-phase verification.
Four ordered changes; 16 exact backend IDs (14 production, two EVIDENCE).
OpenSpec exposes task IDs `1`–`4`, NOT heading labels `1.1` etc.
Owner explicitly approved three calibration examples, 180-minute/240-call final
verification/repair budget, up to two repair attempts and selected local macOS
source/generated/scratch-installed surfaces. Prior plan default two no-progress
events remains in the reviewed plan. This budget is final-boundary-only, not a
claim of total development cost. Approval is recorded; do not ask for it again.
Existing explicit Execute request authorizes continuation within this scope.
No push/publication/deployment, global install, protected-scenario mutation,
new paid/remote agent or credential change is authorized by this handoff.

Contract SHA-256: `c6c2db6d12c9588a9213fe7f53efbd316cab2155c432c45a814307ebbf0f5734`.
Bound planRevision: 19. Do NOT edit bound content; material changes use existing
signed amendment authority. Actual incident #194 journal/1.11.3 binary is absent;
no live incident repair or incident-root-cause claim. Representative integration
fixtures later prove only their declared scope.

## Exact checkpoint

Canonical revision 158; Execute is in_progress, lifecycle ready. No Execute completion or phase closure is claimed. Conflicts at capture: 0.

| Change | Canonical task state |
|---|---|
| kbd-194-196-delivery-qualification | 1: pending, 2: pending, 3: pending, 4: pending |
| kbd-194-change-identity-repair | 1: complete, 2: in_progress, 3: pending, 4: pending |
| kbd-195-vlbv-policy-propagation | 1: complete, 2: complete, 3: complete, 4: complete |
| kbd-196-value-scenario-workflow | 1: complete, 2: in_progress, 3: pending, 4: pending |

Open task obligations: `task:kbd-194-change-identity-repair/2` and
`task:kbd-196-value-scenario-workflow/2`. Keep their original start receipts.
#194/#196 change and current phase obligations also remain open. A historical
phase-readiness-contract obligation is carried from prior work: disclose it,
never close it merely to make this handoff look clean. Snapshot contains all.

## What is on disk

- **#194 partial**: new `substrate/kbd-runtime/src/change_identity.rs`; runtime lib/project-document wiring; CLI main clap variants and commands/kbd.rs; early reconcile diagnostics. Task1 complete; task2 source drafted but still in_progress. Alias/apply integration and operator docs tasks3/4 not complete. No build/typecheck/test run.
- **#195**: canonical tasks1–4 COMPLETE, implementation complete (one of four changes). New `shared/method-context.mjs`, reference doc and team `phase-policy.mts`; bootstrap/reanchor/path rule changes; team prompt/provenance changes; KBD method authority sections, actual handoff script and lifecycle hook entries; delivery-cadence and `docs/guide/kbd-selected-method-recovery.md`. Compiled team scripts and all generated packages deliberately NOT rebuilt. Completion is authored production only, NOT behavior acceptance.
- **#196 partial**: `skills/testing/value-scenario-discovery/SKILL.md` and `value-scenario-verify/SKILL.md`; standalone author-only refs in both JS/Rust skills; mode routes added to JS/Rust SKILL.md; discovery sections added after #195 policy sections in Assess/Spec/Plan. Task1 complete. Task2 source authored but its end-task is NOT recorded. Task3 new procedure is drafted but has not been driven/closed as a canonical task. Task4 still needs BDD lifecycle/KBD integration rewrite and operator docs; old failing-test-first/per-change guidance remains there.
- **Delivery**: all tasks pending. No compilation, distribution regeneration, executable acceptance test authoring, integration, refinement or independent review has occurred. Future final integration driver in the contract DOES NOT EXIST yet.

## Resume without losing task/receipt history

First establish Claude Desktop's real local terminal/filesystem/MCP capabilities;
a file-reading-only session cannot execute this phase. No portable session token,
credentials or source permissions transfer. If local tools are available, read
current state and inspect source completeness before using the normal kbd-apply
end-task for an already open task. Do NOT blindly begin the same task again or
fabricate a start receipt. Use the full actual title from backend-task-map.json:

```
KBD_ORCHESTRATOR_ROOT="$PWD/skills/process/kbd-process-orchestrator" \
  bash skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh \
  end-task CHANGE ID INDEX 4 FULL_TITLE
```

The helper resolves exact backend IDs and signed receipts. Only finish a task
whose full source obligation is actually complete. Reconcile a partial/uncertain
transition from observed state before retry. Previously concurrent lifecycle
writes produced a causal-frontier conflict; state was inspected first, then same
idempotent operation replayed in a quiet window. Current captured conflicts are
empty. SERIALIZE ALL future canonical begin/end/decision writes even if source
work is parallel. Never weaken guards to overcome this.

Finish #194 task2, then task3 effective registered task/receipt aliases and task4
docs. Finish #196 task2, drive task3 and complete task4 without overwriting #195
shared stage sections. Keep evidence-only delivery tasks separate from authored
implementation counters. No reviewer/verifier roles before all production source.
The new skill inventory auto-discovers skills under the existing recursive
`skills` root; do not hand-edit generated inventories to register them.

## #194 implementation interface reported by worker (confirm source)

New `change inspect-identity --phase PHASE` emits source map keys and JCS SHA256.
Proposed implemented `change repair-identity --phase --expected-revision
--resolution-file --reason --command-id` is operator-signed typed/runtime work.
Mapping: `{sources:{KEY:SHA256},survivorKey:KEY,targetKey:LOGICAL_ID,
taskMappings:[{sourceKey,sourceTaskKey,targetTaskKey}],
receiptMappings:[{sourceKey:OUTSTANDING_OBLIGATION_KEY,targetSubject:TARGET_OR_TARGET/TASK}]}`.
Original change/task/receipt snapshots are retained. Same command/content is
idempotent; stale/conflicting mapping refuses. All source keys bind the same
embedded logical ID; conflicting task outcomes cannot silently merge. Historical
EventKind replay stays readable; invalid new input is rejected during new-command
preparation rather than retroactively invalidating old signed history.
Opaque legacyCompletionBaseline is preserved, not fabricated; unresolved aggregate
count ambiguity must remain an explicit limitation/blocker for an actual incident.

At FINAL boundary only, a real signed historical damaged-state fixture can be
seeded using kbd-runtime public APIs in a scratch root: Runtime::open_canonical_at,
initialize with Operator actor, append_command LegacyStateImported with Phase
map keys alias-a/target and embedded Change.id=target. This uses real signing/store,
not editing live journals or mocking repair. Append actual Before boundary events
for receipt cases. Worker supplied this recipe; it has NOT been authored/run.

## #195 method context interface

`skills/process/kbd-process-orchestrator/shared/method-context.mjs` exports
`discoverMethodContext/renderMethodContext/writeMethodContext`; CLI `--project
ROOT --phase ID --write [--json] [--require-resolved]`. Derived index is not
contract authority. Selection uses existing signed decision summary `KBD-METHOD `
plus JSON {phaseId,contractDigest,method,retiredMethods,references:[{path,sha256}]}.
Lead recorded `method-vlbv-issues-194-195-196` selecting VLBV with current bound
digest and approved calibration hash, retiredMethods empty (none inferred).
Do not edit contract to add this selection. New helper has not been run as a
validation probe. It surfaces policy conflicts and instruction-only limits.

## Final boundary and limits

Complete ALL planned production before executable test authoring/build/test/lint/
format/probe/import or implementation review. Only then compile team outputs,
regenerate distributions twice deterministically, build CLI with ONE machine-wide
Cargo/rustc process, and author/run the exact finite integration packet. Preserve
separate target directories, use actual real collaborators/installer/runner/native
host observations; unit/mock/helper-only/text-only evidence does not pass.
Missing actual host observation is BLOCKED/NOT_EXERCISED, not invented success.
Safe oracle challenges must exercise actual assertions, not disconnected return-false.
Keep every failure, consumed high-water mark, uncertainty and cleanup receipt.
Use the existing readiness/typed closure writer; bound status changes require
existing signed authority, never manual snapshot editing. Final review remains
pending, not waived. Never mark Execute complete on dispatch or authored N/N.

## Dirty tree preservation

Branch main, HEAD bfe572ff8a97a21c7610c44ea6b39d5f97a857b7 at capture.
`git pull` earlier was already up to date, no unresolved merge entries.
Preexisting changes include generated OpenSpec integrations in multiple harness
folders, staged/unstaged knowledge index/log + many untracked memory captures,
and a modified surreal-memory-server submodule. Do NOT revert/stage/commit those
as part of this work. No commit or push was made. Accompanying git status lists
ALL existing state; production patch covers only tracked selected paths and does
not include new files. Source snapshot ZIP includes selected modified/new sources
and phase artifacts for portability; the actual same-machine tree remains primary.

## Suggested initial Claude Desktop prompt

Continue issues-194-195-196 in /Users/gqadonis/Projects/prometheus/prometheus-skill-system.
Read CLAUDE-DESKTOP-HANDOFF.md and its linked canonical/task snapshots first.
Preserve dirty work and bound contract. Resume the two open task2 obligations
from actual state; finish whole production before any test authoring/build/check/
review. The owner already approved calibration/budgets/surfaces and requested
Execute. Do not rerun planning, ask again for those approvals, fabricate receipts,
modify other products or automatically publish. Report local tool capability if
Desktop cannot access this repository; do not pretend a native transfer occurred.

## Attached checkpoint files

- /Users/gqadonis/Projects/prometheus/prometheus-skill-system/.kbd-orchestrator/phases/issues-194-195-196/harness-handoffs/claude-desktop-20261027T103845Z.json
- /Users/gqadonis/Projects/prometheus/prometheus-skill-system/.kbd-orchestrator/phases/issues-194-195-196/harness-handoffs/claude-desktop-20261027T103845Z-git-status.txt
- /Users/gqadonis/Projects/prometheus/prometheus-skill-system/.kbd-orchestrator/phases/issues-194-195-196/harness-handoffs/claude-desktop-20261027T103845Z-tracked-production.patch
