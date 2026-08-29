# Plan — bottleneck-evaluator-and-detector

1. `add-boundary-gate-runtime`: add signed receipt types, folded state, replay,
   projection reconciliation, guard evaluation, and typed gate execution.
2. `wire-lifecycle-detector`: wrap KBD/OpenSpec task boundaries, KBD phase/child
   boundaries, ZeeSpec checkpoints, and per-harness completion/re-anchor hooks.
3. `package-terminal-review`: package the skill and add bounded adversarial plus
   sycophancy-screened terminal escalation.
4. `certify-and-return`: run only local full-integration gates, complete the
   child, and restore the parent to `repair-kbd-memory-rest-contract`.

Implementation is completed before builds or tests. The hot path is local and
network-free; only terminal/ambiguous review may call a model gateway.
