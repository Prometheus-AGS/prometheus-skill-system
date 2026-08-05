# Bounded terminal-wait remediation

The distinct-model finding was valid. A target-signed peer response is a
terminal receipt envelope, so a one-shot local `status` result could not safely
represent a still-running local run.

The local handoff now exposes `await_terminal`, whose contract requires a
request-bounded terminal wait. `RemoteTarget` independently rejects a
nonterminal outcome, leaves the exact durable `Running` record and `runId`
unchanged, emits no peer response, and permits a later duplicate delivery to
resume the wait without another submit. A returned run ID cannot replace the
stored ID.

The design and canonical remote-reconciliation documentation now state the
same behavior.

Local verification:

- transport scenarios: 8 passed, including
  `premature_terminal_wait_keeps_the_durable_run_reconcilable` and the
  zero-resubmission duplicate-running case;
- library suite: 8 passed;
- warnings-denied clippy: passed.

As explained in remediation feedback 9, task 6.3 is the self-referential phase
closure task and remains open until this independent review returns zero
findings. It is not a product acceptance defect.
