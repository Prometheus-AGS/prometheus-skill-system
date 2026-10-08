# Verification — change-tli-a2-pk-worker-attribution

Repository: `prometheus-knowledge-rs`
Depends on: none

## Acceptance criteria

- A job with projectId P, teamId T, roleId R produces an outbox memory op with `user_id=P` and `agent_id=T/R`.
- A job with no team/role produces `agent_id=@project`; no outbox op in the test run has a null `agent_id` or `user_id`.
- An `add_task_step` payload normalises with non-null `user_id`/`agent_id`.
- After `run-once`, `pk context --format json` in the test KB returns the session entry the job created.
- Existing `pk-learning-worker/tests/worker.rs` still passes.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
cargo test -p pk-learning-worker --test attribution
cargo test -p pk-learning-worker --test worker
```
