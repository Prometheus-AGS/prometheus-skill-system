# KBD continuity and progress-memory evidence

Date: 2026-09-19
Change: `repair-kbd-progress-continuity`
Recovery task: `skill-pack-1-10-recovery/release-skill-pack-1-10-recovery/recovery-2`
Source commit at verification: `1e18a9b5422cc5b125e581f511497d2e147ebfae`
Recovery commit: `1e18a9b5422cc5b125e581f511497d2e147ebfae`

## Delivered behavior

- Canonical task state derives active and next change/task identity after phase,
  change, and task folds.
- `exactNextWork` remains visible operator intent and is no longer a selector in
  status, reminders, renderers, gates, or apply routing.
- Task boundaries use qualified change/task identity.
- Change boundaries open before their first observed task and close only after
  the final task commits both task and change guard receipts.
- Nested phase activation, top-level reset, restart, and resume regenerate the
  same cursor in canonical state and compatibility projections.
- `karpathy-progress-memory` validates versioned boundary events against
  canonical KBD status, appends an idempotent session record, uses project-scoped
  `pk` ingestion, and falls back to the existing durable memory outbox.
- Stable hook event identity excludes unrelated KBD revisions and wall-clock
  replay time; concurrent writers serialize on an event lock, `pk` is bounded
  by a timeout, and likely GitHub, Slack, AWS, and OpenAI secrets are rejected.
- Task, change, and phase completion recording occurs after the corresponding
  terminal transition; task recording also follows its signed postcommit guard.

## Commands and observed results

| Command | Exit | Observed result |
|---|---:|---|
| `openspec validate repair-kbd-progress-continuity --strict` | 0 | Change is valid. |
| `npm run validate:strict -- skills/process/karpathy-progress-memory` | 0 | 1 skill validated; no errors or warnings. |
| `python3 skills/process/karpathy-progress-memory/tests/progress-memory-integration.py` | 0 | 12 production-entry scenarios passed: run-scoped identity, dynamic-state replay, concurrency, pre/post-`pk` crash recovery, bounded timeout, secret rejection, outage, restart, mismatch rejection, and projection non-mutation. |
| `RUSTC_WRAPPER= cargo check --locked --manifest-path substrate/kbd-runtime/Cargo.toml` | 0 | `kbd-runtime` compiled successfully. |
| `RUSTC_WRAPPER= cargo check --locked --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli` | 0 | `prometheus-cli` compiled successfully. |
| `RUSTC_WRAPPER= cargo fmt --manifest-path substrate/kbd-runtime/Cargo.toml --all -- --check` | 0 | Runtime source and integration target formatted. |
| `RUSTC_WRAPPER= cargo fmt --manifest-path tools/prometheus-cli/Cargo.toml --all -- --check` | 0 | CLI source and integration target formatted. |
| `RUSTC_WRAPPER= cargo test --manifest-path substrate/kbd-runtime/Cargo.toml --test position_continuity` | 0 | 1 passed, 0 failed. |
| `RUSTC_WRAPPER= cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd` | 0 | 8 passed, 0 failed. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-hooks.sh` | 0 | 14 hook checks passed. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-task-completed-gate-e2e.sh` | 0 | Incomplete qualified task was blocked with the expected signed-receipt requirement. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-apply-boundaries.sh` | 0 | Final task opened and closed task/change receipts in order, then emitted exactly one task and one change completion hook. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-next-phase.sh` | 0 | A reflected complete phase was not transitioned or recorded twice before successor activation. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-position-sync.sh` | 0 | 9 checks passed. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-progress-semantics.sh` | 0 | 6 checks passed. |
| `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-waypoint-path.sh` | 0 | 8 checks passed. |
| `bash shared/scripts/tests/test-position-render.sh` | 0 | 47 passed, 0 failed. |
| `bash shared/scripts/tests/test-position-stop-gate.sh` | 0 | 19 passed, 0 failed. |
| `bash shared/scripts/tests/test-position-on-prompt.sh` | 0 | 6 passed, 0 failed. |
| `git diff --check -- <affected paths>` | 0 | No whitespace errors. |

The first root-level Cargo attempt found an active writer in the separate
`prior-auth` checkout and then failed because this repository has no root
`Cargo.toml`; no source compiled. A later helper logged a newly started external
writer but did not abort before the CLI check. That CLI check was interrupted,
the helper was changed to exit `75` on any writer, and the CLI check was rerun
from the beginning after the external writer cleared. All reported passing
Cargo results came from the corrected serial runs. Swap remained high: the
passing checks observed 24,594.44 to 24,857.50 MiB used of 25,600 MiB.

## Defects found during acceptance

1. The first progress-memory integration run counted lines in a multiline fake
   `pk` payload as invocations. The harness now counts the unique scoped-source
   argument prefix; the unchanged recorder then passed all assertions.
2. The existing ambiguous-authority CLI test corrupted a folded checkpoint.
   Checkpoints are now explicitly non-authoritative caches, so replay correctly
   ignored that corruption. The regression now corrupts the canonical Loro
   project document and proves precommit evaluation fails closed, writes one
   atomic recovery receipt, and leaves the signed journal unchanged.
3. Six renderer assertions still encoded the former contract in which stale
   operator text selected work. They now assert derived `nextChange`/`nextTask`
   output and prove OpenSpec/Spec Kit operator commands cannot override it.
4. The first isolated critic found eight production defects: unreachable change
   completion recording, revision-unstable event identity, concurrent/crash
   duplicate submission risk, unbounded `pk`, repeated-ID receipt ambiguity,
   duplicate phase completion, stale non-null cursor reuse, and incomplete
   secret detection. The repaired artifact has a real change boundary, semantic
   receipt identity with serialization, bounded memory submission, qualified
   receipts, guarded phase completion, canonical cursor synchronization, and
   expanded secret regressions.
5. The expanded restart test exposed that replay time and recorder-owned files
   changed the semantic payload even when the event ID was stable. `observedAt`
   is now receipt metadata, and recorder-owned session/receipt/outbox paths are
   excluded from hook touched-file discovery.

## Review

Isolated artifact critic round 1: **BLOCK** with eight production findings. The
artifact was repaired and reverified.

Isolated artifact critic round 2 (final permitted round): **BLOCK**. It found
three progress-memory durability defects:

1. Hook event identity omits `runId`, so a successor run that reuses the same
   phase/change/task IDs collides with the prior run.
2. Replay recalculates `touchedFiles`, `commitSha`, and `exactNextWork`; changes
   to unrelated Git or canonical state can make the stable ID fail payload
   comparison.
3. A crash after writing the provisional receipt but before memory submission
   leaves an incomplete receipt that suppresses both `pk` and outbox delivery
   on replay.

The operator resumed the blocked task. All three defects were repaired without
a third critic round, as required by the two-round review limit:

1. Canonical `runId` is part of event identity and the explicit event schema.
2. Receipts retain the original durable event and stable identity digest, so
   later Git, waypoint, or exact-next-work changes replay the original result.
3. A provisional receipt no longer suppresses delivery. Replay resumes memory
   delivery while holding the event lock, then atomically records completion.

The 12-scenario integration suite above exercises each repair. The typed KBD
blocker `recovery-2-progress-memory-durability` has a recorded resolution and
`recovery-2` is back in progress.

## Open verification

Generated skill indexes, manifests, documentation, and `dist/` remain
unregenerated by design. The recovery plan performs one generation pass after
the unique source commits are integrated onto current `origin/main`.
