# Reflection — safeguards

Gate: sycophancy-correction analyze_reflect_phase — score 0.018 (PASS, threshold 0.4); S-08 not detected; one Low S-07 (length) note only.

## Delta

1. scope-guard/scope-record shipped with two test-time bugs: (a) path relativization used raw string-prefix against the passed-in root, failing on macOS /var vs /private/var symlinks so out-of-scope paths never matched globs; (b) the in-scope check used a `<<'PY'` heredoc inside `$(...)`, capturing python source as literal text instead of executing it. Both caught by the fixture test.
2. The artifact-gate test failed twice initially: the fake MCP binary built JSON-RPC with `printf '%s'` (mangled nested-JSON escaping), and the pipeline-enforce fixture lacked current-waypoint.json (pipeline-enforce reads phase from the waypoint and skips silently without it). Both were test-harness bugs masking correct production behavior.
3. The artifact gate cannot un-write a file — enforcement is a progress.json flag + a pipeline-enforce block at the next lifecycle boundary. A sycophantic reflection lands on disk and is only caught at phase advance. Strongest achievable PostToolUse guarantee, weaker than a true pre-write block.
4. The real sycophancy-correction binary is not built here, so every test uses a PATH-shimmed fake. The JSON-RPC handshake, FIFO timing, and response shape are assumed to match the real server, not verified against it.
5. All three safeguards are hooks that won't take effect until reload (session-snapshot). The scope guard ships in warn mode — even once live it only logs.
6. Orchestrator SKILL.md is still 620 lines (>500 warn) — the Phase 2 carry-forward to extract a section was not addressed (no change this phase touched that file).

## Root Cause

1. macOS symlink and bash heredoc-in-subshell semantics are non-obvious; authoring simulated on canonical paths and a wrong mental model of heredoc behavior in command substitution. The Phase 1/2 fixture-test-every-branch rule is what surfaced them.
2. The fake binary was written the fast way (printf) not the correct way (python json.dumps); the missing waypoint was an incomplete fixture. Test fixtures got less rigor than production code.
3. PostToolUse cannot block a write by design; the flag+boundary-gate pattern is the accepted workaround — a true pre-write block would require knowing an artifact is sycophantic before it exists.
4. Building the Rust binary was out of scope for a hooks/scripts phase; the gate degrades gracefully when the binary is absent (verified), so the untested real path fails safe (skips).
5. Session-snapshot is a Claude Code property; warn mode is a deliberate user decision to observe before enforcing.
6. No change this phase had reason to edit orchestrator SKILL.md, so the deferred extraction did not come up.

## Corrective Actions

1. Make path canonicalization (`cd && pwd -P`) standard preamble for any hook comparing paths to roots — applied to scope-guard/scope-record; carry into the Phase 5 child-scope hook. Ban heredoc-in-command-substitution for python; use `python3 -c` with env-var inputs (applied to both scope scripts).
2. Test fixtures driving a hook must replicate the full state it reads (waypoint + progress + artifact); any fake binary must emit protocol output via a real serializer. Add to test conventions.
3. If a true pre-write artifact guarantee is needed, move it to a PreToolUse hook inspecting tool_input.content — possible future hardening, noted not scheduled.
4. Phase 4 (or a dedicated step) should build the sycophancy-correction binary in CI and run one real end-to-end gate test to verify the protocol assumptions once.
5. First action next session: confirm all accumulated hooks (position inject, Stop gate, protect-tests, scope-guard, artifact gate) are live after reload; record results.
6. The next phase editing orchestrator SKILL.md must extract a section to references/ to clear the 500-line warning — now a two-phase-old carry-forward; schedule a dedicated cleanup change if no Phase 4 change touches it.

## Recommended Next Phase

memory-and-karpathy — memory-bridge.sh with the mandatory outbox fallback, automatic write-back of accepted reflections to surreal-memory (scoped global vs project), pk-health on SessionStart, pk ingest at reflect:end, explicit reference-only decision for karpathy-tokenizer, per approved plan Phase 4. Also: build the sycophancy binary in CI (CA-4) and the orchestrator SKILL.md extraction (CA-6) if a change touches that file.
