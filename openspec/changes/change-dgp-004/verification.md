### change-dgp-004 — Learn linking + kb-adapter merge
`/opsx:new change-dgp-004`
Serve canonical `docs/learn` as a "Learn internals" docs instance
(`path: '../docs/learn'`); keep `site/docs/learn` narrative pages; merge the
overlapping kb-adapter pages (`docs/learn/kb-adapter-guide.md` ↔
`site/docs/learn/kb-adapters.md`) into one canonical page with a cross-link
from the other section.
Acceptance: both learn sections render; exactly one kb-adapter page remains
canonical; build green.
Agent: build. | library: cand-003


## Vet evidence (round 1, PASS)
- Merge audit: canonical kb-adapter-guide.md "Privacy Guarantee" (L27-42) is a
  SUPERSET of the deleted site content (env-var ignore list incl. FIRECRAWL,
  stderr warning, auditable privacy_mode field) — no content lost by the merge.
- Build green verified live after the change (npm run build; route
  /docs/learn-internals/kb-adapter-guide/index.html exists in build output).
- Cross-link route resolution is guarded by onBrokenLinks: 'throw' at build.

## Gate note
Anti-theater gate accepted at the 2-rejection soft cap: the S-03 detector
flags inline "resolution" annotations inside the findings report as
insufficient criticality. Process lesson for the adversarial-review skill:
keep the judge's report pristine and record operator resolutions in a
separate file (resolutions.md), not inside findings.json.
