# prometheus-knowledge

Git submodule: https://github.com/Prometheus-AGS/prometheus-knowledge-rs.git

The Karpathy LLM Knowledge Base method in Rust. Self-maintaining, human-readable
Markdown wiki compiled and linted by LLMs. No vector database. Every fact traces
to a readable `.md` file. Powers the Karpathy learning loop in `forge-rs`.

## Initialize

```bash
git submodule update --init tools/prometheus-knowledge
cd tools/prometheus-knowledge
cargo build --release -p pk-cherry -p pk-cli
```

## Role in the skill pack

`forge-rs reflect` pipes completed iteration data to `pk ingest`, feeding
the Karpathy learning loop. `forge-rs enrich` calls `pk focus` to pull
relevant learned context into enriched task documents.

## Key binaries

- `pk` — CLI: `ingest`, `lint`, `focus`, `search`, `list`
- `pk-cherry` — Cherry Studio MCP bridge on port 8942

## MCP tools

- `knowledge_ingest` — compile raw content into wiki
- `knowledge_focus` — build a mini-KB for a topic (used by forge enrich)
- `knowledge_search` — TF-IDF search
- `knowledge_lint` — scan for contradictions and gaps
- `knowledge_get` — retrieve a single article

## PMPO Reflect hook

```bash
echo "$SESSION_NOTES" | pk ingest --source "forge:reflect:$(date +%s)"
```
