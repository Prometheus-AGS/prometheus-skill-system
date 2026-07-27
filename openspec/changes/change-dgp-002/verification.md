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


NOTE (round-2 SUGGESTION): Pages MUST be enabled (Settings → Pages → Source: GitHub Actions) BEFORE this change merges to main, else main runs stay red until enablement. This ordering is already a plan-level manual task.
