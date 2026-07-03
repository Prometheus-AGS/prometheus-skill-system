# Reflection — phase-ci-cross-model-qa-and-hardening

_Generated: 2026-07-03 · Reflect stage. Verified against `main` after PRs
#26 + #27 merged — not assumed._

## Goal achievement

| Goal | Status | Verified how |
|---|---|---|
| Green/retire `cross-model-qa.yml` (startup_failure) | **MET** | actionlint on main = 0; **no** cross-model-qa run exists for the post-fix shas (8625ba8, 3c624c3) — it no longer fires a failed startup run on push. |
| Constant-time forge-mcp bearer auth | **MET** | `subtle::ConstantTimeEq` present, `#[allow(deprecated)]` removed on main; e2e on the live :8943 service (no-token 401, wrong 401, correct 200). |
| Pin local stable toolchain | **MET** | `tools/forge-rs/rust-toolchain.toml` on main; pin resolves to stable; all gates pass on rustc 1.96.0. |

3/3 goals MET. `validate.yml` on `main` remains **all-green (success)** — the
regression guard held through all three changes.

## Delivered changes (3/3) & PRs

| Change | Gap | PR |
|---|---|---|
| 001 pin stable toolchain | C | #26 (merged) |
| 002 fix cross-model-qa block-scalar | A | #26 (merged) |
| 003 constant-time bearer auth | B | #27 (merged) |

## Root-cause quality (found by tooling + review, not inference)

- **GAP-A:** I did **not** trust my local PyYAML "line 130" guess — installed
  `actionlint` and confirmed GitHub's real parser error (a de-indented bash
  string terminating the `run: |` block scalar). Fixing it also revealed the
  subtler truth: once the file parses, `on: workflow_dispatch`-only means push
  **correctly stops triggering it** — so "no runs on push" IS the success state,
  not a missing run. I verified that interpretation rather than assuming red→gone.
- **GAP-B:** routed the auth change through the **security-reviewer**, which
  caught a real **MEDIUM auth-bypass** (empty/whitespace `FORGE_MCP_TOKEN` was
  accepted → `Bearer ` would authenticate) plus a HIGH documentation defect (my
  constant-time comment was factually wrong about `subtle`'s length early-out)
  and a constitution `expect()` violation. All fixed with a regression test.
  Shipping my first draft would have replaced one weakness with another.
- **GAP-B tower-http API:** read the `ValidateRequest` trait from the vendored
  crate source (docfork was down) to get `type ResponseBody` / signature exact.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with artifact-refiner QA | 0/3 |
| First-pass pass rate | n/a |

No artifact-refiner logs — these were CI-config + a Rust security change, driven
directly. QA was: `actionlint` (GAP-A), the security-reviewer + 6 unit tests +
live-service e2e (GAP-B), and the pinned-stable CI gate sequence (GAP-C). Every
change verified locally on the same stable toolchain CI uses, then confirmed
green on the PR.

## Technical debt introduced / remaining

1. **NEW debt: none of substance.** The bearer change *removed* debt (the
   deprecated non-constant-time layer + its TODO). No new `#[allow]` remains on
   that path.
2. **Carry-forward (pre-existing, out of scope):**
   - **OQ-A3 — `ANTHROPIC_API_KEY` is unset.** cross-model-qa now *loads* and no
     longer shows red, but a real `workflow_dispatch` review run will fail at the
     API step until the owner provisions the secret. This is operational, not
     code. **The goal was "not red / loads clean"; a successful dispatch is a
     separate, owner-gated step.** Do not claim the workflow is "working end to
     end" — it is fixed and ready, pending the secret.
   - Optional hardening the reviewer noted but I deliberately did NOT do (fails
     closed, low value here): case-insensitive `Bearer` scheme; `WWW-Authenticate`
     header on 401; hashing the token to fixed width to hide even its length.

## Process deltas (cost time — corrective actions)

- **`${{ }}` in commit messages** breaks a bash heredoc (`bad substitution`) —
  hit it twice (change-002, -003). **Corrective:** commit messages / workflow
  edits containing `${{ }}` must use `git commit -F <file>`, never inline `-m`
  with a heredoc. (New standing habit.)
- **CI trigger scope:** `validate.yml` runs only on push/PR **to main**, so
  PR-B stacked on the PR-A branch got "no checks reported." **Corrective:**
  target PRs at `main` (or rebase onto merged main) to get CI coverage; verified
  by retargeting + rebasing PR #27, after which all 9 jobs ran and passed.
- **fmt-after-clippy** (carried from last phase) held: I ran `cargo fmt` last
  after the auth edits — no CI fmt surprise this time.

## Recommended next phase

Low urgency — the badged CI surface is green and hardened. Candidates:
1. **Provision `ANTHROPIC_API_KEY`** (owner action) + a smoke `workflow_dispatch`
   of cross-model-qa to confirm it runs end-to-end. Small, but closes OQ-A3.
2. **Broaden the stable-toolchain pin** to the other vendored Rust trees
   (`tools/prometheus-cli`, `tools/surreal-memory-server` build) so the whole
   repo has CI/local parity, not just forge-rs.
3. Otherwise: no pressing CI/security debt remains; return to product work.

## Sycophancy self-check

This reflection names: a goal whose "success = absence of a run" I had to verify
(not assume); a real auth-bypass my own first draft introduced and the review
caught; two repeated process frictions I caused (`${{ }}` commits, CI trigger
scope); and an explicit refusal to overclaim GAP-A (loads clean ≠ runs
end-to-end without the secret). It does not present the phase as flawless.
