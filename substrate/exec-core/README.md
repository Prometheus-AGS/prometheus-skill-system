# `prometheus-exec-core`

Backend-independent policy, Cedar tightening, grants, content-addressed artifact retention, receipt assembly, and hash-linked receipt log. Execution backends implement its port; adapters do not bypass it.

```bash
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-core/Cargo.toml
RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path substrate/exec-core/Cargo.toml --all-targets -- -D warnings
```

Canonical documentation: [architecture and tiers](/docs/execution/architecture-and-tiers) and [evidence-producing execution](../../docs/decisions/evidence-producing-execution.md).
