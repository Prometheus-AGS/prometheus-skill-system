# Handoff: plan → execute (phase-ci-all-green)

5 changes, ordered safest→riskiest. First to apply: change-green-001.

CRITICAL finding from Plan (reproduced locally): the BDD loader fix (tsx) makes the
suite RUN but exposes real failures — 5 real forge-behavior failures + 6 undefined
(all in tests/features/drafts/, which CI wrongly sweeps in). So GAP-1 splits:
- 004 = tsx loader + exclude drafts/ from the CI glob (config only, NOT steps).
- 005 = fix the forge binary so `forge validate` exits non-zero on error (currently
  exits 0) to satisfy the IMMUTABLE steps. Do NOT edit tests/steps or non-draft
  features. HIGH risk: forge-rs is consumed by forge-mcp + Check Rust CLI — reverify.

001 (prettier) + 002 (forge fmt) are cosmetic/low-risk → land first.
003 = forge-rs clippy+test (second-order, must actually run, not assume).

PR strategy: PR-A {001,002,003}, PR-B {004,005}. Badges flip green only when the
WHOLE validate.yml passes on main → the final merge is the trigger.

Read plan.md for full per-change detail + verify commands.
