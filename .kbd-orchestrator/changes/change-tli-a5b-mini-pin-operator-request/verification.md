# Verification — change-tli-a5b-mini-pin-operator-request

Repository: `prometheus-skills-mini`
Depends on: `change-tli-a5a-skillpack-pin-v1-10-0`

## Acceptance criteria

- The request file names both v1.10.0 commits and a versions.toml diff that `rules/test/versions-toml.test.mjs` would accept with those gitlinks.
- Mini `origin/main` has both gitlinks at the v1.10.0 commits and `npm test -- rules/test/versions-toml.test.mjs` passes there.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
git fetch -q origin
pk_c="$(git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge rev-list -n1 v1.10.0)"; sm_c="$(git -C /Users/gqadonis/Projects/prometheus/surreal-memory-server rev-list -n1 v1.10.0)"; a="$(git ls-tree origin/main tools/prometheus-knowledge | awk "{print \$3}")"; b="$(git ls-tree origin/main tools/surreal-memory-server | awk "{print \$3}")"; test -n "$pk_c" && test -n "$sm_c" && test "$a" = "$pk_c" && test "$b" = "$sm_c" || { echo "BLOCKED-ON-OPERATOR: mini pins not yet at v1.10.0" >&2; exit 2; }
```
