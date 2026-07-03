# Assessment — phase-ci-all-green

_Generated: 2026-07-03 · Assessed against goal: **all README status badges / CI checks GREEN on `main`**_

## Goal

The 4 README badges ("Validate Skills — failing" ×4) all point at the single
`validate.yml` workflow (whole-workflow badge + 3 job-scoped badges:
`forge-rs-test`, `bdd-test`, `secret-scan`). A GitHub workflow badge renders
**failing if ANY job in the workflow fails**. So all four badges go green the
moment `validate.yml` passes end-to-end on `main`.

## Current state (latest `main` run, sha `a77f50e`)

`validate.yml` overall = **failure**. Per-job:

| Job | Status | Notes |
|---|---|---|
| Build sycophancy binary + real gate e2e | ✅ | |
| Validate AgentSkills.io Compliance | ✅ | |
| Check Rust CLI | ✅ | |
| Check hooks.json symlink integrity | ✅ | |
| Skill description collision detection | ✅ | |
| Secret scanning (gitleaks) | ✅ | fixed in PR #22 (CLI, no license) |
| **Check Formatting** | ❌ | prettier |
| **forge-rs (fmt + clippy + test)** | ❌ | `cargo fmt --check` |
| **BDD tests (forge validate + enrich)** | ❌ | missing `ts-node` |

6 of 9 jobs pass. **3 jobs block all 4 badges.**

Not badged in README but also red: `cross-model-qa.yml` (out of scope for the
badges; flag as a follow-up).

## Root causes (verified from CI logs + reproduced locally)

### GAP-1 — BDD tests: `Cannot find module 'ts-node/register'`
- **Cause:** `package.json` `cucumber` script runs
  `cucumber-js … --require-module ts-node/register`, but `ts-node` is **not**
  a declared dependency and is not installed. `tsx@^4.22.3` **is** already a
  devDependency and is the loader used everywhere else in this repo
  (`npx tsx scripts/…`).
- **Not a real test failure** — the suite never starts; it dies at loader
  resolution.
- **Fix (smallest correct):** switch the cucumber invocation to the
  already-present tsx ESM loader (or add `ts-node` as a devDependency). Prefer
  tsx for consistency with the rest of the repo. Confirm scenarios actually run
  and pass afterward (they may surface real step failures once the loader
  works — must verify, not assume).
- **Severity: HIGH** (blocks a badge) · **Effort: LOW**

### GAP-2 — forge-rs: `cargo fmt --check --all` finds 53 diffs
- **Cause:** `tools/forge-rs/crates/forge-cli/src/main.rs`,
  `forge-core/src/lib.rs`, and others are not rustfmt-formatted. The
  `forge-rs-test` job runs `cargo fmt --check` (working-directory
  `tools/forge-rs`) and fails before clippy/test even run.
- **`tools/forge-rs` is VENDORED, not a submodule** (absent from `.gitmodules`;
  its git remote is the parent repo). So `cargo fmt` fixes commit **directly to
  this repo** — no submodule PR needed. Same for `tools/prometheus-cli`.
- **Fix:** `cd tools/forge-rs && cargo fmt --all`, commit the reformatted files.
  Then confirm the job's later steps (clippy `-D warnings`, tests) also pass —
  fmt was the first gate; clippy/test have not been observed yet and must be
  verified.
- **Severity: HIGH** (blocks a badge) · **Effort: LOW for fmt; MEDIUM if clippy/test surface issues**

### GAP-3 — Check Formatting: prettier flags 123 files
- **Cause:** `prettier --check --ignore-path .prettierignore "**/*.{md,json,js,ts}"`
  flags 123 files. Breakdown: **106 in `site/` (generated `.docusaurus/*` build
  output)**, 5 openspec specs, 4 tests, plus `.mcp.json`,
  `.claude-plugin/marketplace.json`, `CONTRIBUTING.md`, `SKILLS.md`, 1 memory,
  2 shared.
- The **106 `site/` files are generated Docusaurus build artifacts** that
  shouldn't be linted at all — `site/` is not in `.prettierignore` (which
  already ignores `node_modules`, `target`, `docs`, etc.).
- **Fix (two parts):**
  1. Add `site/` (or at least `site/.docusaurus/` + `site/build/`) to
     `.prettierignore` — generated output must not gate CI. Verify the site
     source that IS meant to be linted (if any) is handled intentionally.
  2. `prettier --write` the ~17 genuine source files (config JSON, docs,
     openspec specs, tests) so they conform.
- **Severity: HIGH** (blocks a badge) · **Effort: LOW**

## Cross-cutting risks / open questions for Plan

1. **Hidden second-order failures.** GAP-1 and GAP-2 currently fail at the
   *first* gate (loader / fmt). Fixing that gate can expose real BDD step
   failures or clippy `-D warnings` / test failures underneath. The plan MUST
   treat "make the gate pass" and "make the real checks pass" as separate,
   verified steps — do not claim green until a full local/CI run is observed.
2. **`.prettierignore` scope decision.** Ignoring all of `site/` is correct for
   generated output, but confirm no hand-authored source under `site/` needs
   formatting coverage (Docusaurus config, custom components). Open question for
   Plan: ignore `site/` wholesale vs. only `site/.docusaurus` + `site/build`.
3. **BDD immutable-tests rule (CLAUDE.md).** Code-gen agents may NOT edit
   existing `tests/steps/*.ts` / `tests/features/*.feature` to make failing
   tests pass. GAP-1's fix is a *loader/config* change (package.json + maybe
   the workflow), which is allowed; but if fixing the loader surfaces real step
   failures, those must be surfaced to the user, not silently patched by editing
   steps.
4. **`cross-model-qa.yml`** is red but unbadged. Decide in Plan whether to
   include it (stretch) or defer.
5. **Determinism.** BDD/forge jobs build Rust + run network-ish steps; watch for
   flakiness vs. real failures when validating.

## Recommended sequencing (for Plan)

All three gaps are independent and low-effort. Suggested order (each verified
locally before the next):
1. **GAP-3 Check Formatting** — `.prettierignore` + `prettier --write` (safest,
   no behavior change)
2. **GAP-2 forge-rs fmt** — `cargo fmt`, then verify clippy + test
3. **GAP-1 BDD loader** — switch to tsx, then verify scenarios actually pass

Then one PR (or three small ones) → confirm `validate.yml` green on the branch →
merge → confirm green on `main` → badges flip green.

## Stage handoff

Key gaps: 3 independent CI jobs block all 4 badges — BDD `ts-node` loader
missing (use existing tsx), forge-rs `cargo fmt` drift (vendored, commits
locally), prettier flags 123 files (106 are generated `site/` output → ignore +
format the ~17 real ones). Open for Plan: whether fixing each first-gate failure
exposes real second-order failures (clippy/test/steps) that need their own fix,
and the `.prettierignore` scope for `site/`.
