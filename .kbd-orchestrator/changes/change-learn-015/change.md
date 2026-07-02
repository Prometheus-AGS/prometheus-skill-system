---
id: change-learn-015
title: learn-kb skill (operator knowledge base management)
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-003
  - change-learn-004
---

# change-learn-015 — learn-kb skill

## Summary

Add a `learn-kb` skill that lets operators manage their own knowledge bases for
grounding the Feynman learning loop. Supports four KB adapters: local file
ingestion via `palace_ingest`, Dify knowledge base integration via
`dify_search`, URL scraping via Firecrawl, and MCP filesystem reads. Exposes
subcommands `add`, `list`, `query`, `update`, and `remove`.

## Motivation

The Feynman loop currently relies on built-in corpus entries. Operators need a
way to inject domain-specific material (internal docs, API references, runbooks)
so that `learn-grade` can assess transfer to their actual problem space.

## Scope

- New skill directory: `skills/learn/learn-kb/`
- Subcommands: `add`, `list`, `query`, `update`, `remove`
- KB adapter reference: `references/kb-types.md`
- No changes to existing skills in this change

## Tasks

- [x] Write `skills/learn/learn-kb/SKILL.md` with frontmatter, overview, and subcommand reference (`add`, `list`, `query`, `update`, `remove`)
- [x] Implement `add` subcommand: ingest local files via `palace_ingest` (accept glob path, recursively walk directories, emit ingestion summary JSON)
- [x] Implement `add` subcommand: ingest Dify KB by ID via `dify_search` integration (accept `--dify-kb <id>`, pull all segments, normalise into grounding-corpus entries)
- [x] Implement `add` subcommand: scrape URL via Firecrawl then ingest result into palace (accept `--url <url>`, call `firecrawl_scrape`, pass markdown to `palace_ingest`)
- [x] Write `skills/learn/learn-kb/references/kb-types.md` documenting local palace vs. Dify vs. MCP filesystem adapters, privacy notes, and schema for `grounding-corpus.json` entries with `source_type: operator_kb`
