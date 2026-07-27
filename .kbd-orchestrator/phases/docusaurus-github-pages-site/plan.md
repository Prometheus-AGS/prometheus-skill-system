# Plan — docusaurus-github-pages-site

Date: 2026-07-27 · Backend: **openspec** (root `openspec/` present)
Inputs: assessment.md, analysis.md, library-candidates.json (7 candidates), decision-log.md.

## Ordering rationale

Config correctness first (everything downstream builds against the right
`url`/`baseUrl`); CI second (every later change gets PR build verification
before any content lands); dedupe/serving third (content changes then happen
in their canonical homes); features and content next; the catalog generator
before the sections that link to it. Deploy is live from change 2 onward —
each merged change ships incrementally.

## Changes (ordered)

### change-dgp-001 — Pages-correct docusaurus config
`/opsx:new change-dgp-001`
Fix `site/docusaurus.config.js`: `organizationName: 'Prometheus-AGS'`,
`projectName: 'prometheus-skill-system'`, default
`url: 'https://prometheus-ags.github.io'` + `baseUrl: '/prometheus-skill-system/'`
parameterized via `SITE_URL`/`BASE_URL` env (donor pattern); fix `editUrl` to
this repo; `onBrokenLinks: 'throw'`, markdown broken-link throw.
Evidence: git remote `Prometheus-AGS/prometheus-skill-system`.
Acceptance: `npm run build` in `site/` succeeds with zero broken-link errors;
built HTML asset URLs carry the baseUrl prefix.
Agent: build. | library: —

### change-dgp-002 — GitHub Pages workflow (vendored)
`/opsx:new change-dgp-002`
Add `.github/workflows/docs-pages.yml` adapted from the donor (vendored YAML,
pinned action SHAs recorded in-repo): PR = build-only; push to main = build →
`upload-pages-artifact` → `deploy-pages@v4` (`github-pages` environment);
Node 24 + npm cache on `site/package-lock.json`; path filters `site/**`,
`docs/**`, workflow file. No collision: existing workflows (cross-model-qa,
prometheus-research, sovereign-sync, validate) touch neither Pages nor
`site/**` paths.
Acceptance (CI-verifiable): PR run builds green; every `uses:` is
SHA-pinned; workflow permissions are least-privilege (`contents: read` at
top; `pages: write`/`id-token: write` only in the deploy job).
Acceptance (operator-verified, after Pages is enabled): main run deploys;
site reachable at the Pages URL.
**QA override**: despite being <3 files, this change is security-sensitive
(CI workflow with deploy permissions) — refine-validate AND diff-mode
adversarial review are FORCED, not skipped.
Agent: build. | library: cand-002

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

### change-dgp-006 — Skills-catalog generator
`/opsx:new change-dgp-006`
`site/scripts/generate-skills-catalog.mjs`: walk `skills/*/*/SKILL.md`
frontmatter (name/description/tags/category — same fields
`scripts/validate-skills.js` parses), emit MDX catalog pages (index by
category + per-skill entries) into a generated docs instance; wire into
`site` build script (`prebuild`); **extend the search-local route-base
configuration to include the catalog instance**. Excludes
`skills/imported/` submodules.
Acceptance: catalog lists every non-imported skill (count matches validator's
count); regenerating is idempotent; search index covers the catalog route
base and returns a known skill name; build green.
Agent: build. | library: cand-006

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

### change-dgp-008 — KBD lifecycle section
`/opsx:new change-dgp-008`
Expand the single `metaprompting-pmpo-kbd` guide page into a KBD-lifecycle
section: stages (assess/analyze/spec/plan/execute/reflect), hooks, waypoints,
progress signaling — sourced from the orchestrator SKILL.md.
Acceptance: section with ≥4 pages; mermaid stage diagram renders;
**non-duplication**: pages summarize and link to the canonical orchestrator
SKILL.md / CLAUDE.md sections rather than copying them (no verbatim section
>3 paragraphs); build green.
Agent: build. | library: —

## Deferred pending operator decision (NOT in the executable list)

### change-dgp-009 — knowme --km-* brand port
**Do not create or start without operator approval** — out-of-goal scope per
assess vet; goals amendment required first. Then `/opsx:new change-dgp-009`:
port donor `--km-*` token contract into `site/src/css/custom.css`
(+ optionally the style-contract validator).
Acceptance: tokens match donor contract; dark default; build green.
Agent: build. | library: cand-007

## Manual tasks (operator)

- Enable GitHub Pages (Source: **GitHub Actions**) in repo settings before
  change-dgp-002 merges to main.
- Answer the brand question to unblock or drop change-dgp-009.

## Per-change QA

Standard gates apply per kbd-execute: refine-validate → adversarial-review
(diff mode) → archive. change-dgp-001 is <3 files — QA + adversarial review
auto-skip per heuristic. change-dgp-002 **forces both gates** (see its QA
override — deploy-permission workflow).
