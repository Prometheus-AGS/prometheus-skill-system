---
id: change-okf-003-permissive-okf-parser
title: OKF §9 permissive parser + reserved-filename handling in pk-store
phase: phase-okf-llm-wiki-adoption
gaps: [Goal1, Goal2]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: [change-okf-002]
scope:
  - prometheus-knowledge-rs/pk-store/src/markdown.rs
  - prometheus-knowledge-rs/pk-store/src/store.rs
  - prometheus-knowledge-rs/pk-core/src/types.rs
---

# change-okf-003 — Permissive OKF parser

## Context

markdown_to_entry is struct-strict: a valid OKF doc missing `revision`/`id`
fails to parse. OKF §9 requires consumers to tolerate missing optional fields
and unknown keys. Store load treats every .md as an entry, so an OKF index.md
(no frontmatter) would break KB load. Parser must land BEFORE the writer
change (004) to keep round-tripping safe.

## Tasks

- [ ] Parse frontmatter into a permissive doc struct: only `type` required
- [ ] Preserve unknown frontmatter keys on round-trip (serde_yaml Value map)
- [ ] Derive missing pk fields: id from wiki-relative path, timestamps
      defaulted to file mtime/now, revision=1
- [ ] Skip reserved filenames index.md and log.md in store load
- [ ] Back-compat: legacy pk frontmatter (id/title/tags/links/...) still parses
- [ ] Verify: unit tests — OKF minimal doc parses; legacy doc parses; unknown
      keys round-trip; index.md/log.md skipped
