# Acceptance — change-ldd-02-codex-memory-control

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Run the real installer entry point in scratch HOME/CODEX_HOME; assert the three settings, retained unrelated TOML and backup/archive contents. Cases include missing tables, existing true/false, comments, unsupported/malformed config and repeat installation. Doctor is invoked from the built CLI, not a copied parser.

- AC-2: In a wholly scratch Codex home, use a real supported local model/provider and real Codex binary to create eligible memory state/outputs, demonstrate consolidation with the feature enabled despite generate_memories=false, then restart with the feature disabled and prove the startup job and filesystem writes do not occur. Do not copy real auth or memory databases. A missing local provider or inability to trigger the control is BLOCKED; a stub writer is not accepted.

- AC-3: Bound observation by the actual startup/consolidation job state and recorded control behavior, not a short sleep alone. Record CLI engine version separately from any desktop engine; application to desktop remains pending unless its effective settings are established.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
