# Verification — change-tli-d2-mini-hook-timeouts-seconds

Repository: `prometheus-skills-mini`
Depends on: none

## Acceptance criteria

- Every hook timeout is between 1 and 600.
- hooks-budget.test.mjs fails when a fast-budget (≤ 5 s) payload imports lib/karpathy (negative control with trap restore).
- The full suite adds no failure beyond the 19 pre-existing ones recorded in `evidence/mini-baseline-failures.txt` (package-builder fixture lacks scripts/kbd-apply.mjs; OpenSpec pin test expects 1.10.0), and `npm run check:distribution` passes.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
node -e 'const h=require("./hooks/hooks.json").hooks;for(const g of Object.values(h).flat())for(const x of g.hooks)if(x.timeout!==undefined&&(x.timeout<1||x.timeout>600))process.exit(1)'
node --test hooks/hooks.test.mjs lib/karpathy/hooks-budget.test.mjs
node --test --test-reporter=tap 2>&1 | grep -E "^not ok" | sed "s/^not ok [0-9]* - //" | sort > "${TMPDIR:-/tmp}/tli-mini-fail.$$"; comm -23 "${TMPDIR:-/tmp}/tli-mini-fail.$$" "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt" > "${TMPDIR:-/tmp}/tli-mini-new.$$"; if [ -s "${TMPDIR:-/tmp}/tli-mini-new.$$" ]; then echo "new test failures beyond the recorded mini baseline:" >&2; cat "${TMPDIR:-/tmp}/tli-mini-new.$$" >&2; exit 1; fi
npm run check:distribution
```
