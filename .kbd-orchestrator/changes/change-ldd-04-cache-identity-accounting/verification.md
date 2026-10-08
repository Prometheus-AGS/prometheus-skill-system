# Acceptance — change-ldd-04-cache-identity-accounting

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Through the production server HTTP search and stats endpoints backed by real isolated SurrealDB and an actual local embedding executor, repeat and coalesce queries, change query, disable capacity, and induce a real worker failure. Counter deltas equal observed executor attempts and successful reuse; failure does not install a value.

- AC-2: Exercise actual supported model/configuration selection, including two available same-width models where available, and record model identities and results across restarts. Source/contract inspection must independently establish that dimensions alone cannot identify entries. Missing required model artifacts is BLOCKED, not a mock provider pass.

- AC-3: Writes and duplicate detection leave query-cache stats unchanged. Stats on the supported backend expose the same public object; unsupported durable storage returns its documented error. Isolated legacy trait/fixture tests are not acceptance evidence.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
