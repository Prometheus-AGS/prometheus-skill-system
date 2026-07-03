# Handoff: assess → plan (phase-ci-cross-model-qa-and-hardening)

3 carry-forward goals from phase-ci-all-green:
- GAP-A cross-model-qa.yml: startup_failure on push (event=push, 0 jobs, conclusion=failure)
  despite on: workflow_dispatch-only. Unbadged, gates nothing → red is cosmetic.
  OPEN: OQ-A1 read GitHub's real parse error (actionlint, not the PyYAML line-130 guess);
  OQ-A2 fix vs retire; OQ-A3 is ANTHROPIC_API_KEY set (else review step still fails).
- GAP-B forge-mcp lib.rs:83: deprecated non-constant-time bearer auth. Add `subtle`,
  write custom ValidateRequest w/ ConstantTimeEq, drop #[allow(deprecated)], unit-test
  accept/reject/missing, keep clippy -D warnings clean, reverify :8943 service.
- GAP-C: add tools/forge-rs/rust-toolchain.toml (stable + rustfmt,clippy). Check no
  nightly-only features first.

Sequence: GAP-C (trivial, aligns B's local verify w/ CI) → GAP-A (after OQ-A1/2/3)
→ GAP-B (highest risk). Keep validate.yml green throughout. Read assessment.md.
