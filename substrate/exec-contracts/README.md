# `prometheus-exec-contracts`

Transport-free request, receipt, signature, receipt-log, evidence-index, certification-status, JSON Schema, and OpenAPI contracts for Prometheus Exec. The crate depends on no daemon, async runtime, execution backend, KBD, or Sovereign service.

```bash
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-contracts/Cargo.toml
RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path substrate/exec-contracts/Cargo.toml --all-targets -- -D warnings
```

Canonical documentation: [receipts and certification](/docs/execution/receipts-verification-and-certification) and the checked [OpenAPI specification](/openapi/prometheus-exec.openapi.json).
