# Plan — phase-ci-all-green

_Backend: **OpenSpec** (`openspec/` present). Goal: all 4 README badges / every
`validate.yml` job GREEN on `main`._

## What Plan discovered that Assess could not

Assess flagged the risk that fixing each first-gate failure might expose
second-order failures. **Plan reproduced all three locally and confirmed that
risk is real for BDD:**

- The tsx loader fix makes the BDD suite *run*, but it then reports
  **13 scenarios: 5 failed, 6 undefined, 2 passed.**
- **6 undefined** are entirely inside `tests/features/drafts/okf-wiki-ingest.feature`
  — a **draft** feature with no step definitions. CI's glob
  `tests/features/**/*.feature` sweeps `drafts/` in, so drafts fail CI.
- **5 failed** are real behavioral mismatches in `forge-validate.feature` /
  `forge-enrich.feature`: `forge validate` **exits 0 when the steps require a
  non-zero exit** (e.g. unknown language prints `Error: invalid language
  'cobol'` but returns exit 0), and validate emits `No constitution found`
  paths the steps don't expect.

Per the CLAUDE.md **BDD immutable-tests rule**, `tests/steps/*.ts` and
non-draft `tests/features/*.feature` are the contract and MUST NOT be edited to
force a pass — the **`forge` binary must be fixed** to satisfy them, or the
failures surfaced. So GAP-1 is **not** a one-line loader change; it splits into
loader + real forge fixes.

GAP-2 (forge-rs fmt) and GAP-3 (prettier) are cosmetic and low-risk as assessed.

## Ordered change list (5 changes)

Order = safest/cosmetic first, then the real forge behavior work. Each change is
independently verifiable and independently mergeable; a change is DONE only when
its target CI job passes on the branch — never on "the gate stopped erroring."

### change-green-001 — Check Formatting → green  _(GAP-3)_
- **What:** Add generated site output to `.prettierignore` (`site/.docusaurus/`,
  `site/build/`, and `site/.docusaurus` cache), then `prettier --write` the ~17
  genuinely-authored files (`.mcp.json`, `.claude-plugin/marketplace.json`,
  `CONTRIBUTING.md`, `SKILLS.md`, the openspec specs, `tests/*`, `shared/*`, the
  1 memory file).
- **Decision (OQ-2):** ignore `site/.docusaurus/` and `site/build/` only, NOT
  all of `site/` — hand-authored Docusaurus config/components under `site/src`
  SHOULD stay linted. Verify none of the remaining flagged files are generated.
- **Verify:** `npx prettier --check --ignore-path .prettierignore "**/*.{md,json,js,ts}"` exits 0.
- **Risk:** LOW · **no second-order failures possible** (formatting only).
- **Agent:** direct edit.

### change-green-002 — forge-rs formatting → unblock fmt gate  _(GAP-2, part 1)_
- **What:** `cd tools/forge-rs && cargo fmt --all`; commit the reformatted files.
  `tools/forge-rs` is **vendored in this repo** (not a submodule) → commits here.
- **Verify:** `cd tools/forge-rs && cargo fmt --check --all` exits 0.
- **Risk:** LOW.
- **Agent:** direct + `rust-build-resolver` only if fmt reveals a parse issue.

### change-green-003 — forge-rs clippy + tests → green  _(GAP-2, part 2)_
- **What:** The `forge-rs-test` job runs fmt → clippy `-D warnings` → tests. Only
  fmt has been observed so far. Run `cargo clippy --all-targets -- -D warnings`
  and `cargo test --all` locally in `tools/forge-rs`; fix whatever real issues
  surface (this is the "second-order" step for GAP-2).
- **Verify:** both commands exit 0 locally; `forge-rs-test` job green on branch.
- **Risk:** MEDIUM (unknown until clippy/test run) · **must not be skipped or
  assumed.**
- **Agent:** `rust-reviewer` / `rust-build-resolver`.

### change-green-004 — BDD loader + draft exclusion → suite runs, drafts don't gate  _(GAP-1, part 1)_
- **What:**
  1. Replace the `ts-node/register` loader in the `cucumber` npm script with the
     already-present **tsx** ESM loader (verified locally:
     `NODE_OPTIONS="--import tsx" cucumber-js …` runs the suite). Add `ts-node`
     is NOT needed.
  2. Stop CI from running **draft** features: scope the cucumber glob to the
     real features (exclude `tests/features/drafts/**`) via a `cucumber.mjs`
     config profile or an explicit glob. This is allowed — drafts are, by the
     immutable-tests rule, the sanctioned home for not-yet-implemented features;
     they must not gate CI. **Do NOT delete the draft or add stub steps to force
     it green.**
- **Verify:** `npm run cucumber` executes only the 2 real feature files and the
  undefined-step count is 0.
- **Risk:** LOW-MEDIUM · leaves the 5 real forge failures (→ change-005).
- **Agent:** direct edit (config only — NOT steps/features).

### change-green-005 — forge validate/enrich behavior → BDD scenarios pass  _(GAP-1, part 2)_
- **What:** Fix the **`forge` binary** so the 5 immutable scenarios pass:
  - `forge validate` must **exit non-zero** on validation error (unknown
    language, constitution violation) — currently exits 0. Source:
    `tools/forge-rs/crates/forge-cli/src/main.rs` (validate arm) +
    `forge-core`.
  - Reconcile the missing-constitution / output-string expectations with what
    the committed steps assert (steps scaffold `.forge/constitution/rust.toml`
    and pass `--project-root`).
  - **Immutable-tests rule:** fix forge, NOT the steps. If any scenario encodes
    a genuinely wrong expectation, STOP and surface it to the user rather than
    editing the step.
- **Verify:** `NODE_OPTIONS="--import tsx" FORGE_BIN=…debug/forge npm run cucumber`
  → all real scenarios pass, exit 0; `bdd-test` job green on branch.
- **Risk:** **HIGH** — real behavioral change to a shipped binary; touches
  `forge-rs` which is also consumed by the forge-mcp service. Must re-verify
  forge-mcp + `Check Rust CLI` + `forge-rs-test` still pass after the exit-code
  change.
- **Agent:** `rust-reviewer` + `tdd-guide`; adversarial check that the exit-code
  change doesn't break other forge consumers.

## Sequencing & PR strategy

- **001, 002** can land immediately (cosmetic, zero risk) — could be one quick PR.
- **003** after 002 (same job).
- **004** independent; **005** depends on 004 (need the suite running to verify).
- Recommend **two PRs**: PR-A = {001, 002, 003} (formatting + forge-rs job), PR-B
  = {004, 005} (BDD job, higher risk, needs careful forge review).
- After each PR merges, confirm the corresponding `validate.yml` job is green on
  `main`. Badges flip green only when the WHOLE workflow passes — so the last
  merge is what turns them all green.

## Open questions carried into Execute

1. **change-005 scope risk:** does making `forge validate` exit non-zero break
   any other caller (forge-mcp, hooks, `Check Rust CLI`)? Must verify, not assume.
2. **cross-model-qa.yml** is red but **unbadged** — out of scope for the badges;
   defer to a follow-up phase unless you want it included.
3. Whether any of the 5 forge scenarios encodes a wrong expectation (→ surface,
   don't edit).

## First change to apply

**change-green-001** (Check Formatting) — lowest risk, no second-order failures,
immediate visible progress. Then 002 → 003 → 004 → 005.
