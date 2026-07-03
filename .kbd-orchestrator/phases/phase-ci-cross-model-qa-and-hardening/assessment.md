# Assessment — phase-ci-cross-model-qa-and-hardening

_Generated: 2026-07-03 · Assessed against the 3 carry-forward goals from
`phase-ci-all-green`'s reflection._

## Goals

1. Green or retire the `cross-model-qa.yml` workflow (red on `main`, unbadged).
2. Replace the deprecated forge-mcp bearer auth with a constant-time validator
   (closes the `TODO(security)` + the pre-existing timing-attack surface).
3. Pin a local stable Rust toolchain so local `clippy`/`fmt` matches CI.

## Current state & gaps (verified from CI/API + code)

### GAP-A — `cross-model-qa.yml` fails on every push (startup_failure)
- **Evidence:** every recent run shows `event=push`, `conclusion=failure`, and
  **zero jobs** (`/actions/runs/<id>/jobs` is empty; no check-run annotation).
  That signature = GitHub rejected the workflow at load time
  (**startup_failure**), so it never ran a job.
- **Contradiction that confirms it:** the committed `on:` block is
  `workflow_dispatch:` **only** — a `push` should not trigger it at all. A
  `push`-event failed run with no jobs means GitHub tried to parse the file on
  push and the parse/config failed.
- **Local signal (not authoritative):** PyYAML fails to parse the file at
  **line 130** (`**Model:**` inside a bash heredoc string — PyYAML mis-scans the
  leading `*` as a YAML alias). GitHub's Actions parser is more lenient than
  PyYAML, so this specific line may be a false positive — **the real GitHub
  error must be read from the Actions UI or `actionlint`** (see OQ-A1).
- **Nuance:** this workflow is **`workflow_dispatch`-only and unbadged** — it
  gates nothing and isn't one of the 4 README badges. Its red status is cosmetic
  (a failed startup run in the Actions tab), not a merge blocker. **Decision
  needed (OQ-A2): fix vs retire.** It's a genuinely useful tool (independent
  secondary-model review) — leaning fix — but it also needs an
  `ANTHROPIC_API_KEY` secret that does not appear to be set (OQ-A3), so even a
  parse-fixed workflow would fail its review step on dispatch until the secret
  exists.
- **Severity:** LOW (cosmetic, unbadged) · **Effort:** LOW to fix the parse;
  MEDIUM if it also needs the secret provisioned + a real dispatch test.

### GAP-B — forge-mcp bearer auth is deprecated + non-constant-time
- **Evidence:** `tools/forge-rs/crates/forge-mcp/src/lib.rs:83` uses
  `#[allow(deprecated)] ValidateRequestHeaderLayer::bearer(&token)`. tower-http
  0.6 deprecated it; the comparison is a plain `==` (timing-attackable).
- **Fix:** implement a custom `ValidateRequest` that reads the `Authorization`
  header, strips `Bearer `, and compares with `subtle::ConstantTimeEq`. Remove
  the `#[allow(deprecated)]`. `subtle` is **not yet a dependency** — add it.
- **Constraints:** CI runs `cargo clippy --all --all-features -- -D warnings`
  and `cargo test --all` — the new validator must be clippy-clean and covered by
  a unit test (accept-valid / reject-invalid / reject-missing). The
  `MemoryStorage`-style public surface isn't touched, but forge-mcp is a live
  service (the `forge-mcp` launchd MCP on :8943) — verify it still starts and
  authenticates after the change.
- **Severity:** MEDIUM (real security hardening; low exploitability at the
  127.0.0.1 default, higher if bound to 0.0.0.0) · **Effort:** MEDIUM.

### GAP-C — no local toolchain pin; local nightly ≠ CI stable
- **Evidence:** no `rust-toolchain.toml` in `tools/forge-rs/` or repo root; CI
  uses `dtolnay/rust-toolchain@stable`; local default is
  `nightly-aarch64-apple-darwin`. This mismatch is exactly what caused the
  fmt/clippy surprise in the previous phase (green locally on nightly, then a CI
  round-trip on stable).
- **Fix:** add `tools/forge-rs/rust-toolchain.toml` with
  `[toolchain] channel = "stable"` (+ `components = ["rustfmt", "clippy"]`).
  Then local `cargo fmt`/`clippy` in that tree auto-selects stable, matching CI.
- **Risk to check:** does pinning `stable` break any nightly-only feature the
  forge-rs crates use? (grep for `#![feature(...)]` / nightly attrs — likely
  none, it built on stable in CI.)
- **Severity:** LOW (developer-experience / CI-parity) · **Effort:** LOW.

## Cross-cutting risks / open questions for Plan

- **OQ-A1:** Get GitHub's *actual* startup-failure error for cross-model-qa
  (install `actionlint`, or read the Actions run page). Do NOT fix based on the
  PyYAML guess — it may be a false positive.
- **OQ-A2:** Fix vs retire cross-model-qa. (Recommend fix — it's a useful
  anti-sycophancy secondary-review tool aligned with this repo's ethos.)
- **OQ-A3:** Is `ANTHROPIC_API_KEY` set as a repo secret? If not, a parse-fixed
  workflow still can't complete a real review; scope may be "make it *load*
  cleanly" vs "make a dispatch succeed end-to-end."
- **GAP-B blast radius:** re-verify forge-mcp service starts + authenticates,
  and that `Check Rust CLI` / `forge-rs-test` stay green after adding `subtle` +
  the validator.
- **GAP-C:** confirm no nightly-only features before pinning stable.
- **Regression guard:** all of `validate.yml` is currently green — every change
  here must keep it green (especially GAP-B touching forge-rs).

## Recommended sequencing (for Plan)

Independent; low→higher risk:
1. **GAP-C** toolchain pin (trivial, and makes B's local verification match CI).
2. **GAP-A** cross-model-qa parse fix (once OQ-A1/A2/A3 resolved).
3. **GAP-B** constant-time bearer auth (highest risk; verify service + CI green).

## Stage handoff

3 carry-forward goals. GAP-A: cross-model-qa is a startup_failure
(`workflow_dispatch`-only, unbadged, gates nothing) — real GitHub parse error
still to be read (OQ-A1), and fix-vs-retire + missing ANTHROPIC_API_KEY are open
(OQ-A2/A3). GAP-B: replace deprecated non-constant-time forge-mcp bearer auth
with a `subtle::ConstantTimeEq` custom `ValidateRequest` (add `subtle` dep;
keep clippy `-D warnings` clean; reverify the :8943 service). GAP-C: add
`tools/forge-rs/rust-toolchain.toml` pinned to stable (check no nightly-only
features first). Keep `validate.yml` green throughout.
