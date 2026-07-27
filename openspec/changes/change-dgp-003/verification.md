### change-dgp-003 — Canonical serving + guide reconciliation + README mapping
`/opsx:new change-dgp-003`
Reconcile the diverged `site/docs/guide/*` ↔ `docs/guide/01-*.md` fork:
merge content into canonical `docs/guide/`, delete the site copies, add a
`plugin-content-docs` instance (`path: '../docs/guide'`) per the donor
pattern; keep sidebar order via the numbered filenames or explicit sidebar.
**README mapping (Goal 3)**: audit `README.md` (512 lines) section-by-section;
each major section must be either represented in a guide page or explicitly
out-of-site (recorded in the change's tasks); add a site link to README's
header. CLAUDE.md is agent-operational and stays off-site except the
architecture/commands material already mirrored in `docs/guide` (see
change-dgp-008 for the KBD portion).
Acceptance: no file under `site/docs/guide/`; guide pages render from
`docs/guide` sources; README section→site mapping table **and CLAUDE.md
section→(site page | out-of-site rationale) mapping table** committed in
the change (the "already mirrored" claim is audited, not assumed); build
green.
Agent: build. | library: cand-003


## Reconciliation audit (round-1 CRITICAL resolution)
Direction: canonical docs/guide (24 chapters, 90–235 lines each) retained; the
11 deleted site pages were derivative condensations (22–60 lines). Containment
audit run per page; flagged "missing" lines verified as rephrasings of facts
present in canonical (e.g. ch19 L75 lists every platform install destination;
ch01 covers all skill domains). No substantive site-only content existed
except quick-start.md, which was MOVED to docs/guide/00-quick-start.md (URLs
corrected to Prometheus-AGS/prometheus-skill-system in the move).
onBrokenLinks: 'throw' audit = the live green build over the full 25-page tree.
