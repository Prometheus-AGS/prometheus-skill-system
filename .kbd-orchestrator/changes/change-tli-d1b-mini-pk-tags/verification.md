# Verification — change-tli-d1b-mini-pk-tags

Repository: `prometheus-skills-mini`
Depends on: `change-tli-a5b-mini-pin-operator-request`, `change-tli-d1a-mini-file-tier-port`

## Acceptance criteria

- With pk 1.10.0 the spawned args include the tags; with an older pk they do not; the full suite adds no failure beyond the recorded baseline.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
git fetch -q origin
git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge fetch -q origin --tags; pk_c="$(git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge rev-list -n1 v1.10.0)"; a="$(git ls-tree HEAD tools/prometheus-knowledge | awk "{print \$3}")"; test -n "$pk_c" && test -n "$a" && git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge merge-base --is-ancestor "$pk_c" "$a" || { echo "BLOCKED-ON-OPERATOR: mini pk pin is not at or after v1.10.0" >&2; exit 2; }
node --test lib/karpathy/transport.test.mjs
node --test --test-reporter=tap 2>&1 | grep -E "^not ok" | sed "s/^not ok [0-9]* - //" | sort > "${TMPDIR:-/tmp}/tli-mini-fail.$$"; comm -23 "${TMPDIR:-/tmp}/tli-mini-fail.$$" "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/mini-baseline-failures.txt" > "${TMPDIR:-/tmp}/tli-mini-new.$$"; if [ -s "${TMPDIR:-/tmp}/tli-mini-new.$$" ]; then echo "new test failures beyond the recorded mini baseline:" >&2; cat "${TMPDIR:-/tmp}/tli-mini-new.$$" >&2; exit 1; fi
```
