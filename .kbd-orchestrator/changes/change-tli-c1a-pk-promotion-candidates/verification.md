# Verification — change-tli-c1a-pk-promotion-candidates

Repository: `prometheus-knowledge-rs`
Depends on: `change-tli-a5a-skillpack-pin-v1-10-0`

## Acceptance criteria

- The same lesson in two projects yields exactly one candidate with 2 evidence entries.
- `pk candidates accept <id>` writes the shared snapshot and queues a `@global` op; the file moves to accepted/.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
cargo test -p pk-learning-worker --test promotion
cargo test -p pk-cli --test candidates
```
