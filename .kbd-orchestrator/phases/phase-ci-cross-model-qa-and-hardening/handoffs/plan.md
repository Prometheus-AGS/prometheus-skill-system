# Handoff: plan → execute (phase-ci-cross-model-qa-and-hardening)

3 changes, safest first. First: change-hard-001.

OQ resolved by Plan: cross-model-qa error is REAL (actionlint: block-scalar parse
err at line 130 — the COMMENT="..." multiline string escapes the run: | scalar).
Decision = FIX (not retire). ANTHROPIC_API_KEY appears unset → YAML fix stops the
red startup_failure; a real dispatch still needs the secret (owner-provisioned,
out of code scope).

- 001 GAP-C: add tools/forge-rs/rust-toolchain.toml (stable + rustfmt,clippy).
  No nightly features present → safe.
- 002 GAP-A: rebuild the "Post PR comment" step's COMMENT via a 'cat <<EOF' heredoc
  + --body-file so nothing escapes the block scalar. Verify with actionlint (0).
- 003 GAP-B: custom ValidateRequest w/ subtle::ConstantTimeEq; add subtle dep;
  drop #[allow(deprecated)]; unit-test accept/reject/missing/malformed; keep
  clippy -D warnings clean; reverify :8943 service + validate.yml green. Route
  through security-reviewer. MEDIUM-HIGH risk.

PR strategy: PR-A {001,002}, PR-B {003}. Keep validate.yml green. Read plan.md.
