# `prometheus-exec-service`

The shared durable run facade and sidecar API. It owns request replay/conflict behavior, spawn boundaries, ordered events, response-loss reconciliation, receipt publication, CAS ownership, health/readiness, and the same-user Unix-socket REST surface used by the CLI daemon.

```bash
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-service/Cargo.toml
RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path substrate/exec-service/Cargo.toml --all-targets -- -D warnings
```

Canonical documentation: [local API, CLI, and MCP](/docs/execution/local-api-cli-and-mcp) and [installation, doctor, and recovery](/docs/execution/installation-doctor-and-recovery).
