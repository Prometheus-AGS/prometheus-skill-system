# Oversized MCP artifact retrieval remediation

The final-pass critical finding was valid. `exec-artifact` returned bounded
metadata for a non-inline result but did not tell the caller how to retrieve
the complete artifact.

The MCP server now retains its private runner socket path and emits a
deterministic `retrieval` object whenever `inline` is false:

- `transport`: `unix-domain-http`
- `method`: `GET`
- `socketPath`: the active MCP runner socket
- `path`: `/api/v2/exec/artifacts/{digest}`

The response remains bounded and never truncates bytes into a false success.
The focused regression asserts all four retrieval fields, and the canonical
Docusaurus page plus crate README describe the same contract.

Local verification:

- `cargo fmt --manifest-path crates/prometheus-exec/Cargo.toml --all -- --check`
- `cargo test --manifest-path crates/prometheus-exec/Cargo.toml mcp::tests::artifact_tool_never_truncates_an_oversized_inline_result -- --exact --nocapture` — 1 passed
- `cargo clippy --manifest-path crates/prometheus-exec/Cargo.toml --all-targets -- -D warnings`
