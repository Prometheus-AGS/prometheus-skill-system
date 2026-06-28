# Tasks — change-learn-015

- [ ] Write `skills/learn/learn-kb/SKILL.md` with frontmatter, overview, and subcommand reference (`add`, `list`, `query`, `update`, `remove`)
- [ ] Implement `add` subcommand: ingest local files via `palace_ingest` (accept glob path, recursively walk directories, emit ingestion summary JSON)
- [ ] Implement `add` subcommand: ingest Dify KB by ID via `dify_search` integration (accept `--dify-kb <id>`, pull all segments, normalise into grounding-corpus entries)
- [ ] Implement `add` subcommand: scrape URL via Firecrawl then ingest result into palace (accept `--url <url>`, call `firecrawl_scrape`, pass markdown to `palace_ingest`)
- [ ] Write `skills/learn/learn-kb/references/kb-types.md` documenting local palace vs. Dify vs. MCP filesystem adapters, privacy notes, and schema for `grounding-corpus.json` entries with `source_type: operator_kb`
