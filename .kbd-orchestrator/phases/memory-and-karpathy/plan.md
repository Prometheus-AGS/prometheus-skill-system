# Plan — memory-and-karpathy

Backend: native-kbd. Ordered: the bridge lib first (everything depends on it),
then write-back wiring, then pk/CI/karpathy housekeeping. Each change carries
`scope:`, enforced by the warn-mode guard from Phase 3.

| # | Change | Gaps | Summary |
|---|--------|------|---------|
| 1 | change-001-memory-bridge | M1, M2 | `shared/scripts/lib/memory-bridge.sh` — HTTP JSON-RPC wrapper (pattern from mem0-compress.sh): `mem_available`, `mem_add_memory`, `mem_create_task_stream`, `mem_add_task_step`, `mem_complete_step`. Every failure appends to `.kbd-orchestrator/memory-outbox.jsonl` and returns 0. `[GLOBAL]` prefix → user_id="global" else project. `.gitignore` the outbox. Tests with PATH-shimmed fake curl. |
| 2 | change-002-memory-writeback | M1, M2 | Orchestrator builtin hook entries (KBD/hooks/hooks.json): `execute:before` → create task stream + step per change; `reflect:end` → mem_add_memory of Delta/Corrective-Actions. New `shared/scripts/memory-writeback.sh` (Claude Code PostToolUse on reflection.md, `|| true`) only fires when reflect_gate passed. New `shared/scripts/memory-outbox-flush.sh` (SessionStart) drains outbox. Wire both. Tests. |
| 3 | change-003-pk-and-binary | M3, M4, M5 | `shared/scripts/pk-health.sh` (SessionStart, 24h-throttled `pk lint --check`); pk ingest at orchestrator `reflect:end`; register pk-lint/mem0-compress via npm scripts; CI job builds sycophancy-correction binary + runs 1 real e2e gate test; karpathy-tokenizer SKILL.md gets explicit "reference-only, intentionally not hook-wired" note. Wire SessionStart hook. Tests. |

Completion per change: change.md tasks checked, tests green, commit. Phase end:
`npm run validate:strict`, `npm run build`, `validate:signals`, full shell-test
sweep including new memory + pk tests. Reflection gated. If a change edits the
orchestrator SKILL.md, also extract a section to clear the 500-line warning
(CF CA-6) — change-002/003 touch hooks.json, not SKILL.md, so this likely
remains a standing carry-forward.
