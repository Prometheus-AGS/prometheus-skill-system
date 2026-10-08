# Spec review resolution — team-aware-learning-memory

Two harness-native rounds were run (same model family; the REST judge reports JUDGE_MODEL_COLLISION).
- Round 1: BLOCK 1/8/2. All findings revised.
- Round 2: BLOCK 1/5/5. All findings revised after the round, as listed below.

## Unresolved review findings
The round cap (2) was reached. These round-2 fixes were applied **without a third review**. Execute must treat them as reviewed-once:
- tlm-001 CRITICAL: Codex isolation now uses `--ephemeral --ignore-user-config`, not `-c features.memories=false`, so behaviour 5 stays observable. If `--ignore-user-config` drops authentication or project hooks, the run is UNVERIFIABLE.
- tlm-001: `~/.claude.json` is added to the hashes. Behaviour 5 is probed without writing memory (grounded self-report). Verification is keyed per (behaviour, harness), requires a fallback on every non-CONFIRMED row, and re-hashes live files against the pre-probe hashes. The plugin-agent form uses `claude --plugin-dir`.
- tlm-004: the 10 s floor is limited to the one-time conversion (standing rule 1–600 s, consistent with tlm-002's ≤5 s delivery hooks). Idempotency is content-hashed. The millisecond-wording greps are widened.
- tlm-002: every schema property must appear in the mapping table, and the invalid example must fail on `visibility`.
- tlm-003: parent-plan backup and prefix check; `$HOME` instead of a hard-coded path; matcher, mini and Cortex PRs located as dedicated PRs.
