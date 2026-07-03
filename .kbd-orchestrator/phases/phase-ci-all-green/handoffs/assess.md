# Handoff: assess → plan (phase-ci-all-green)

3 independent validate.yml jobs block all 4 README badges:
- GAP-1 BDD: `Cannot find module 'ts-node/register'` — use the already-present tsx loader.
- GAP-2 forge-rs: `cargo fmt --check` drift (53 diffs); tools/forge-rs is VENDORED (commits to this repo).
- GAP-3 Check Formatting: prettier flags 123 files — 106 are generated site/ output (add to .prettierignore), ~17 are real source (prettier --write).

OPEN QUESTIONS for plan:
1. Fixing each first-gate failure may expose real second-order failures (clippy -D warnings, tests, BDD steps). Treat gate-pass and real-pass as separate verified steps.
2. .prettierignore scope: all of site/ vs. only site/.docusaurus + site/build.
3. BDD immutable-tests rule: loader/config fix is allowed; real step failures must be surfaced, not silently patched.
4. cross-model-qa.yml is red but unbadged — include or defer.

Read assessment.md for full detail.
