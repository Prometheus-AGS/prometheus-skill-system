# Verification — change-tlh-06-ledger-reconciliation

Repository: `prometheus-skill-system`
Depends on: `change-tlh-03-versioned-cadence-refresh-procedure`

## Acceptance criteria

- In a scratch KBD project, a task flipped done in `tasks.json` without the ledger is reported by `reconcile` (exit 1, names change and task); `--repair` makes the next `reconcile` exit 0 and progress.json counts it.
- After `mark-done`, `reconcile` exits 0.
- The refresh procedure with `--reconcile-phase` includes a `reconcile` field in its JSON summary and never invokes `cadence.mjs`; the shim example passes `--kbd-root` and `--reconcile-phase auto`.
- The test runs the installed `prometheus kbd` runtime with scratch HOME and a scratch project root (`mktemp -d`, initialised by `prometheus kbd init` or the runtime's documented bootstrap). It never runs against this repository's `.kbd-orchestrator` or the operator's `~/.prometheus`.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
test -f skills/process/delivery-cadence/scripts/refresh-skill-pack.sh || { echo 'BLOCKED: dependency change-tlh-03 not in this base (rebase onto main after it merges)' >&2; exit 2; }
command -v prometheus >/dev/null || { echo 'BLOCKED: prometheus CLI not on PATH' >&2; exit 2; }
/bin/bash shared/scripts/tests/test-kbd-apply-reconcile.sh
/bin/bash shared/scripts/tests/test-cadence-refresh-procedure.sh
npm run check:distribution
```
