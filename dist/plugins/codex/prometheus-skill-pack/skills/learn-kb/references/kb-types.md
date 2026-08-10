# KB Adapter Types — Deep-Dive Reference

This document covers the four adapter types supported by `learn-kb`. Each
section describes prerequisites, how the adapter works internally, and an
example `add` command.

---

## 1. Dify KB (`type: dify`)

### What it is

A Dify knowledge base is an AI-native document store with chunking, embedding,
and semantic search built in. The `dify` adapter queries it through the
`dify_search` MCP tool (or the REST API) at query time. No data is ingested
locally — Dify manages the index.

### Prerequisites

| Variable | Description | Example |
|---|---|---|
| `DIFY_API_KEY` | API key from your Dify workspace | `app-abc123...` |
| `DIFY_BASE_URL` | Base URL of your Dify instance | `http://localhost/v1` |

Both variables must be set in the environment before calling `add` or `query`.
They are read by `content-grounding-kb.sh` at query time.

### Setup steps

1. Deploy Dify (self-hosted Docker or cloud).
2. Create a knowledge base in the Dify UI: **Knowledge** → **Create Knowledge
   Base** → name it to match what you will pass to `--name`.
3. Upload or sync documents in the Dify UI.
4. Export or copy your API key from **Settings** → **API Access**.
5. Set the environment variables:
   ```bash
   export DIFY_API_KEY="app-abc123..."
   export DIFY_BASE_URL="http://localhost/v1"
   ```

### Example `add` command

```
/learn-kb add --type dify --name legal-frameworks \
  --description "Internal legal framework library maintained in Dify"
```

### Query behavior

At query time, `content-grounding-kb.sh` calls `dify_search` with the subject
text and returns the top-k chunks as corpus `sources`. Each source carries the
Dify document title and content snippet.

### Notes

- `learn-kb update` for a Dify KB prints a reminder to update the index from
  the Dify UI — there is no local ingestion path for this adapter.
- If Dify is unreachable, `content-grounding-kb.sh` returns an empty sources
  array and logs a warning. The learning loop continues with public corpus
  sources only.

---

## 2. surreal-memory Palace (`type: palace`)

### What it is

A surreal-memory palace is an isolated vector + graph store within the
surreal-memory MCP server. Content is chunked, embedded, and stored locally.
Queries use semantic search (`palace_search` or `palace_recall`) without any
external API calls.

### Prerequisites

- surreal-memory MCP server running and reachable (configured in `.mcp.json`)
- No additional API keys required for palace operations themselves

### How `palace_ingest` works

`palace_ingest` accepts a list of text chunks and a palace ID. It:

1. Embeds each chunk using the local embedding model configured in
   surreal-memory.
2. Stores the embedded chunks in the palace namespace.
3. Returns a count of ingested chunks.

Supported input formats during `add --content-dir`:
- `.md` — Markdown files (chunked at headings)
- `.txt` — Plain text files (chunked by paragraph)
- `.json` — JSON files (each top-level object or array element is one chunk)

### Palace ID naming convention

Default: `kb-<name>` where `<name>` is the `--name` argument.

Example: `--name company-playbook` → palace ID `kb-company-playbook`.

Override with `--palace-id <custom-id>` when integrating with an existing
palace.

### Example `add` command

```
/learn-kb add --type palace --name company-playbook \
  --content-dir ~/docs/playbook \
  --description "Company sales methodology and playbook"
```

To register an existing palace without ingesting:

```
/learn-kb add --type palace --name existing-palace \
  --palace-id my-custom-palace-id
```

### Updating a palace KB

```
/learn-kb update --name company-playbook --content-dir ~/docs/playbook
```

This re-runs `palace_ingest` on the directory. Existing chunks are not
deduplicated automatically — for a full refresh, `remove --purge-palace` first,
then `add` again.

### Purging

`remove --purge-palace` calls `palace_delete` on the palace ID, which removes
all embedded chunks and the palace namespace from surreal-memory.

---

## 3. Local directory (`type: local`)

### What it is

A local KB registers a filesystem directory. At query time, `content-grounding-kb.sh`
reads files from the directory, filters by subject relevance using simple
keyword matching, and returns matching content as corpus sources. No ingestion
or embedding step occurs — files are read fresh at each query.

### Prerequisites

- None beyond filesystem read access to the directory
- No external services required

### Supported file formats

| Format | How it is read |
|---|---|
| `.md` | Read as-is; headings used as source title |
| `.txt` | Read as-is; filename used as source title |
| `.json` | Parsed; top-level keys used as source labels |

Files with other extensions are ignored.

### Directory structure (recommended)

```
~/docs/my-kb/
├── overview.md
├── api-reference.md
├── runbook-deployments.md
├── glossary.txt
└── schema.json
```

Flat directories work best. Subdirectories are read recursively but file count
above 200 triggers a warning.

### Example `add` command

```
/learn-kb add --type local --name architecture-docs \
  --content-dir ~/projects/myapp/docs \
  --description "Internal architecture documentation"
```

### Update behavior

`learn-kb update --name architecture-docs` re-scans the registered directory
path. If the directory no longer exists, the command prints an error. The
registry entry is NOT automatically removed — use `remove` explicitly.

### Notes

- For large or frequently-updated directories, consider migrating to a palace
  KB for faster semantic search.
- Binary files (`.pdf`, `.docx`) are not supported. Convert to `.md` or `.txt`
  before registering.

---

## 4. URL scraping (`type: url`)

### What it is

The `url` adapter uses Firecrawl to scrape a URL (or site) during `add` or
`update`, then ingests the scraped content into a surreal-memory palace. After
ingestion, all query-time access hits the palace only. The source URL is stored
in the registry for future `update` calls but is NOT re-fetched at query time.

### Prerequisites

| Variable | Description |
|---|---|
| `FIRECRAWL_API_KEY` | API key from firecrawl.dev |

The key is used only during `add` and `update`, not at query time.

### What gets scraped

By default, `firecrawl_scrape` fetches the single URL and converts it to
clean Markdown. For documentation sites with many pages, use
`firecrawl_crawl` instead — this is done automatically when the URL contains
a path prefix pattern (e.g., `/docs/`).

The scraped content is chunked by heading level and ingested into a palace with
ID `kb-<name>` (or `--palace-id <custom-id>`).

### Example `add` command

```
/learn-kb add --type url --name openai-api-docs \
  --url https://platform.openai.com/docs \
  --description "OpenAI API reference scraped via Firecrawl"
```

### Re-scrape (update)

```
/learn-kb update --name openai-api-docs
```

This re-scrapes the registered URL and re-ingests the result into the palace.
Existing palace content is replaced (not appended) to avoid stale duplicates.

### Privacy note

Firecrawl is called only during `add` and `update`. Between those calls, all
queries are served from the local palace. If `FIRECRAWL_API_KEY` is a cloud
key, the URL content is sent to Firecrawl's servers during the scrape step.
Operators who require fully local operation should use the `local` or `palace`
adapter instead and manually convert web content to local files.

---

## Adapter comparison

| Capability | dify | palace | local | url |
|---|---|---|---|---|
| Semantic search | Yes (Dify) | Yes (palace) | No (keyword) | Yes (palace) |
| Requires external service | Yes (Dify) | No | No | Once (Firecrawl) |
| Local-only after setup | No | Yes | Yes | Yes |
| Ingestion step | No | Yes | No | Yes (on add/update) |
| Supports incremental updates | Via Dify UI | Via re-ingest | Via re-scan | Via re-scrape |
| Purge supported | No | Yes | No | Yes |
