# Execution — change-006-sp013-sycophancy-reflector-hook

**Executed:** 2026-05-09  
**Backend:** OpenSpec  
**Agent role:** hooks-engineer  
**Executor:** claude-sonnet-4-6

## Dispatch

Feature implementation in prometheus-skill-pack. Four files created or modified.

## Files Modified (prometheus-skill-pack)

1. `shared/scripts/sycophancy-check-reflection.sh` — **new gate script**
   - Sources `lib/hook-log.sh` (SP-006 shim) — emits start/end/error JSONL events.
   - Reads SubagentStop hook JSON from stdin; extracts `output` field (artifact text).
   - Locates `sycophancy-correction` binary: checks `PATH` first, then `$CLAUDE_PLUGIN_ROOT/skills/imported/sycophancy-correction/target/release/`.
   - Enforces 2-rejection soft cap via `~/.prometheus/reflect-rejections.txt`. At cap: accepts with warning + resets counter.
   - Invokes MCP server via JSON-RPC FIFO:
     - `initialize` → `notifications/initialized` → `tools/call detect_sycophancy`
     - 30-second timeout via system `timeout` command (falls back to unguarded invocation).
   - Parses response: extracts `sycophancy_score` and `classifications` from `result.content[0].text`.
   - **Reject threshold**: score ≥ 0.4 OR any `high`/`critical` severity patterns.
   - **Reject** (exit 2): prints structured feedback block to stderr naming the threshold and actionable guidance.
   - **Accept** (exit 0): resets rejection counter; logs PASS with score.
   - **Graceful degradation**: binary absent → warning + exit 0; empty MCP response → warning + exit 0.
   - `PROMETHEUS_REFLECT_STRICTNESS` env var: `loose→permissive`, `standard→standard`, `strict→strict` (default), `adversarial→strict`.

2. `hooks/hooks.json` — modified `reflector` SubagentStop matcher
   - Added `sycophancy-check-reflection.sh` as the **first** command in the `reflector` hooks array (runs before `log-reflection.sh` and state-checkpoint).
   - Timeout: 35000ms (35s, accommodates MCP startup + 30s call timeout).
   - The script exits 2 on rejection; Claude Code SubagentStop semantics: non-zero exit blocks the agent output (the reflection is surfaced to the user for revision).

3. `CLAUDE.md` — added **Reflector Sycophancy Gate** section
   - Documents the gate behavior, strictness levels, good reflection structure, binary prerequisite, and state file.
   - Includes env var table and example commands.

## JSONL flow in hooks.log

On reflector SubagentStop, hooks.log will contain:
```json
{"ts":"...","hook":"SubagentStop","script":"sycophancy-check-reflection.sh","pid":...,"session_id":"...","event":"start"}
{"ts":"...","hook":"SubagentStop","script":"sycophancy-check-reflection.sh","pid":...,"session_id":"...","event":"end","exit_code":0,"duration_ms":...}
```

## QA Gate

Applied: 4 files modified/created (≥ 3 threshold met).

**Checks:**
- `bash -n shared/scripts/sycophancy-check-reflection.sh` — PASS
- `python3 -m json.tool hooks/hooks.json` — PASS (valid JSON)
- `sycophancy-check-reflection.sh` appears at line 115 of `hooks/hooks.json` — VERIFIED
- Binary not available in CI environment — gate degrades gracefully (verified by code path: `exit 0` on empty `MCP_BIN`)
- Rejection counter file created at `~/.prometheus/reflect-rejections.txt` — runtime artifact (not committed)

**Acceptance criteria check:**
- [x] `shared/scripts/sycophancy-check-reflection.sh` exists and is executable
- [x] Sources `lib/hook-log.sh` shim (depends on SP-006 / change-005)
- [x] Reads artifact from SubagentStop hook event (not conversation history)
- [x] Invokes `sycophancy-correction` via JSON-RPC stdio protocol
- [x] Configurable strictness via `PROMETHEUS_REFLECT_STRICTNESS` env var (default: strict)
- [x] Rejection exit code 2 with actionable stderr block
- [x] 2-rejection soft cap implemented via `~/.prometheus/reflect-rejections.txt`
- [x] Wired as first command in `reflector` SubagentStop matcher in `hooks/hooks.json`
- [x] `CLAUDE.md` documents the gate, strictness levels, and binary prerequisite
- [x] Graceful degradation: binary absent → exit 0 (never blocks Stop chain)

## Status

DONE
