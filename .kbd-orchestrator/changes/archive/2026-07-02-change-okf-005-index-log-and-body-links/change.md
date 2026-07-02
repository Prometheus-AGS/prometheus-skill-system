---
id: change-okf-005-index-log-and-body-links
title: index.md/log.md maintenance on ingest + body cross-links and Citations
phase: phase-okf-llm-wiki-adoption
gaps: [Goal2, Goal3]
priority: P1
effort: L
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: [change-okf-004]
scope:
  - prometheus-knowledge-rs/pk-librarian/src/prompts.rs
  - prometheus-knowledge-rs/pk-librarian/src/
  - prometheus-knowledge-rs/pk-store/src/store.rs
---

# change-okf-005 — Reserved files and body-link conventions

## Context

pk never generates index.md/log.md (Karpathy's two special files; OKF §6–7),
and relationships live in a frontmatter links array instead of bundle-relative
markdown body links (§5) with Citations (§8).

## Tasks

- [ ] Ingest updates wiki-root index.md: catalog grouped by category, entries
      carry frontmatter descriptions (§6)
- [ ] Ingest appends to log.md: `## YYYY-MM-DD` groups, newest first, entry
      prefix convention `**Update**/**Creation**` (§7)
- [ ] Librarian prompts: write bundle-relative markdown body links
      (/path/to/concept.md) instead of frontmatter links array
- [ ] Librarian prompts: emit `# Citations` section mapped from sources
- [ ] Link graph (search/focus) derives edges from body links; frontmatter
      links array read for back-compat only
- [ ] Verify: ingest twice → index.md lists both, log.md has dated entries,
      body links resolve; grep "^## " log.md parseable
