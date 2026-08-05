# `prometheus-exec-tier-p`

Native Python, Node, and Bash execution through fail-closed platform sandboxes. macOS uses Seatbelt; Linux carries deterministic bubblewrap/Landlock planning and cross-build fixtures but no runtime certification from the release Mac. There is no direct-process fallback.

```bash
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path substrate/exec-tier-p/Cargo.toml
RUSTUP_TOOLCHAIN=stable cargo clippy --manifest-path substrate/exec-tier-p/Cargo.toml --all-targets -- -D warnings
```

Canonical documentation: [Tier P architecture](/docs/execution/architecture-and-tiers#tier-p-native-process-isolation) and [platform status](/docs/execution/platform-and-evidence-status).
