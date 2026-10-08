### 2026-10-04T02:46:03Z — analyze decisions (team-aware-learning-memory-impl)
- D-1 Codex delivery: native SubagentStart (spike CONFIRMED: agent_type, agent_id, additionalContext reached subagent). Fallback not built. Provenance: research (spike).
- D-2 agent-team memory.mts: adapt in place to learning envelope + agent_id table + resolved user_id; new PR B3b. Provenance: research.
- D-3 Mini pin: A5 -> A5a (agent) + A5b (operator); D1 -> D1a + D1b. Provenance: repo policy (versions.toml operator-authored).
- D-4 Matchers namespaced; iterative-evolver agents not renamed. Provenance: research.
- D-5 pk run_context: score all, cap after scoring, redistribute failed-scope budget.
- D-6 surreal-memory: lean search default, categories honoured in storage + REST, loopback-only rekey op.
- D-7 Mini D2: seconds + budget test rewrite; no Codex hooks in mini this phase.
### 2026-10-04T02:51:50Z — analyze round-1 revisions
- D-1 evidence strengthened (child-thread developer message; plugin path + anchored matcher PASS). PreToolUse fallback = specced contingency, unbuilt.
- D-4 revised: five per-role anchored prefix-required matchers (claude/kimi) + Codex-only bare groups guarded by team-role check; no combined regex.
- D-2 revised: caller passes project id; memory.mts fails closed.
- D-6 revised: v2 operation rekey_agent_id, loopback-only, NONE-or-NULL predicate; lean DTO at response boundary; categories in both hybrid legs.
- D-3 revised: G1 mini half BLOCKED-ON-OPERATOR.
- Critical path: max(A1→{A2,A3},A4)→A5a→B3→B4→B5; B1→B2 parallel from start.
