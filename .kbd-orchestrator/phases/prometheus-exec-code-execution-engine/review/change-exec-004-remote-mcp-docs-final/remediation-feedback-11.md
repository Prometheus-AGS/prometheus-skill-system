# MCP positive-limit schema remediation

The PASS-with-warning finding was valid. Runtime parsing rejects zero values,
but the generated schemas inherited the unsigned integer default of
`minimum: 0`.

Schemars range annotations now declare `minimum: 1` for `timeoutMs`, `outputMb`,
and `inlineCeilingBytes`. The checked MCP contract was regenerated from the
compiled Rust types. A focused contract regression asserts the positive
timeout/output minima.

Local verification:

- `prometheus-exec contracts --output-dir docs/reference/api` regenerated the
  contract;
- `mcp::tests::run_schema_rejects_private_key_arguments` passed with the new
  minimum assertions;
- warnings-denied clippy passed;
- generated JSON readback reports `minimum: 1` for all three positive limits.

Task 6.3 remains the circular closure checkbox described in remediation
feedback 9 and 10; it is completed only after the zero-finding review is
recorded and the phase is archived/reflected.
