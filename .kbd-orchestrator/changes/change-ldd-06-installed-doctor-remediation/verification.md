# Acceptance — change-ldd-06-installed-doctor-remediation

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Run built prometheus doctor from an unrelated scratch working directory against a real scratch generation whose path contains spaces. Execute its suggested repair command against scratch configuration and re-run the diagnostic predicate.

- AC-2: Missing generation/helper, corrupt manifest or an escaping pointer yields an actionable unhealthy result and no unsafe command. No source checkout is required at runtime.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
