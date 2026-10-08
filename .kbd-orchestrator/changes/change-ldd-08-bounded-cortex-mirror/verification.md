# Acceptance — change-ldd-08-bounded-cortex-mirror

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: Invoke production learning_write.py concurrently with actual Cortex MCP/server processes and a scratch data directory. Record actual live feeder/server counts; never exceed the configured bound, including mixed installed generations using the same state root.

- AC-2: Saturation preserves every durable outbox/log lesson and reports skipped mirrors. Kill a scratch feeder, force spawn failure/timeout and confirm capacity can be reused without killing unrelated processes or leaking servers. Run actual completed remember calls and verify scratch persistence.

- AC-3: Gate code must never read/assert real Cortex database mtime/size. No fixture MCP server replaces Cortex for acceptance; absent executable/model is BLOCKED. Host-specific lock behavior cannot imply untested Windows certification.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
