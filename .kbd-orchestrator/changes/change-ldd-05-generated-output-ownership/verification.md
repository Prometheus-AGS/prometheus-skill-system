# Acceptance — change-ldd-05-generated-output-ownership

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Invoke real generation and rebase-regenerate on disposable Git worktrees with initialized and uninitialized submodule layouts; the same legitimate change has the same verdict.

- AC-2: Negative controls remove a required generated output and introduce an undeclared generated write in an isolated fixture source; the production check/consumer must reject both. A comparison of two derived lists is insufficient.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
