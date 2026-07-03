# Plan — phase-ci-cross-model-qa-and-hardening

_Backend: **OpenSpec**. 3 goals. Keep `validate.yml` GREEN throughout._

## What Plan resolved (open questions from Assess)

- **OQ-A1 (real cross-model-qa error): RESOLVED.** Installed `actionlint`; it
  reports the true GitHub error:
  `cross-model-qa.yml:130: could not parse as YAML: could not find expected ':'`.
  **Confirmed root cause:** the `run: |` block scalar (step "Post PR comment")
  starts content at 10-space indent (lines 127-128), but the multi-line bash
  `COMMENT="…"` string continues at **column 0** (lines 130-142). A block-scalar
  line with less indentation than the first content line **terminates the
  scalar**, so YAML reads `**Model:**` as a new mapping key → parse failure →
  GitHub records a **startup_failure** on every push (event=push, 0 jobs). The
  PyYAML line-130 guess was, this time, correct.
- **OQ-A2 (fix vs retire): DECISION = FIX.** It's a genuinely useful
  anti-sycophancy secondary-model review tool, aligned with this repo's ethos,
  and the fix is a small YAML indentation change. Retiring would discard value.
- **OQ-A3 (ANTHROPIC_API_KEY secret): appears UNSET** (`gh secret list` empty).
  **Scope consequence:** the YAML fix makes the workflow *load* cleanly and stops
  the red startup_failure — that fully satisfies the "not red" goal. A
  *successful dispatch* additionally needs the secret, which only the repo owner
  can provision. Plan targets "loads cleanly + red status gone"; the secret is a
  documented operational prerequisite, not a code deliverable.
- **GAP-C pre-check: RESOLVED.** No `#![feature(...)]` anywhere in forge-rs →
  pinning `stable` is safe.
- **GAP-B pre-check:** token comes from `FORGE_MCP_TOKEN`; tower-http 0.6 has the
  `auth`/`validate_request` module for a custom `ValidateRequest`. Need to add
  the `subtle` crate.

## Ordered change list (3 changes)

Order = lowest risk first; GAP-C first so GAP-B's local verification runs on the
same stable toolchain CI uses.

### change-hard-001 — pin local stable Rust toolchain  _(GAP-C)_
- **What:** add `tools/forge-rs/rust-toolchain.toml`:
  `[toolchain]\nchannel = "stable"\ncomponents = ["rustfmt", "clippy"]`.
- **Why:** local default is nightly; CI is `@stable`. This mismatch caused the
  fmt/clippy CI round-trip last phase. Pinning makes local `cargo fmt`/`clippy`
  in the forge-rs tree select stable automatically.
- **Verify:** `cd tools/forge-rs && rustup show active-toolchain` reports stable
  (once installed); `cargo fmt --check --all` + `cargo clippy --all
  --all-features -- -D warnings` + `cargo test --all` all pass on stable.
- **Risk:** LOW (no `#![feature]` present). One-time `rustup toolchain install
  stable` may be needed locally.
- **Agent:** direct edit.

### change-hard-002 — fix cross-model-qa.yml block-scalar parse error  _(GAP-A)_
- **What:** in the "Post PR comment" step, restructure the multi-line
  `COMMENT="…"` so no line escapes the `run: |` block scalar. Preferred: build
  the comment via a `cat <<'EOF' > /tmp/comment.md … EOF` heredoc (all lines
  properly indented under the block scalar) or a `printf`, then
  `gh pr comment "$PR_NUMBER" --body-file /tmp/comment.md`. Keep the rendered
  markdown identical (no leading whitespace in the posted comment).
- **Verify:** `actionlint .github/workflows/cross-model-qa.yml` exits 0 with no
  findings; `python3 -c yaml.safe_load` parses; push shows no startup_failure
  run for this workflow.
- **Note (OQ-A3):** add a comment in the workflow (or CONTRIBUTING) that a real
  dispatch needs the `ANTHROPIC_API_KEY` repo secret. Do NOT add a fake key.
- **Risk:** LOW (config only; workflow is unbadged and gates nothing). The only
  way to fully confirm is that the next push produces no failed startup run.
- **Agent:** direct edit + `actionlint`.

### change-hard-003 — constant-time forge-mcp bearer auth  _(GAP-B)_
- **What:** in `forge-mcp/src/lib.rs`, replace
  `#[allow(deprecated)] ValidateRequestHeaderLayer::bearer(&token)` with a custom
  type implementing `tower_http::validate_request::ValidateRequest` that:
  reads the `Authorization` header, requires `Bearer <token>`, and compares with
  `subtle::ConstantTimeEq` (constant-time). Remove `#[allow(deprecated)]` and
  the TODO. Add `subtle` to `tools/forge-rs/Cargo.toml` (workspace) + forge-mcp.
- **Constraints (CI must stay green):** clippy `-D warnings` clean; add unit
  tests (accept valid token / reject wrong token / reject missing header /
  reject malformed non-Bearer). Preserve the existing behavior of returning 401
  on failure.
- **Verify:** `cargo fmt/clippy/test` (stable) all 0; the `forge-mcp` :8943
  launchd service still starts and authenticates (curl with/without the token);
  `validate.yml` jobs (`forge-rs-test`, `Check Rust CLI`, BDD, sycophancy) stay
  green on the PR.
- **Risk:** **MEDIUM-HIGH** — real auth-path change on a live service. Route
  through `rust-reviewer` (security lens) before finalizing; adversarially check
  the header parsing (no panic on malformed/absent header; case-insensitive
  scheme per RFC 7235).
- **Agent:** `security-reviewer` + `rust-reviewer`; `rust-build-resolver` if the
  ValidateRequest trait bounds fight the borrow checker.

## Sequencing & PR strategy

- **001 + 002** are trivial/independent (toolchain file + workflow YAML) → one
  small PR (PR-A).
- **003** is the substantive security change → its own PR (PR-B) with the
  security review, so it can be scrutinized in isolation.
- Each change verified locally (on stable) before its PR; each PR must show
  `validate.yml` green before merge.

## Open questions carried into Execute

1. **OQ-A3 secret** — confirm with the owner whether `ANTHROPIC_API_KEY` should
   be provisioned so cross-model-qa can actually run (beyond just loading). Out
   of code scope.
2. **change-003** — does any other caller rely on the *exact* current 401
   behavior/headers of the bearer layer? Verify forge-mcp clients (Claude
   Desktop / MCP config) still authenticate.

## First change to apply

**change-hard-001** (toolchain pin) — trivial, and it makes 003's local
verification match CI. Then 002 → 003.
