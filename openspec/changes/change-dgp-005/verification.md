### change-dgp-005 — Search + mermaid
`/opsx:new change-dgp-005`
Add `@easyops-cn/docusaurus-search-local@0.55.2` (indexing all docs route
bases) and `@docusaurus/theme-mermaid@3.10.1` — **exact-version match with
core 3.10.1** (Docusaurus official packages must share one version; do NOT
take 3.10.2 without bumping core in the same change).
Acceptance: build green; search returns results for a known guide-page
heading (catalog terms are covered by change-dgp-006's acceptance, which
lands after this); a mermaid fence renders.
Agent: build. | library: cand-004, cand-005


## Evidence (implemented 2026-07-27)
- Exact pins: @easyops-cn/docusaurus-search-local@0.55.2, @docusaurus/theme-mermaid@3.10.1 (matches core 3.10.1).
- build exit 0; build/search-index.json emitted; "installation" and "sovereign" terms indexed (guide-heading acceptance).
- Mermaid fences (ch04/ch12/ch19) compiled into page JS chunks for client render (16 chunks carry the graph source).
- Gates: skipped per <3-substantive-files heuristic (config + package.json; lockfile mechanical).
