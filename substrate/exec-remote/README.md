# `prometheus-exec-remote`

Estate-only, transport-injected Tier R dispatch kernel. It verifies enrolled origin/target identities, persists immutable origin and target queues, rejects replay/expiry/conflicts, delegates accepted work to the local execution facade, verifies signed peer responses, and derives per-target aggregates. It does not pair devices or depend on KBD/Sovereign services.

Complete the production phase before validation. Acceptance must exercise a
production entry point with real collaborating components across its filesystem,
process or protocol boundary. Run the applicable local integration gate and record
source identity, commands, results and limitations; a module-only or unit suite
is not release evidence.


Canonical documentation: [remote dispatch and reconciliation](../../site/docs/execution/remote-dispatch-and-reconciliation.md).
