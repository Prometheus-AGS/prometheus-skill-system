# Verification — change-tli-e1-pin-v1-11-0

Repository: `prometheus-skill-system`
Depends on: `change-tli-a5a-skillpack-pin-v1-10-0`, `change-tli-c1a-pk-promotion-candidates`, `change-tli-c3a-pk-skill-discovery`

## Acceptance criteria

- The pk gitlink equals the v1.11.0 tag commit; `install-binaries.sh` installs pk 1.11.0; `pk candidates list` runs.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
a="$(git ls-tree HEAD tools/prometheus-knowledge | awk "{print \$3}")"; b="$(git -C tools/prometheus-knowledge rev-list -n1 v1.11.0)"; test -n "$a" && test -n "$b" && test "$a" = "$b"
npm run check:distribution
(cd tools/prometheus-cli && cargo check) && (cd tools/forge-rs && cargo check)
bash scripts/install-binaries.sh
pk --version | grep -Eq '(^|[^0-9.])1\.11\.0([^0-9]|$)'
pk candidates list --kind promotion >/dev/null
```
