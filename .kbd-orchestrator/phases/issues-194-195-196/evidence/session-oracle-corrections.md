# Session oracle corrections (attempt 11 -> 12)

Attempt 11 results are kept unchanged (evidence/attempt-11). Four assertions were corrected after seeing them, and that is disclosed here:
- POL-001 / SCN-001 / SCN-002 asserted "attempted no shell command". The contract wording is "no generated-code check while production is incomplete". The sessions' shell use was read-only exploration (find/cat/ls/grep). The oracle now forbids test/build/lint/dry-run/install/code-running commands (`earlyChecks`) and still forbids Write/Edit where the scenario does.
- POL-002 matched the literal phrase "narrow...compiler". The session said "at most one compiler check, aimed narrowly" and flagged the VLBV conflict from the method-context index. The oracle now accepts either word order and also requires a statement that the exception grants no test authority.
- SCN-002 hit ENOTEMPTY while cleaning a scratch world a session had written to; cleanup now retries.
No assertion was removed and none was loosened to accept an absent behavior.

# Attempt 12 -> 13: fixture and harness defects (not product verdicts)
- SCN-002 positive: the fixture said `implementationComplete: true` while the canonical task was still pending, had no real source, and the contract's scope disagreed with its matrix. The session correctly refused to author against a self-contradicting record (a genuine observation of the skill). The fixture now completes the canonical task/change, adds a real `src/pref.js`, and uses one consistent CS-1.
- "files written" excluded the plugin's own background-hook output (`.prometheus/`, `.kbd-orchestrator/`); the unknown-ID refusal regex now accepts "doesn't exist".
- Scratch cleanup raced a detached plugin learning worker still writing into a session project (ENOTEMPTY). Cleanup now retries and records a leak instead of throwing; this masked three results in attempt 12.

# Attempt 15 -> 16
- POL-003 budget oracle matched the stale phrases the session correctly quoted ("none, start fresh ... must not reset"); it now requires the retained figure (50) plus retained/consumed wording.
- Codex: the scratch login works but the ChatGPT account returned "You've hit your usage limit. Try again at Nov 2nd, 2026". That is an environment block; Codex assertions are skipped and the limitation is recorded. Codex remains NOT exercised.
