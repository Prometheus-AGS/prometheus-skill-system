# Verification — change-tli-d1a-mini-file-tier-port

Repository: `prometheus-skills-mini`
Depends on: `change-tli-b7-cross-agent-awareness-beta`, `change-tli-d2-mini-hook-timeouts-seconds`

## Acceptance criteria

- The carried schema is byte-identical to the skill-pack copy at the pinned commit.
- With an empty HOME the hook exits 0 with empty stdout; a fixture team role receives only its own file-tier lessons.
- The full suite adds no failure beyond the 19 pre-existing ones recorded in `evidence/mini-baseline-failures.txt` (package-builder fixture lacks scripts/kbd-apply.mjs; OpenSpec pin test expects 1.10.0), and `npm run check:distribution` passes.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack cat-file -e origin/main:shared/scripts/lib/learning_route.py || { echo 'BLOCKED: B7 not on skill-pack origin/main' >&2; exit 2; }
git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack show origin/main:shared/schemas/learning-envelope.schema.json > "${TMPDIR:-/tmp}/tli-d1a-schema.$$.json"
cmp lib/learning/learning-envelope.schema.json "${TMPDIR:-/tmp}/tli-d1a-schema.$$.json"
node -e 'const h=require("./hooks/hooks.json").hooks;for(const g of Object.values(h).flat())for(const x of g.hooks)if(x.timeout!==undefined&&(x.timeout<1||x.timeout>600))process.exit(1)'
node --test lib/learning/identity.test.mjs hooks/subagentstart-learning.test.mjs
node --test --test-reporter=tap 2>&1 | grep -E "^not ok" | sed "s/^not ok [0-9]* - //" | sort > "${TMPDIR:-/tmp}/tli-mini-fail.$$"; comm -23 "${TMPDIR:-/tmp}/tli-mini-fail.$$" "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt" > "${TMPDIR:-/tmp}/tli-mini-new.$$"; if [ -s "${TMPDIR:-/tmp}/tli-mini-new.$$" ]; then echo "new test failures beyond the recorded mini baseline:" >&2; cat "${TMPDIR:-/tmp}/tli-mini-new.$$" >&2; exit 1; fi
npm run check:distribution
```
