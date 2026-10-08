# Verification — change-tli-a3-pk-tags-and-type

Repository: `prometheus-knowledge-rs`
Depends on: `change-tli-a1-pk-context-scoring`

## Acceptance criteria

- Two entries ingested with `--type Lesson --tag role:a` and `--type Lesson --tag role:b`: `pk context --tag role:a` returns only the first.
- `pk get` on an entry ingested with `--type Gotcha` reports type Gotcha; one ingested without `--type` reports Reference.
- `pk context` with no `--tag` behaves exactly as after A1.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
cargo test -p pk-cli --test tags_and_type
cargo test -p pk-cli --test context_scoring
```
