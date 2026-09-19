# Adversarial review disposition — spec stage

Judge k3 via rest-gateway, cross_model_check verified-distinct, producer claude-fable-5-1. Verdict PASS: 0 CRITICAL, 7 WARNING, 4 SUGGESTION. No re-vet required; every finding was applied by regenerating the change set.

| Finding | Disposition |
|---|---|
| W1 rah-005 negative fixture missing from scope | `package-unlabelled` added to Scope and task 3 files; task 3 verify runs the negative control |
| W2 rah-004 task 2 had no verify | verify greps the SSE emission call sites and the blocked mapping in job/mod.rs |
| W3 rah-003 only post-stage.sh fired | firing points defined for all four hooks; new requirement and `hooks-fired` scenario with markers in order |
| W4 rah-006 verifier-before-reviewer never specified | new requirement "Verification precedes review"; review refused without a valid stage 05 artifact; `review-without-verify` scenario |
| W5 rah-004 launchd criterion untestable | verify block now runs `launchctl kickstart -k` then polls `/health` |
| W6 rah-004 two test targets unverified | all three files confirmed present on 2026-09-04 by `ls`; task 1 records the listing; the judge's packet file tree was partial |
| W7 C-01 reconciliation unnamed | rah-011 named as reconciliation change in every spec's constraints; `check:distribution`, `check-harness-adapters.js`, `check:services-manifest` added to its certification; SKILL.md feeds only the skills index |
| S1 rah-002 jq fallback weaker than scenario | fallback compares full key set and types; result PASS WITH NOTES; scenario scoped to the jsonschema path |
| S2 rah-002 `--ingest-palace` unjustified | cited assessment research gap 9 (documented flag the script ignores, C-03) |
| S3 rah-009 hollow verifies | task 1 verifies the recorded decision in README.md; task 3 defers to the rpc_roundtrip difficulty-delta assertion |
| S4 Evidence text used FAIL as a label | Evidence instruction now separates gate outcome from the provenance enum PASS, PASS WITH NOTES, BLOCKED |
