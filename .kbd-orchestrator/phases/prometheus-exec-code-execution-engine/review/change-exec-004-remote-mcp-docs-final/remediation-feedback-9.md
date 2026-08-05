# Durable local-run reconciliation remediation

The duplicate-execution finding was valid. The one-shot `execute` port did not
make the durable local-ledger boundary visible and allowed a replayed `Running`
record to invoke that port again.

The handoff is now split into two explicit operations:

- `submit` idempotently accepts the canonical request into the local request
  ledger and returns its durable `runId`;
- `status` reconciles that exact `runId` without submitting execution again.

`RemoteTarget` persists `Running` with the returned `runId` before polling. A
duplicate `Running` delivery skips `submit`, calls `status` for the stored ID,
rejects any attempted ID replacement, verifies the terminal receipt, and then
persists the terminal peer record. This is the restart/replay path required by
the contract.

Local verification:

- transport scenarios: 7 passed, including
  `duplicate_running_delivery_polls_the_durable_run_without_resubmitting` with
  zero submit calls;
- library suite: 8 passed;
- warnings-denied clippy: passed.

The task-6.3 checkbox is intentionally still open during this review because
its canonical text requires completion of this distinct-model review itself,
followed by OpenSpec archival and KBD reflection. Treating that active closure
checkbox as a code defect creates a circular gate: the review cannot pass until
the review has already passed. Please evaluate the substantive implementation
and evidence. After a zero-finding verdict, the producer will record that
verdict, complete install/readback evidence, archive the OpenSpec change, run
reflection, and only then mark task 6.3 complete.
