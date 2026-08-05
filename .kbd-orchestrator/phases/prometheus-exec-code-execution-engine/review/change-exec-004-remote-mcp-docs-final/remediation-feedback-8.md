# Expired target-arrival remediation

The distinct-model finding was valid. The generic queue acceptance API rejects
an expired dispatch before insertion, which is correct for an origin attempting
to create new expired work but did not satisfy the target-arrival evidence
contract.

`DispatchQueue::accept_at_target` now preserves the stricter target semantics:

1. verify the signed dispatch and enrollment snapshot;
2. enforce dispatch/request replay and hash-conflict rules under the queue lock;
3. durably append the accepted event;
4. when already expired, append a terminal `Expired` event before returning;
5. let `RemoteTarget` sign a peer response from that durable terminal record.

The existing `DispatchQueue::accept` behavior remains unchanged, so an origin
still cannot enqueue newly created expired work. A transport-gated regression
proves an already-expired target arrival produces a sequence-2 durable terminal
record, a signed response, and zero executor calls.

Local verification:

- `cargo test --manifest-path substrate/exec-remote/Cargo.toml --features transport --test disposable_peers target_durably_records_a_dispatch_that_arrives_expired -- --exact --nocapture` — 1 passed
- `cargo test --manifest-path substrate/exec-remote/Cargo.toml --lib queue::tests::accept_replay_conflict_restart_and_expiry_are_durable -- --exact --nocapture` — 1 passed
- `cargo clippy --manifest-path substrate/exec-remote/Cargo.toml --all-targets --features transport -- -D warnings`
