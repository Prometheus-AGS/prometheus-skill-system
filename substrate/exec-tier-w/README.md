# `prometheus-exec-tier-w`

Wasmtime 46 component authorization, typed capability host, deterministic time/random inputs, resource fences, Cranelift/Pulley profiles, portable replay, and backend-independent receipt projections. The checked reference component is authorized by signed generation or exact pin before validation or compilation.

```bash
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-tier-w/Cargo.toml --features estate
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-tier-w/Cargo.toml --no-default-features --features bundled-mobile
```

Canonical documentation: [Tier W architecture](/docs/execution/architecture-and-tiers#tier-w-portable-component-execution) and [signed component distribution](/docs/plugin-distribution/signing-index-and-receipts).
