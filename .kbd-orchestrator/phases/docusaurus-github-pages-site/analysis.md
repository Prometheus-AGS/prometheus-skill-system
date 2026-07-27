# Analysis — docusaurus-github-pages-site

Date: 2026-07-27
Mode: stack specified (Docusaurus 3 + GitHub Pages — fixed by goals and the existing `site/`).
Donor: `/Users/gqadonis/Projects/know-me/hybrid-mobile-architecture-skill` (working knowme-brand Docusaurus + Pages pipeline).

## Evidence gathered this stage

Corrections/verifications against the assess-stage adversarial findings:

- **Org/repo confirmed from git remote**: `git@github.com:Prometheus-AGS/prometheus-skill-system.git`
  → Pages default URL is `https://prometheus-ags.github.io/prometheus-skill-system/`;
  current `site/docusaurus.config.js` values (`organizationName: 'prometheusags'`,
  `projectName: 'prometheus-skill-pack'`, `url: 'https://prometheus-skill-pack.prometheusags.ai'`,
  `baseUrl: '/'`) are all wrong for this repo's Pages deploy.
- **Duplication provenance (vet WARNING resolved)**: only `guide/` is a fork —
  `docs/guide/01-introduction.md` vs `site/docs/guide/introduction.md` **differ**
  (diverged copies, not identical). `docs/learn` and `site/docs/learn` hold
  **different file sets** (partial topic overlap, mostly site-original);
  `site/docs/sovereign-sync/` (12 pages) has **no root canonical counterpart** —
  it is site-original. So G3 remediation = reconcile+dedupe `guide/` only;
  learn/sovereign-sync need linking strategy, not deletion.
- **README/CLAUDE.md (vet WARNING resolved)**: `README.md` 512 lines,
  `CLAUDE.md` 1075 lines. README content (install, quick start, catalog
  summary) maps onto existing guide pages; CLAUDE.md is agent-operational and
  should NOT be published wholesale — derive only the architecture/commands
  sections already mirrored in `docs/guide`. Neither file should be served
  verbatim as a docs page.
- **Workflow inventory (vet WARNING resolved)**: `.github/workflows/` contains
  exactly `cross-model-qa.yml`, `prometheus-research.yml`, `sovereign-sync.yml`,
  `validate.yml` (verified by `ls` this session) — none deploys Pages, none
  path-filters on `site/**`, so a new `docs-pages.yml` collides with nothing.
- **Version constraint**: all official `@docusaurus/*` packages must share an
  identical version — adding theme-mermaid means either
  `@docusaurus/theme-mermaid@3.10.1` (match existing core) or bumping core to
  3.10.2 in the same change; "3.10.x" is not a valid pairing.
- **Donor evidence portability**: donor citations are a local checkout
  (`file:///Users/gqadonis/Projects/know-me/...`); the execute stage must
  vendor the adapted workflow YAML + pinned action SHAs into this repo rather
  than depending on that path existing.
- **Registry health (tier 3, 3 queries)**:
  `@easyops-cn/docusaurus-search-local` 0.55.2 (modified 2026-05-31 — active);
  `@docusaurus/theme-mermaid` 3.10.2 (matches core 3.10.x);
  npm search shows **no existing skills-catalog plugin** — catalog generation
  must be built (small script; can reuse the frontmatter parsing approach of
  `scripts/validate-skills.js`).

## Build-vs-adopt calls

| Gap | Call | Candidate |
|---|---|---|
| G2 deploy workflow | **adapt** donor `docs-pages.yml` (pinned SHAs, Node 24, build gate, `deploy-pages@v4`, PR build-only, `SITE_URL`/`BASE_URL` env parameterization) | cand-002 |
| G2 config fix | **build** (edit): correct org/repo/url/baseUrl per git remote; parameterize via env like donor | — |
| G3 canonical serving | **adopt** donor pattern: `plugin-content-docs` instances with `path` outside `site/` (donor's prompting plugin reads `../docs/prompting`) | cand-003 |
| G3 guide reconciliation | **build** (merge): reconcile diverged `site/docs/guide` ↔ `docs/guide`, keep one canonical home, serve via cand-003 | — |
| G1 skills catalog | **build**: generator over `skills/*/*/SKILL.md` frontmatter (name/description/tags/category) → MDX pages; no npm plugin exists | cand-006 |
| G1 search over 140 skills | **adopt** `@easyops-cn/docusaurus-search-local` | cand-004 |
| G1 diagrams | **adopt** `@docusaurus/theme-mermaid` | cand-005 |
| G1 substrate section | **build** (write): pages for storage-provider, learner-model, surface-bridge, sovereign-client, prometheus-research (sovereign-sync already covered) | — |
| G1 KBD lifecycle | **build** (extend): `site/docs/guide/metaprompting-pmpo-kbd.md` exists but is one page; add a KBD-lifecycle section (assess/analyze/spec/plan/execute/reflect stages, hooks, waypoints) sourced from orchestrator SKILL.md | — |
| G3 learn linking | **build** (link): `docs/learn` (canonical: crdt-conflict-semantics, kb-adapter-guide, meta-corpus) and `site/docs/learn` (site-original: feynman-loop, anti-sycophancy, mastery-criterion, kb-adapters) hold different topics with overlap only on KB adapters — serve `docs/learn` via cand-003 as a "Learn internals" instance, keep site pages as the narrative layer, and merge the two kb-adapter pages into one canonical home | — |
| Brand contract | **reference** donor `--km-*` CSS + `validate-style-contract.mjs` — **out-of-goal**, held for operator decision | cand-007 |

## Decisions taken (see decision-log.md)

1. **Domain**: default GitHub Pages URL, parameterized via `SITE_URL`/`BASE_URL`
   env (donor pattern) — implicit, reversible; a custom domain later is env +
   CNAME only, no code change. Operator may override before execute.
2. **Gate depth**: minimal first — `onBrokenLinks: 'throw'` + docusaurus build +
   internal link check in CI; donor's full `release:check` suite deferred.
3. **Brand**: not scoped into this phase without operator approval (adversarial
   vet flagged it as imported scope). cand-007 recorded as `reference`.

## Open questions

- Operator: adopt knowme `--km-*` brand contract this phase (amend goals) or
  keep current CSS? (Held from assess vet; does not block spec/plan — plan
  will carry it as an optional change gated on approval.)
