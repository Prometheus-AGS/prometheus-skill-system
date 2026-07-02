---
id: change-okf-004-okf-writer-and-id-mapping
title: OKF frontmatter emitter + path-based concept-ID mapping
phase: phase-okf-llm-wiki-adoption
gaps: [Goal1, Goal3]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: [change-okf-003]
scope:
  - prometheus-knowledge-rs/pk-store/src/markdown.rs
  - prometheus-knowledge-rs/pk-core/src/types.rs
  - prometheus-knowledge-rs/pk-store/src/store.rs
---

# change-okf-004 — OKF writer and ID mapping

## Context

entry_to_markdown emits pk-native frontmatter without OKF's required `type`.
OKF concept IDs are wiki-relative paths (minus .md); pk ArticleIds are opaque
slugs in a flat dir. Design decision: concept ID = wiki-relative path;
subdirectories supported; pk fields (revision, sources) kept as producer
extension keys per OKF §4.1 Extensions.

## Tasks

- [ ] Emit OKF frontmatter: type (required), title, description, tags,
      timestamp (= updated_at) + extension keys revision, sources
- [ ] ArticleId ↔ concept-ID mapping: path-based, subdirectory-safe
- [ ] article_path/store write path handles nested concept paths
- [ ] Librarian assigns a `type` on entry creation (default: Reference)
- [ ] Verify: round-trip write→parse identity; OKF §9 conformance of emitted
      docs (external check against vendored spec)
