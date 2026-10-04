# Completed-boundary CLI operation

On 2026-10-04, the real `cadence.mjs` CLI operated a disposable Git source and durable journal under the local temporary directory. It initialized a run, started an iteration, finalized it as failed, admitted an evidence-linked corrective iteration, finalized that correction as failed, retired only the immediately preceding failure with a SHA-256-linked JSON receipt and explicit authority reference, and admitted a third iteration.

The final `status` and `report` reads showed two failed iterations, a visible `retired` disposition on the second, a third active iteration, and `successfulDeliveries: 0`. The initial gate script asserted a nonexistent `report.successfulDeliveries` field; inspection showed that the field belongs to `status`, and the affected assertion passed when rerun against both actual contracts. No product failure or extra build was involved.

This disposable operation had no publication obligation. Task 5 checks the existing C09.4 publication debt before and after reconciliation of the live C08 candidate. The operation does not certify C08 or mark its canonical KBD tasks complete.
