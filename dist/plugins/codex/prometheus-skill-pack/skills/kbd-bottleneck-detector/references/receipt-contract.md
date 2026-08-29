# Receipt contract

Canonical authority is the signed KBD event journal. Waypoint, progress, and
position files are replayable projections and never prove completion by
themselves.

- A successful `before` receipt opens one obligation keyed by boundary kind and
  canonical subject. A duplicate start is blocked.
- A successful `after` receipt closes that obligation. Missing or out-of-order
  completion is blocked and records a typed blocker.
- Receipt identity binds project, run, phase path, change/task/checkpoint,
  boundary edge, and source revision. Phase ordering comes from `PhaseDefined`
  event order; task ordering comes from canonical task sequence.
- Projection repair is safe only when replay leaves the canonical revision
  unchanged. Ambiguous authority permits no canonical mutation and produces an
  atomic recovery receipt under `.kbd-orchestrator/recovery/bottleneck/`.
- Integration and certification gates require complete implementation state.
  Certification also requires closed boundaries, valid task completion
  receipts, a passed integration gate, no unfinished gate, and no unresolved
  blocker.
- `kbd gate run` executes argv directly. A Rust gate refuses to start while any
  other Cargo or rustc process is active on the machine.

The journal retains full history. Folded state intentionally keeps current
obligations and the latest receipt summaries only.

`guard evaluate --json` returns `outcome` (`pass`, `repaired`, or `blocked`),
`authoritativeRevision`, `position`, `findings`, `outstandingObligations`,
`exactSignal`, `repairedProjections`, and `receiptId`. Surface `exactSignal`
verbatim, followed by `Position: <position> @ revision <authoritativeRevision>`.
`status` exposes the same folded obligations and latest receipt/gate summaries.
