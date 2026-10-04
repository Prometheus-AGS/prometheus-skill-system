# Completed-boundary CLI operation

On 2026-10-04, the real `cadence.mjs` CLI operated a disposable Git source and durable journal under the local temporary directory. It initialized a run, started an iteration, finalized it as failed, admitted an evidence-linked corrective iteration, finalized that correction as failed, retired only the immediately preceding failure with a SHA-256-linked JSON receipt and explicit authority reference, and admitted a third iteration.

The final `status` and `report` reads showed two failed iterations, a visible `retired` disposition on the second, a third active iteration, and `successfulDeliveries: 0`. The initial gate script asserted a nonexistent `report.successfulDeliveries` field; inspection showed that the field belongs to `status`, and the affected assertion passed when rerun against both actual contracts. No product failure or extra build was involved.

This disposable operation had no publication obligation. Task 5 checks the existing C09.4 publication debt before and after reconciliation of the live C08 candidate. The operation does not certify C08 or mark its canonical KBD tasks complete.

## Live C08 recovery

The first `finish` attempt encountered an observed heap failure while rotating the 5.3 GB active journal. Its state remained at event 1075 with the candidate active and no command result. Rotation was changed to scan event sequence prefixes while streaming raw bytes into gzip; it no longer parses each historical full-state event. Retrying the same command ID completed the one-time archive and finalized iteration `287674dc-c9d7-4c0b-aef8-991f33c5796d` as `failed` with no completion credit.

`failure resolve` then recorded an explicit `retired` disposition against the existing independent gate JSON (SHA-256 `bad49eb712ee6356d7f11a47facfea63d0e74b4cb19677700b47cc2cab6d24d4`). The journal reached event 1082, active iteration became null, successful deliveries remained 5, and the existing publication obligation `3aa9f8fd-78d2-462f-9584-4f934cabd914` remained pending with `reconcile-legacy-publication-evidence` missing. No canonical KBD task was marked complete by this bookkeeping action.
