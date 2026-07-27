### change-dgp-007 — Substrate crate pages
`/opsx:new change-dgp-007`
Add site pages for storage-provider, learner-model, surface-bridge,
sovereign-client, prometheus-research (sovereign-sync section already
exists) — sourced from crate READMEs/CLAUDE.md architecture sections, not
duplicated verbatim.
Acceptance: five new pages under the substrate section; internal links to
sovereign-sync section resolve; **non-duplication**: each page links to its
canonical source (crate README path) and contains no verbatim-copied
sections >3 paragraphs (summaries + links, not forks); the **existing
sovereign-sync pages get the same audit** (canonical-source links added,
no verbatim sections >3 paragraphs); build green.
Agent: build. | library: —


## Evidence (implemented 2026-07-27)
- 5 new pages + index under /docs/substrate/; build exit 0; routes verified.
- Non-duplication: every page ≤3 paragraphs of summary + canonical-source link
  (crate tree / README); content condensed from CLAUDE.md substrate section and
  crate lib.rs doc headers, not copied.
- Existing sovereign-sync audit: pages are site-original (assess/analyze
  provenance finding — no canonical counterpart existed to copy); canonical
  source links added to overview.md and architecture.md.
- Gates: skipped (docs-only change per heuristic).
