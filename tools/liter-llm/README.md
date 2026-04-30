# liter-llm

Git submodule: https://github.com/GQAdonis/liter-llm.git

Rust LLM proxy with built-in MCP server. Provides per-phase model routing
across 140+ providers for the Prometheus skill pipeline (KBD, iterative-evolver,
forge-rs, zeespec-interrogator).

## Initialize

```bash
git submodule update --init tools/liter-llm
cd tools/liter-llm
cargo build --release -p liter-llm-cli
```

## Role in the skill pack

Used by `skills/process/liter-llm-bridge/` as the routing backend.
`forge-rs` uses liter-llm for cheap-model dispatch during skill enrichment and
reflection phases.

## MCP server

```bash
./target/release/liter-llm mcp --transport stdio
```

## Key tools exposed (22 MCP tools)

- `complete` — route a completion to the cheapest model matching the class
- `list_models` — list available models per provider
- `get_cost` — estimate cost for a prompt class
