# Acceptance — change-ldd-09-partition-footer

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Use the production partition CLI on scratch files at cumulative counts 9/10 and 99/100 with exact byte-boundary and multibyte text cases, both with and without a prior footer. Reapply and compare output bytes and outbox identities.

- AC-2: Assert the byte ceiling, ranking and no duplicate durable operations; a genuinely impossible structural budget must report failure without destructive partial replacement. Historical off-by-one is a claim to reproduce through the control, not prior passing evidence.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
