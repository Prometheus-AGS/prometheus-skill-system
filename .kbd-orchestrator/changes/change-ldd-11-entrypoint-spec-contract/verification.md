# Acceptance — change-ldd-11-entrypoint-spec-contract

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Through the packaged kbd-spec guidance, produce a disposable native example and an OpenSpec example and check each contains a real executable production route and isolation/negative-control contract. Exercise the declared route for a small existing real CLI; merely grepping template headings is insufficient acceptance.

- AC-2: Independently review the example criterion that offers only an isolated helper test and require rejection as incomplete evidence. Cross-family review is performed only at the final completed-production boundary.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
