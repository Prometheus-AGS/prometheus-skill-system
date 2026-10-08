# Acceptance — change-ldd-10-mini-executable-modes

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Generate a real mini package through its CLI; execute its refresh helper in an explicitly scratch environment and verify it can be launched directly on the local POSIX filesystem. Document any required safe dry-run arguments so the helper cannot refresh a real installation.

- AC-2: Remove the executable bit in a disposable generated copy and require --check to reject the drift. Byte-identical but nonexecutable output must fail. Record exact mini commit, selected production integration entry points and failure/blocker identities in the new baseline.

- AC-3: Do not run bare node --test/npm test or treat isolated module tests as acceptance. Preserve historical baseline and label unmapped legacy failures unverified, rather than silently dropping them.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
