# Verification — change-tlh-03-versioned-cadence-refresh-procedure

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- With `state.json` iteration 7 `--mode auto` selects full; iteration 8 selects verify; a missing state file, a non-numeric iteration, or an unreadable file exits 2 and does no work.
- Full mode on a scratch deploy worktree fast-forwards it and leaves its submodule at the recorded commit (clean `git status`); a dirty or diverged worktree exits 1 before any install step.
- The procedure runs under `/bin/bash` (3.2) in the test.
- Verify mode leaves HEAD, the submodule commit and the stub-call log unchanged, and its JSON summary carries `sourceCommit`, `versions` and `health` keys.
- In the test, install and kickstart steps are replaced by recording stubs through `REFRESH_UPDATE_CMD`/`REFRESH_INSTALL_CMD`/`REFRESH_KICKSTART_CMD` env overrides; the real machine is never refreshed by the gate.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
/bin/bash shared/scripts/tests/test-cadence-refresh-procedure.sh
npm run check:distribution
```
