# `prometheus-exec-remote`

Estate-only, transport-injected Tier R dispatch kernel. It verifies enrolled origin/target identities, persists immutable origin and target queues, rejects replay/expiry/conflicts, delegates accepted work to the local execution facade, verifies signed peer responses, and derives per-target aggregates. It does not pair devices or depend on KBD/Sovereign services.

```bash
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-remote/Cargo.toml
RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path substrate/exec-remote/Cargo.toml --all-targets -- -D warnings
```

Canonical documentation: [remote dispatch and reconciliation](/docs/execution/remote-dispatch-and-reconciliation).
