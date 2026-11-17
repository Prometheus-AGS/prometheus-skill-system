# SDL and Rust tooling — schema, CLI, MCP

## `entity-graph-sdl` (npm)

Exports: `parseSdl`, `parseSdlJson`, `SdlValidationError`.

Parses and validates the SDL JSON schema format (the `EntityGraphIR` shape)
into typed schema objects consumed by `registerSchema` and the code
generators. Gate: `pnpm --filter @prometheus-ags/entity-graph-sdl test`.

## `entity-graph-cli` (Rust crate, `entity-graph` binary)

Self-contained scaffolder + generator for the entity-graph ecosystem.
Install: `cargo install --path packages/entity-graph-cli`.

- `entity-graph init [--out <dir>] [--force]` — writes a starter
  `schema.json` in the SDL format consumed by `parseSdl()`.
- `entity-graph generate --target react --schema schema.json` — emits
  TypeScript type definitions and transport-registration stubs.

Gate: `cargo test --manifest-path packages/entity-graph-cli/Cargo.toml`.

## `entity-graph-mcp` (Rust crate)

MCP server exposing the entity graph to agent hosts.

- Resources: `entity://{entityType}` (list), `entity://{entityType}/{entityId}`
  (read).
- Tools: `entity_list_types`, `entity_query` (field-equality filters),
  `entity_upsert` (insert/shallow-merge), `entity_delete`.
- Transports: **stdio** (local hosts such as Claude Desktop) and
  **Streamable HTTP** (remote / multi-tenant, Axum).

Gate: `cargo test --manifest-path packages/entity-graph-mcp/Cargo.toml`.

## Boundary rules

- Generated TypeScript targets the public packages only; regeneration must not
  hand-edit generated files.
- The MCP server mutates the graph through its tools; hosts remain
  responsible for caller authorization. Do not expose a write-capable MCP
  transport to untrusted networks without an auth proxy (see
  `docs/flint-integration.md` for the fabric's identity plane).
