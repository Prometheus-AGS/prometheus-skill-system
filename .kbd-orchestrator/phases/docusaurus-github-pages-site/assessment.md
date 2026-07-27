# Assessment — docusaurus-github-pages-site

Date: 2026-07-27
Reference implementation reviewed: `/Users/gqadonis/Projects/know-me/hybrid-mobile-architecture-skill`
(knowme-brand Docusaurus site + GitHub Pages pipeline, treated as the pattern donor).

## Current state

A partial Docusaurus site **already exists** in this repo at `site/` (48 tracked
files, Docusaurus 3.10.1, classic preset): sections `guide/` (10 pages),
`learn/`, `sovereign-sync/`; a knowme-ish `src/css/custom.css`; `sidebars.js`.
There is **no** docs deploy workflow in `.github/workflows/` (only
cross-model-qa, prometheus-research, sovereign-sync, validate).

## Gap analysis vs goals

### Goal 1 — Docusaurus site for skill-pack docs (skills catalog, KBD lifecycle, learn domain, substrate crates) — PARTIAL

- **Present**: working scaffold; guide pages (introduction, installation,
  quick-start, four-layer-pipeline, language-skills, loop-architecture,
  mcp-substrate, memory-and-learning, metaprompting-pmpo-kbd, process-skills);
  learn/ and sovereign-sync/ sections.
- **Missing content**: no skills-catalog section (the pack's ~140 skills have
  no generated or hand-written index); substrate coverage limited to
  sovereign-sync (storage-provider, learner-model, surface-bridge,
  sovereign-client, prometheus-research undocumented on site); no dedicated
  KBD-lifecycle section beyond one guide page.
- **Brand divergence**: our CSS uses ad-hoc `--knowme-*` tokens (ember
  `#E04E28`) while the reference's canonical contract is the `--km-*` system
  (ember `#a93618` light / `#ff8a63` dark; canvas/chrome/surface/raised
  layers; border/shadow flattening; ember-tinted article links; Inter base +
  Space Grotesk headings — ours sets Space Grotesk for body text too; the
  reference defaults to dark mode). The reference even ships
  `validate:style` (`validate-style-contract.mjs`) to enforce the contract.
- **Config quality gap vs reference**: reference uses `onBrokenLinks: 'throw'`
  (ours `warn`), mermaid theme, `@easyops-cn/docusaurus-search-local`,
  multi-instance `plugin-content-docs`, sitemap, full og/twitter metadata.
  Ours has none of these.

### Goal 2 — GitHub Pages deploy via Actions on push to main — MISSING

- No docs workflow exists. Reference `docs-pages.yml` is a direct template:
  pinned-SHA actions, Node 24 + npm cache, `npm ci` in `site/`, quality gate
  before build, `upload-pages-artifact` + `deploy-pages@v4` with
  `github-pages` environment, PR runs build-only (no deploy).
- **Config mismatches that will break Pages**: `url:
  https://prometheus-skill-pack.prometheusags.ai` with `baseUrl: '/'` only
  works with a custom domain + CNAME; `organizationName: 'prometheusags'`
  (actual org **Prometheus-AGS**), `projectName: 'prometheus-skill-pack'`
  (actual repo **prometheus-skill-system**), `editUrl` points at the wrong
  repo. Without a custom domain the site must use
  `https://prometheus-ags.github.io` + `baseUrl: '/prometheus-skill-system/'`
  (reference parameterizes both via `SITE_URL`/`BASE_URL` env — adopt that).
- `site/package.json` has only stock docusaurus scripts — no link check, no
  built-site validation, nothing to gate a deploy on. Reference's
  `release:check` chain (sanitize → validate → build → validate-built →
  link checks → browser gate) is the donor pattern; at minimum a build +
  internal-link gate is needed.

### Goal 3 — Migrate/link existing docs without duplicating canonical sources — AT RISK

- **Duplication already exists**: `site/docs/guide/*` duplicates
  `docs/guide/01-*.md` content as copies — exactly the drift the goal
  forbids. learn/ and sovereign-sync/ site sections appear to be copies too.
- Reference shows the fix: `plugin-content-docs` instances whose `path`
  points **outside** `site/` at the canonical source (its `prompting` plugin
  reads `../docs/prompting`). Adopting that pattern lets `docs/guide`,
  `docs/learn`, etc. be served directly.
- Root `docs/` content still unmapped to the site: `articles/`,
  `deep-research*`, `codex-plugin.md`, `deployment-modes.md`,
  `QUICK_START.md`, `SUBMODULES.md`, `guide/` (canonical), `learn/`;
  `future-work/`, `plans/`, `BUG-FIX-LEDGER.md` likely should stay
  off-site — inclusion scope is a plan-stage decision.

## Open questions for analyze/plan

1. **Domain**: custom domain (`prometheus-skill-pack.prometheusags.ai` +
   CNAME + Pages custom-domain config) or default
   `prometheus-ags.github.io/prometheus-skill-system/`? Determines
   `url`/`baseUrl` and whether a CNAME file ships.
2. **Dedupe direction**: serve canonical `docs/` via multi-instance plugins
   (reference pattern) vs making `site/docs` canonical and deleting root
   copies. Reference pattern preserves repo-relative links in GitHub UI.
3. **Gate depth**: port reference's full `release:check` suite or start with
   build + `onBrokenLinks: 'throw'` + internal link check.
4. **Brand contract**: adopt the reference `--km-*` token set verbatim (and
   optionally its style-contract validator) or keep the current approximation.
5. **Skills catalog**: generate from skill frontmatter (a build script over
   `skills/*/*/SKILL.md` name/description/tags) vs hand-curated pages —
   generation keeps 140 skills honest but adds a build dependency.

## Verdict

Scaffold exists but all three goals have material gaps: content coverage and
brand contract (G1), the entire deploy pipeline + config corrections (G2),
and an anti-goal duplication pattern already in the tree (G3). Estimated
shape: ~5–7 changes (config fix, brand CSS port, workflow, dedupe/link
migration, skills-catalog generation, substrate section, gate scripts).

---

# Post-execution reassessment — 2026-07-27 (after execute 8/8)

All 8 changes (dgp-001…008) report COMPLETE with per-change verification and
a consolidated clean rebuild (140-skill catalog, 9-route sweep, search index).
This section re-scores the three phase goals against the tree and the live
GitHub state.

## Goal 1 — Docusaurus site for skill-pack docs — MET (locally)

Site now carries `kbd/`, `learn/`, `sovereign-sync/`, and `substrate/`
sections plus the generated skills catalog
(`site/scripts/generate-skills-catalog.mjs`, 140=140 count parity, idempotent,
search-indexed per dgp-006 verification). Multi-sidebar split
(`sidebars-catalog.js`, `sidebars-guide.js`, `sidebars-learn-internals.js`)
in place. Build green under `onBrokenLinks: 'throw'`.

## Goal 2 — GitHub Pages deploy via Actions on push to main — AUTHORED, NOT DEPLOYED

`.github/workflows/docs-pages.yml` exists and is gate-certified (dgp-002:
pinned SHAs, least-privilege, `upload-pages-artifact` + `deploy-pages`), but
deployed reality fails both probes:

- `gh api …/actions/workflows/docs-pages.yml` → **404** — the workflow is not
  on the default branch (it is untracked locally; none of the phase outputs
  have been committed).
- `gh api …/pages` → **404** — GitHub Pages is not enabled on the repo
  (operator action: Settings → Pages → Source: GitHub Actions).

The goal says "on pushes to main"; until commit+push happens and Pages is
enabled, no deploy can occur.

## Goal 3 — Migrate/link docs without duplicating canonical sources — MET (locally)

`site/docs/guide/*` (11 files) deleted in favor of canonical `docs/guide/`;
`quick-start.md` relocated to `docs/guide/00-quick-start.md` (tracked rename);
`kb-adapters.md` now points at the canonical kb-adapter-guide (dgp-004).
No duplicated canonical sources remain in the site tree.

## Residual gaps

1. **Uncommitted phase outputs (blocking Goal 2)**: 46 paths — 16 untracked
   (workflow, `site/docs/kbd/`, `site/docs/substrate/`, `site/scripts/`,
   new sidebars, `openspec/changes/change-dgp-00{1..8}/`), 18 modified,
   11 deletions, 1 rename. Nothing from this phase is on `origin/main`.
2. **Operator blockers (already recorded in progress.json)**: enable GitHub
   Pages (Source: GitHub Actions); brand decision for deferred dgp-009.
3. **Preflight**: liter-llm `needs_configure` — adversarial review continues
   in harness-native isolation (judge model family = producer family), as it
   did for all certified changes this phase.
4. **Stale waypoint state**: `current-waypoint.json` still carries leftover
   fields from phase-learn-grader-validation (changes 7/7, goals 6/6,
   `reflect_complete: true`) that contradict this phase's progress.json
   (8/8, reflect pending) — reset before `/kbd-reflect`.

## Verdict

Implementation COMPLETE (8/8). Goals 1 and 3 are met in the working tree;
Goal 2 is authored and gate-certified where certification was recorded
(dgp-002/003/004/006; the other four changes are verified but carry no
recorded adversarial certification) yet unmet in deployed reality, blocked on
(a) committing and pushing the phase outputs and (b) the operator enabling
Pages. Recommended next actions: commit+push the phase outputs, confirm the
`docs-pages.yml` run deploys, then `/kbd-reflect`.
