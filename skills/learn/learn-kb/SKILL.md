---
name: learn-kb
description: Operator knowledge base management for the Feynman learning loop. Lets operators add, list, query, update, and remove custom knowledge bases (Dify, surreal-memory palace, local files, Firecrawl URLs) that ground the learning loop in domain-specific material. KB content never leaves the local environment.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, kb, knowledge-base, dify, palace, grounding, operator, custom-knowledge]
---

# learn-kb

## When to invoke

```
/learn-kb <subcommand> [options]
```

Subcommands: `add`, `list`, `query`, `update`, `remove`

Invoke when an operator wants to manage domain-specific knowledge sources that
ground the Feynman learning loop. Typical triggers:

- "Register our internal API docs as a knowledge base"
- "Add my clinical protocols to the learning loop"
- "List all my knowledge bases"
- "Remove the old runbook KB"

## Use cases

Different operators benefit from different KB types:

- **Lawyers** — add case notes, legal frameworks, statute interpretations
- **Doctors** — add clinical protocols, drug interactions, diagnostic criteria
- **Counselors** — add therapeutic frameworks, client-appropriate psychoeducation
- **Business strategists** — add proprietary methodology, competitive frameworks
- **Engineers** — add internal architecture docs, runbooks, API references

In every case the KB content stays local. Nothing is forwarded to external APIs
during query time. See [Privacy guarantee](#privacy-guarantee) below.

## Subcommands

### `add`

```
/learn-kb add --type dify|palace|local|url --name <kb-name> [options]
```

Registers a new KB entry. The `--name` value is the identifier used in all
other subcommands and in the `--kb` flag of `/learn-goal`.

**Type: `dify`**

Register a Dify knowledge base by name. Queries are routed through
`content-grounding-kb.sh --kb dify:<name>` at query time.

Requirements:
- `DIFY_API_KEY` environment variable set
- `DIFY_BASE_URL` set (default: `http://localhost/v1`)
- A Dify KB already created with the given name

```
/learn-kb add --type dify --name legal-frameworks
```

**Type: `palace`**

Create or register a surreal-memory palace by ID. If `--content-dir` is
provided, immediately ingests all files via `palace_ingest`.

```
/learn-kb add --type palace --name company-playbook \
  --palace-id kb-company-playbook \
  --content-dir ~/docs/playbook
```

Palace IDs follow the convention `kb-<name>` by default when `--palace-id` is
omitted.

**Type: `local`**

Register a local directory as a KB. Content is read at query time (not
ingested). Supported file formats: `.md`, `.txt`, `.json`.

```
/learn-kb add --type local --name architecture-docs \
  --content-dir ~/projects/myapp/docs
```

**Type: `url`**

Scrape a URL via Firecrawl and ingest the result into a surreal-memory palace.
Once ingested, subsequent queries hit the palace only — the URL is not
re-scraped at query time.

Requirements:
- `FIRECRAWL_API_KEY` environment variable set

```
/learn-kb add --type url --name openai-api-docs \
  --url https://platform.openai.com/docs
```

**All `add` calls write a registry entry to `~/.prometheus/learn/kb-registry.json`.**

### `list`

```
/learn-kb list
```

Reads `~/.prometheus/learn/kb-registry.json` and prints a table:

```
NAME                TYPE     CREATED_AT            LAST_QUERIED
legal-frameworks    dify     2024-06-28T10:00:00Z  2024-06-28T11:30:00Z
company-playbook    palace   2024-06-28T09:00:00Z  never
architecture-docs   local    2024-06-27T14:00:00Z  2024-06-28T08:00:00Z
openai-api-docs     url      2024-06-28T10:30:00Z  never
```

If the registry is empty or absent, print: `No knowledge bases registered. Run
/learn-kb add to register one.`

### `query`

```
/learn-kb query --name <kb-name> --subject "query text" [--top-k N]
```

Calls `content-grounding-kb.sh` with the appropriate adapter and prints the
returned corpus JSON. Default `--top-k` is 5.

Under the hood:

```bash
bash "${CLAUDE_PLUGIN_ROOT}/shared/scripts/content-grounding-kb.sh" \
  --kb "<type>:<id>" \
  --subject "<query text>" \
  --level practitioner \
  --output /tmp/learn-kb-query-result.json
cat /tmp/learn-kb-query-result.json | jq .
```

Updates `last_queried` in the registry entry after a successful query.

### `update`

```
/learn-kb update --name <kb-name> [--content-dir <path>] [--url <url>]
```

Re-ingest or refresh a KB:

- **palace**: calls `palace_ingest` on `--content-dir` (required for palace
  type if directory content changed)
- **local**: re-scans the registered directory for new or changed files
- **url**: re-scrapes the registered URL via Firecrawl and re-ingests into
  the palace
- **dify**: no local action needed; Dify manages its own index — prints a
  reminder to update the KB from the Dify UI if needed

### `remove`

```
/learn-kb remove --name <kb-name> [--purge-palace]
```

Removes the KB registry entry from `~/.prometheus/learn/kb-registry.json`.

With `--purge-palace`: also deletes the surreal-memory palace data for `palace`
and `url` type KBs by calling `palace_delete` on the palace ID.

Without `--purge-palace`: the palace data is retained (safe default). Print a
note reminding the operator that palace data remains at the palace ID.

## KB registry schema

The registry lives at `~/.prometheus/learn/kb-registry.json`:

```json
{
  "version": "1.0.0",
  "kbs": [
    {
      "name": "company-playbook",
      "type": "palace",
      "palace_id": "kb-company-playbook",
      "content_dir": "/Users/operator/docs/playbook",
      "created_at": "2024-06-28T10:00:00Z",
      "last_queried": null,
      "description": "Internal business methodology"
    },
    {
      "name": "legal-frameworks",
      "type": "dify",
      "dify_kb_name": "legal-frameworks",
      "created_at": "2024-06-28T10:00:00Z",
      "last_queried": "2024-06-28T11:30:00Z",
      "description": null
    },
    {
      "name": "architecture-docs",
      "type": "local",
      "content_dir": "/Users/operator/projects/myapp/docs",
      "created_at": "2024-06-27T14:00:00Z",
      "last_queried": "2024-06-28T08:00:00Z",
      "description": null
    },
    {
      "name": "openai-api-docs",
      "type": "url",
      "source_url": "https://platform.openai.com/docs",
      "palace_id": "kb-openai-api-docs",
      "created_at": "2024-06-28T10:30:00Z",
      "last_queried": null,
      "description": "OpenAI API reference scraped via Firecrawl"
    }
  ]
}
```

Type-specific optional fields (`palace_id`, `dify_kb_name`, `content_dir`,
`source_url`) are set only for the relevant type. `description` is always
optional. `last_queried` is `null` until the first query.

## Privacy guarantee

All four adapter types respect the privacy rule from `content-grounding-kb.sh`:
KB content is NEVER forwarded to external APIs during query time.

- `dify` — queries run against a local or self-hosted Dify instance. If
  `DIFY_BASE_URL` points to a remote host, that is the operator's explicit
  choice.
- `palace` — queries hit the local surreal-memory server only.
- `local` — files are read from the local filesystem only.
- `url` — Firecrawl scrapes the URL once during `add` or `update`. After
  ingestion into a palace, all subsequent queries hit the palace only. No
  re-scrape occurs at query time.

## Integration with learn-goal

To use a KB in a learning goal:

```
/learn-goal "master our company's sales methodology" --kb company-playbook
```

The `--kb` flag passes the KB name to `content-grounding-kb.sh` automatically.
The resolved KB type and ID are looked up from the registry. The `kb_id` field
in the resulting `goal.json` artifact records which KB was used, so downstream
skills (`learn-survey`, `learn-plan`, `learn-grade`) can re-query the same KB
without prompting the operator again.

## Error handling

| Condition | Behavior |
|---|---|
| Registry file missing | Create it with empty `kbs: []` on first `add` |
| Name already exists | Print error; suggest `update` or `remove` first |
| Type-specific env var missing | Print which var is needed; abort |
| `--name` not found in registry | Print available names; abort |
| palace_ingest fails | Print error with palace ID; registry entry is NOT written |

## Detailed reference

For in-depth information on each adapter type, environment requirements, and
ingestion mechanics, see [references/kb-types.md](references/kb-types.md).

## Directory layout

```
skills/learn/learn-kb/
├── SKILL.md              — this file
└── references/
    └── kb-types.md       — adapter-type deep-dive
```
