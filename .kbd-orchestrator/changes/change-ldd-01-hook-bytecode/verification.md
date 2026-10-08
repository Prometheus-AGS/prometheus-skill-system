# Acceptance — change-ldd-01-hook-bytecode

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: From a packaged scratch installation, dispatch SessionStart, SubagentStart and SubagentStop through actual compiled and shell entry paths, with normal filesystem permissions and bytecode suppression absent/0 in the caller. Exercise direct wrappers and writer/feeder imports too.

- AC-2: Compare the entire immutable generation before and after real hook execution: no new .pyc, __pycache__, content change or executable-mode change; actual installer/generation verification remains successful. Learning activity must occur in scratch outbox/log/service, so a silent no-op is not evidence.

- AC-3: Run an old-source negative control in a disposable generation: reproduce imported local-module bytecode and verifier rejection. Never weaken the positive integrity assertion to make a read-only payload pass.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.
