# Verification — change-tlh-08-recall-scoping-and-eval

Repository: `prometheus-skill-system`
Depends on: `change-tlh-04-scratch-surreal-for-envelope-test`

## Acceptance criteria

- For each of the three held-out fixture queries, >= 3 of the top 5 items are current-project or role scope and 0 are foreign-project untagged entries.
- The fixture contains a current-project untagged pk entry relevant to a held-out query, and the test asserts it appears in that query's recalled items.
- `test-kbd-memory-loop.sh` and `test-subagent-delivery.sh --harness none` still pass.
- Uses scratch HOME, a scratch surreal-memory and a scratch pk root; the real stores are never queried.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
test -f shared/scripts/tests/lib/scratch-surreal.sh || { echo 'BLOCKED: dependency change-tlh-04 not in this base (rebase onto main after it merges)' >&2; exit 2; }
command -v surreal-memory-server >/dev/null || { echo 'BLOCKED: surreal-memory-server not on PATH' >&2; exit 2; }
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-recall-quality.sh
/bin/bash shared/scripts/tests/test-kbd-memory-loop.sh
/bin/bash shared/scripts/tests/test-subagent-delivery.sh --harness none
npm run check:distribution
```
