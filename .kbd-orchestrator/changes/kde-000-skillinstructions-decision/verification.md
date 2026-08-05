# Verification — kde-000

1. `assessment.md` has an E0 section with an explicit ADOPTED verdict.
2. No change to `scripts/install-kimi-desktop-plugin.sh` — this change is a
   decision record, and a code diff here would mean the scope was misread.
3. The decision is discoverable where a reviewer looks: an E0 section in
   `assessment.md` naming this change as its owner, and this change's own
   `spec.md`.

A future review's verdict is deliberately NOT a gate. This change controls what
it writes, not what a later reviewer says about the whole change set — and a
gate that cannot be evaluated without running an external, non-deterministic
process is not a gate. AC3 was corrected at plan time; this file and t2 had kept
the old wording, which is the inconsistency the review then flagged.

## Note

This is the cheapest possible change and it exists only because a decision with
no owner kept being deferred. The lesson is procedural: a warning carried across
three handoffs is not "tracked", it is unowned.
