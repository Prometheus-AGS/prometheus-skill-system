### 2026-10-04T00:05Z — Analyze decisions (team-aware-learning-memory)
- D-1 No external memory library adopted; patterns copied from mem0, CrewAI, LangMem, Graphiti, Letta (cand-001..005). Provenance: research L1.
- D-2 Per-agent delivery via SubagentStart in Claude Code and Codex (cand-006/007); main thread context becomes lead/operator-scoped.
- D-2a File-memory tier (Claude auto-memory ~14 KB into every subagent) must be reduced or partitioned, not stacked on; spec chooses between per-role index sections and per-subagent `memory:` dirs.
- D-3 Visibility levels agent|role:<r>|team|project|user|global, default agent; promotion beyond team = auto-propose, human confirms (approved policy).
- D-4 Budgets: Claude SubagentStart <=8,000 chars; Codex <=2,000 tokens; main-thread prompt context ~2 KB targeted.
- D-5 Fix PR #121 timeout-unit misdiagnosis and namespace SubagentStop matchers within this phase (C-01/C-03).
