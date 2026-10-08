# Verification — change-tli-a5a-skillpack-pin-v1-10-0

Repository: `prometheus-skill-system`
Depends on: `change-tli-a1-pk-context-scoring`, `change-tli-a2-pk-worker-attribution`, `change-tli-a3-pk-tags-and-type`, `change-tli-a4-surreal-lean-search-categories-rekey`

## Acceptance criteria

- `git ls-tree HEAD tools/prometheus-knowledge tools/surreal-memory-server` equals the `v1.10.0` tag commits and `skill-system.json` import commits.
- Built from the pinned submodules, `pk`, `prometheus-learning-worker` and `surreal-memory-server` report 1.10.0. **Amended at execute:** `install-binaries.sh` cannot run from a worktree, because the certified `prometheus-exec` hash depends on the build path (follow-up task). Machine installation happens after merge with the normal installer from the main checkout.
- `cargo check` passes in tools/prometheus-cli and tools/forge-rs (run sequentially).

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
a="$(git ls-tree HEAD tools/prometheus-knowledge | awk "{print \$3}")"; b="$(git -C tools/prometheus-knowledge rev-list -n1 v1.10.0)"; test -n "$a" && test -n "$b" && test "$a" = "$b"
a="$(git ls-tree HEAD tools/surreal-memory-server | awk "{print \$3}")"; b="$(git -C tools/surreal-memory-server rev-list -n1 v1.10.0)"; test -n "$a" && test -n "$b" && test "$a" = "$b"
npm run check:distribution
(cd tools/prometheus-cli && cargo check) && (cd tools/forge-rs && cargo check)
bash scripts/install-binaries.sh
pk --version | grep -Eq '(^|[^0-9.])1\.10\.0([^0-9]|$)'
surreal-memory-server --version | grep -Eq '(^|[^0-9.])1\.10\.0([^0-9]|$)'
```
