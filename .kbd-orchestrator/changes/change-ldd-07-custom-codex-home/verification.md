# Acceptance — change-ldd-07-custom-codex-home

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Run production install/check/uninstall/rollback with scratch HOME and a custom sibling CODEX_HOME, including spaces, empty CODEX_HOME and explicit --home. Prove plugin inventory, settings, MCP, copied skills and receipts agree on the selected root.

- AC-2: Place sentinels in the unused scratch fallback directory and unowned selected-root entries; all remain unchanged. After rollback and generation cleanup, active custom-root copies still resolve to the intended verified generation.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
