# Verification — change-tli-a1-pk-context-scoring

Repository: `prometheus-knowledge-rs`
Depends on: none

## Acceptance criteria

- A project KB with 200 committed entries whose only match sorts last by id is returned by `pk context` with default flags.
- With the project scope failing (no root) the shared scope alone can fill the whole candidate cap.
- Two runs over the same snapshot produce byte-identical JSON output.
- Existing `pk-cli/tests/context.rs` still passes.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
cargo test -p pk-cli --test context_scoring
cargo test -p pk-cli --test context
```
