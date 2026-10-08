# Verification — change-tlh-09-rebase-regenerate

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- In a temporary clone, a rebase whose only conflicts are generated paths is resolved by the helper, the validators pass, and `git diff --cached` contains only generated paths.
- A rebase with one conflicted source file (non-generated) exits 1 naming that file and changes nothing.
- Outside a rebase or merge the helper exits 2 with a message.
- `check:distribution` still passes and uses the shared generated-path set.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
node scripts/generated-paths.mjs | grep -q '^dist/' || { echo 'generated-paths lists no dist paths' >&2; exit 1; }
grep -q generated-paths scripts/generate-skill-system-distribution.js || { echo 'distribution check does not use generated-paths' >&2; exit 1; }
/bin/bash scripts/tests/test-rebase-regenerate.sh
npm run check:distribution
npm run validate:harness-adapters
npm run validate:codex
```
