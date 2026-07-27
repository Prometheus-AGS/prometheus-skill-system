# Execution — docusaurus-github-pages-site

Backend: **openspec** (root openspec/ present; changes change-dgp-001..008 scaffolded)
Executor: Claude Code session acting as the /kbd-apply driver (per-change signals + progress.json ledger)
Dispatch contract: implement changes in plan.md order; per change — implement →
site build checkpoint → mark implementation complete → QA gates per plan
(dgp-002: QA+adversarial FORCED; docs-only/<3-file changes auto-skip) → archive.
Consolidated validation (full build + link check) after change 8, per the
operator's implementation-first completion mode.
Manual tasks (operator): enable Pages (Source: GitHub Actions); brand decision for deferred dgp-009.
