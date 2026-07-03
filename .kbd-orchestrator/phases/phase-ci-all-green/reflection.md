# Reflection — phase-ci-all-green

_Generated: 2026-07-03 · Reflect stage. Verified against the completed `main`
run of `validate.yml` (sha `57ec9ef`), not assumed._

## Goal achievement

| Goal | Status |
|---|---|
| All README status badges / `validate.yml` jobs GREEN on `main` | **MET** |

**Verified outcome:** the latest completed `validate.yml` run on `main`
(`57ec9ef`, after PR #24 merged) is `completed/success` — **all 9 jobs green**:
Check Formatting, forge-rs (fmt+clippy+test), BDD tests, gitleaks, Check Rust
CLI, hooks-integrity, sycophancy e2e, skill-collision, AgentSkills compliance.
Since the 4 "Validate Skills" badges are workflow-status badges for
`validate.yml`, they now render green.

**Delta caught during reflect (important):** my first post-merge status check
showed 2 jobs ❌ (BDD, sycophancy) — but that was a **mid-run snapshot** while
jobs were still executing. Waiting for the run to *complete* showed all-green.
Lesson reinforced: never conclude from an in-progress run; a "success"
reflection written off the transient snapshot would have been both wrong and
sycophantic.

## Delivered changes (5/5)

| Change | Job fixed | PR |
|---|---|---|
| 001 Check Formatting | prettier ignore generated `site/` + `tests/`; format ~7 real files | #23 |
| 002 forge-rs fmt | `cargo fmt` across 6 vendored crates | #23 |
| 003 forge-rs clippy+test | cleared 12 `-D warnings` lints across 5 crates | #23 |
| 004 BDD loader + drafts | tsx loader; `cucumber.mjs` excludes `drafts/` | #24 |
| 005 forge behavior + FORGE_BIN | `#[serde(default)]` on Constitution; absolute FORGE_BIN | #24 |

Enabling prerequisite (earlier, same badge): PR #22 replaced the license-gated
gitleaks action with the free CLI.

## Root-cause quality (what the diagnosis actually found)

The plan's central hypothesis — "fixing the first gate exposes real
second-order failures" — held on **every** non-cosmetic change, and in two
cases the *surface* framing was wrong until reproduced:

- **BDD was NOT "forge exits 0 when it should exit 1."** Reproducing `spawnSync`
  showed `status: null` = a spawn ENOENT: CI's **relative** `FORGE_BIN`
  resolved against the step's temp `cwd`. Real fix was a workflow env change,
  not forge logic.
- **The 2 residual BDD failures were a TOML-semantics trap**, not a forge bug in
  spirit: the immutable step writes `required_skills=[]` after a
  `[[forbidden_patterns]]` table, so it binds there and the top-level field is
  absent → `missing field required_skills`. Fixed in forge with
  `#[serde(default)]` (additive leniency), honoring the immutable-tests rule.

Both were found by **reproduction, not inference** (isolated `spawnSync` repro;
a throwaway `toml::from_str::<Constitution>` test that was then removed cleanly).

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with artifact-refiner QA | 0/5 |
| First-pass pass rate | n/a |

No artifact-refiner logs: these were CI-config + lint + small behavior fixes,
driven directly (not as spec'd OpenSpec changes with a refiner gate). QA was
instead **the CI jobs themselves** — every change was verified locally against
the exact CI command before commit, and confirmed green on the PR and on `main`.
For a "make CI green" phase, CI *is* the acceptance oracle.

## Technical debt introduced

1. **`TODO(security)` in `forge-mcp/src/lib.rs:83`** — `#[allow(deprecated)]`
   on tower-http 0.6 `ValidateRequestHeaderLayer::bearer`. Reviewed
   (rust-reviewer): no *new* regression, but the bearer comparison is
   non-constant-time. Real fix = a custom `ValidateRequest` impl with
   `subtle::ConstantTimeEq`. Carried in the TODO, not silently buried.
2. **`#[allow(dead_code)]` on `McpRequest.jsonrpc`** — deliberate (documents the
   JSON-RPC wire contract); low concern.
3. **`_task_description` in forge-reflect** — a computed-but-unused value
   (`IterationRecord` has no field for it yet); `_`-prefixed with an inline
   note, not deleted.

## Process deltas (things that cost time — corrective actions)

- **fmt-after-clippy round-trip:** I ran `cargo fmt` (002) *before* the clippy
  edits (003); clippy reflows lines, so CI's fmt step failed on one file after
  I pushed PR-A. Cost one CI cycle + a force-push amend. **Corrective action
  (captured to memory [[feedback_fmt_after_clippy]]):** for any fmt+clippy Rust
  job, run `cargo fmt` *last*, then re-verify fmt+clippy+test in that order
  before pushing. Now a standing rule.
- **local nightly vs CI stable:** my local clippy ran on nightly; CI uses
  `stable`. `--all --all-features` locally caught the superset here, but this is
  a latent source of "green locally, red in CI." Worth a stable toolchain pin
  for local verification.

## Out of scope / carry-forward

- **`cross-model-qa.yml`** is red on `main` but **not one of the 4 README
  badges** (it badges nothing), so it was correctly deferred. It remains a real
  red workflow — recommend a follow-up phase to green it (or delete if
  obsolete).
- The security TODO (#1 above) deserves its own small change.

## Recommended next phase

**phase-ci-cross-model-qa-and-hardening** — (a) green or retire
`cross-model-qa.yml`; (b) replace the deprecated forge-mcp bearer auth with a
constant-time custom validator (closes the TODO + the pre-existing timing-attack
surface); (c) pin a local stable Rust toolchain so local clippy/fmt matches CI.
Start with `/kbd-assess phase-ci-cross-model-qa-and-hardening`.

## Sycophancy self-check

This reflection names: a delta I nearly mis-reported (transient CI snapshot), a
process failure I caused (fmt/clippy ordering, one wasted CI cycle), 3 debt
items I introduced, and a still-red workflow I did not fix. It does not claim
the codebase is now clean — only that the badged workflow is verified green.
