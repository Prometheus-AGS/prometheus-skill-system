# Tasks — change-bdd-005-lifecycle-loop-skill

- [ ] Create skills/testing/bdd-lifecycle-loop/ directory
- [ ] Write SKILL.md with frontmatter and four-phase loop overview
- [ ] Document outside-in authoring workflow (write failing feature first, iterate on steps)
- [ ] Write scripts/flake-budget.sh (reads flake-budget.json, wraps cucumber --retry-tag-filter @flaky)
- [ ] Write scripts/test-file-diff-guard.sh (fails PRs where tests/steps/** or tests/features/** are modified without approved label)
- [ ] Write references/immutable-tests.md pointing at shared/scripts/protect-tests.sh and BDD-006 doc
- [ ] Write references/visual-baseline-refresh.md documenting Playwright --update-snapshots workflow with paper trail
- [ ] Commit the change
