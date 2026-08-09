---
license: MIT
name: llm-wiki
version: '1.0.0'
description: >
  Operate a persistent, LLM-maintained knowledge wiki through pk: ingest sources
  into OKF-formatted wiki pages (with index/log maintenance), query the wiki and
  file good answers back as pages, and lint for contradictions, orphans, stale
  claims, and missing cross-references. Use when asked to add knowledge to the
  wiki, ingest a document or conversation, answer from the knowledge base with
  citations, or health-check the wiki.
metadata:
  tags: [documentation, knowledge, wiki, okf, pk]
---

# LLM Wiki Skill

Maintain a persistent, compounding knowledge wiki — the Karpathy LLM-wiki
pattern implemented on pk (prometheus-knowledge-rs) with the Open Knowledge
Format (OKF v0.1). The wiki is not a RAG cache: every source is compiled once
into interlinked markdown pages that stay current, and every good answer is
filed back so exploration compounds.

Three layers (who writes what):

| Layer | What | Who writes it |
|---|---|---|
| Raw sources | Documents, transcripts, notes fed to ingest | The user (immutable — never edit) |
| The wiki | OKF concept pages + `index.md` + `log.md` under the KB dir | pk + this skill (never the user) |
| The schema | [references/wiki-schema.md](references/wiki-schema.md) — structure, conventions, workflows | Co-evolved, versioned in this repo |

Wiki location: `<project_root>/.prometheus/knowledge/` inside a project,
`~/.prometheus/knowledge/` (or `$PK_KB_DIR`) otherwise. All pages conform to
OKF v0.1 (`shared/references/okf-v0.1.md`); the producer checklist is
[references/okf-conformance.md](references/okf-conformance.md).

## Tool access

Prefer the `knowledge_*` MCP tools when the prometheus-knowledge server is
connected; fall back to the `pk` CLI (same engine, same KB):

| Operation | MCP tool | CLI |
|---|---|---|
| Ingest | `knowledge_ingest` | `pk ingest [file] [--kb-dir DIR]` (reads stdin without a file) |
| Search | `knowledge_search` | `pk search "<query>" [--kb-dir DIR]` |
| Focused brief | `knowledge_focus` | `pk focus "<topic>" [--k N] [--kb-dir DIR]` |
| Read one page | `knowledge_get` | `pk get <concept-id> [--kb-dir DIR]` |
| Lint | `knowledge_lint` | `pk lint [--kb-dir DIR]` |
| Inventory | — | `pk list`, `pk stats` |

pk's LLM calls route via `~/.prometheus/.env` (`CLOUD_LLM_URL`,
`PK_COMPILE_MODEL`, …). If ingest fails with an LLM error, check that file
and the endpoint it names before debugging pk itself.

## Operation: ingest

Compile a source into the wiki. A single source may touch many pages — that
is the point.

1. Read the source. Do not modify it (raw sources are immutable).
2. Run it through pk: `pk ingest <file>` or pipe text to `pk ingest`.
   pk compiles it into an OKF concept page — frontmatter `type` (default
   `Reference`), `title`, `tags`, `timestamp`, plus pk extension keys
   (`id`, `sources`, `created_at`, `updated_at`, `revision`) — and assigns
   the concept ID from the wiki-relative path. Ensure `index.md` and
   `log.md` reflect the ingest (pk maintains them automatically where
   supported; otherwise update them per the schema doc's formats), and add
   a `description` to the frontmatter if pk did not emit one.
3. Integrate, don't just append. Search the wiki (`pk search`) for pages the
   new knowledge touches: update entity/concept pages, add body cross-links
   (bundle-relative form: `[title](/path/page.md)`), and note explicitly
   where new information **contradicts** existing claims — flag it in both
   pages rather than silently overwriting.
4. Cite. Claims sourced from external material get a `# Citations` section
   (numbered links) at the bottom of the page.
5. Report back: which pages were created, which updated, what contradictions
   or gaps surfaced.

Ingest sources one at a time when the user is engaged (discuss takeaways,
let them steer emphasis); batch only when explicitly asked.

## Operation: query

Answer questions from the wiki, and make good answers permanent.

1. Orient from the top: read the root `index.md` first (progressive
   disclosure), then `pk search` / `pk focus` for candidate pages, then read
   the pages themselves. Do not answer from memory of the sources.
2. Synthesize an answer **with citations** — link every load-bearing claim
   to the wiki page (bundle-relative link) or external source that backs it.
3. File good answers back. If the answer requires real synthesis — a
   comparison, an analysis, a connection not written down anywhere — save it
   as a new wiki page via ingest (`type: Analysis` or similar), so the
   exploration compounds instead of vanishing into chat history.
4. If the wiki cannot answer, say so and name the missing knowledge — that
   gap is an ingest candidate, not something to paper over.

## Operation: lint

Periodic health check. Run `pk lint` for mechanical OKF conformance, then do
the semantic pass pk cannot:

- **Contradictions** — pages making incompatible claims; resolve or
  cross-flag both sides.
- **Stale claims** — statements superseded by newer ingests (compare
  `timestamp` fields and `log.md` recency).
- **Orphan pages** — no inbound links from any other page or index; either
  link them in or propose merging.
- **Missing pages** — concepts referenced repeatedly in bodies but lacking
  their own page (broken bundle-relative links are legal OKF and often mark
  exactly these).
- **Missing cross-references** — related pages that never link each other.
- **Data gaps** — questions the wiki raises but cannot answer; suggest
  sources or web searches to fill them.

Emit findings as a prioritized list; fix mechanically what the user
approves, and append a lint entry to `log.md`.

## Bookkeeping invariants

Whatever the operation, leave the wiki consistent:

- Every non-reserved page has parseable YAML frontmatter with a non-empty
  `type` (OKF §9 — the only hard conformance rules).
- `index.md` reflects every page (link + one-line description from its
  frontmatter); `log.md` gets an entry per operation under a `## YYYY-MM-DD`
  date heading, newest first, each starting `**Ingest**` / `**Query**` /
  `**Lint**`, so `grep "^## " log.md | head -5` surfaces recent activity.
- Cross-links use the bundle-relative form (`/path/page.md`). Broken links
  are tolerated (they mark not-yet-written knowledge) — but never create one
  without intending the target page.
- Unknown frontmatter keys and unknown `type` values are preserved, never
  stripped (permissive consumption, OKF §9).

Full conventions: [references/wiki-schema.md](references/wiki-schema.md).
Producer checklist: [references/okf-conformance.md](references/okf-conformance.md).
