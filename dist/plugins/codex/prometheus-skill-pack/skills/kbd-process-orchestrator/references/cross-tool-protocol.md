# Cross-Tool Reporting Protocol

The contract for every tool executing a KBD-owned change is the same: read
canonical status, use typed lifecycle commands, and drive each task through
`kbd-apply`. The append-only runtime journal is authoritative. Progress and
waypoint files are generated views, never a coordination database to hand-edit.

## Completion dimensions

Use the active phase's `completion.implementation` for its N/N counter;
`changes_completed/changes_total` are compatibility aliases. Run-wide totals
belong to `runCompletion` and do not certify the active phase. Evidence,
certification, authorization, external/time-bound gates, and publication remain
independent. A receiving tool must not infer a code gap from pending evidence
when implementation is complete, or claim phase certification from run totals.

## Start and task boundaries

1. Read the waypoint, canonical runtime status, the active phase plan, and the
   selected change spec. Resolve pending work from canonical tasks rather than
   blindly replaying `exactNextCommand`.
2. Register missing work through typed `prometheus kbd change register` and
   `task register` commands using the plan's IDs and sequence.
3. Run `kbd-apply begin-task <change> <id> <i> <n> <title>` before each task.
   Derive i/n from the actual task surface; do not invent totals.
4. Implement exactly that task, then run `kbd-apply end-task` with the same
   task identity. The driver records canonical state, regenerates projections,
   and emits task hooks and progress signals. Never increment counters manually.

## Change completion (B2)

1. Record implementation completion through a typed KBD change transition
   once the implementation is complete. The compatibility completion helper
   delegates to the runtime for generated ledgers; it is not permission to
   edit JSON. Pending evidence never reopens implemented work.
2. Nothing else runs at change completion: no QA, no review, no tests, no
   build, no OpenSpec verify or archive. `openspec validate --strict` (a static
   check of the change's spec documents) is the only permitted command.
3. Review and commit the intended artifacts and projections under project
   policy. Git provides review/history, not a lock for concurrent writers.

## Phase boundary (B4)

When every planned change of the phase is implementation-complete, run the B4
sequence exactly as `references/boundaries.md` defines it: admit, freeze, run
the adopted boundary packet once, record `boundary-results.json` (with
`executedBy` naming this harness), one cumulative QA and independent review,
close with `kbd-phase-close-eval.sh`, and only after CLOSED run `kbd-apply
verify` then `kbd-apply archive` for each change. OpenSpec and native backends
both stay behind this driver; never invoke the bare `openspec-apply-change`
skill. A harness whose profile cannot execute the packet hands the phase off
before B4 instead of running part of it.

## Blockers and handoff

Record blockers with `prometheus kbd blocker record` and clear resolved ones
with `blocker clear`. Retain the pending work and report the blocker rather
than writing status arrays or fallback commands into projections. Include the
canonical phase/change/task IDs and evidence locations in the handoff. Before
handoff, inspect current status and the consistency checks required by the
project; do not claim checks that have not run.
