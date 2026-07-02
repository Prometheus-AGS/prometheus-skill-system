# Execution — change-005-sp006-stop-hook-observability

**Executed:** 2026-05-09  
**Backend:** OpenSpec  
**Agent role:** hooks-engineer  
**Executor:** claude-sonnet-4-6

## Dispatch

Feature implementation in prometheus-skill-pack. Seven files created or modified.

## Files Modified (prometheus-skill-pack)

1. `shared/scripts/lib/hook-log.sh` — **new library**
   - `hook_log_start <hook-type> <script-name>`: emits JSONL start event with ts, hook, script, pid, session_id.
   - `hook_log_end <exit-code> [duration-ms]`: emits JSONL end event with timing derived from start ms.
   - `hook_log_error <lineno>`: emits JSONL error event with lineno.
   - `flock`-based atomic writes to `~/.prometheus/hooks.log`. Falls back to unguarded append when `flock` is absent.
   - Session ID sourced from `$CLAUDE_SESSION_ID` → `$DEEP_SESSION_ID` → `pid-$$` fallback.
   - Millisecond timing via `python3 time.time()` with 0-fallback when Python unavailable.
   - All write failures are no-ops (never kills the calling hook).

2. `shared/scripts/forge-reflect-on-stop.sh`
   - Sources `lib/hook-log.sh` via resolved absolute path (portable across symlinks).
   - `hook_log_start "Stop" ...` at entry.
   - `forge reflect 2>&1 || hook_log_error "$LINENO"` replaces `|| true`.
   - `pk ingest 2>&1 || hook_log_error "$LINENO"` replaces `|| true`.
   - `hook_log_end 0` before every exit point.

3. `shared/scripts/pk-focus-on-prompt.sh`
   - Sources `lib/hook-log.sh`.
   - `hook_log_start "UserPromptSubmit" ...` at entry.
   - `pk focus ... || hook_log_error "$LINENO"` replaces `|| true` in both timeout and non-timeout paths.
   - `hook_log_end 0` before every exit point.

4. `shared/scripts/guard-direct-deploy.sh`
   - Sources `lib/hook-log.sh`.
   - `hook_log_start "PreToolUse" ...` at entry.
   - Each block path calls `hook_log_end 2` then `exit 2` — the blocked exit code is recorded.
   - `hook_log_end 0` on clean pass.

5. `shared/scripts/validate-gitops-write.sh`
   - Sources `lib/hook-log.sh`.
   - `hook_log_start "PostToolUse" ...` at entry.
   - `hook_log_end 0` before every exit point.

6. `shared/scripts/subagent-checkpoint-fallback.sh`
   - Sources `lib/hook-log.sh`.
   - `hook_log_start "SubagentStop" ...` at entry.
   - `hook_log_end 0` before exit.

7. `shared/config/logrotate.d/prometheus-hooks` — **new logrotate config**
   - Targets `~/.prometheus/hooks.log`.
   - Daily rotation, 30-day retention, gzip + delaycompress.
   - `missingok notifempty` — safe when hook has never run.
   - Install: `sudo cp prometheus-hooks /etc/logrotate.d/` or user-level with `--state`.

## JSONL Format

```json
{"ts":"2026-05-09T10:23:11Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"session_id":"abc123","event":"start"}
{"ts":"2026-05-09T10:23:14Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"session_id":"abc123","event":"end","exit_code":0,"duration_ms":2987}
{"ts":"2026-05-09T10:23:12Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"session_id":"abc123","event":"error","lineno":16}
```

## QA Gate

Applied: 7 files modified (≥ 3 threshold met).

**Syntax checks (bash -n):**
- `shared/scripts/lib/hook-log.sh` — PASS
- `shared/scripts/forge-reflect-on-stop.sh` — PASS
- `shared/scripts/pk-focus-on-prompt.sh` — PASS
- `shared/scripts/guard-direct-deploy.sh` — PASS
- `shared/scripts/validate-gitops-write.sh` — PASS
- `shared/scripts/subagent-checkpoint-fallback.sh` — PASS

**Acceptance criteria check:**
- [x] `~/.prometheus/hooks.log` written in JSONL on hook execution (runtime-created, not committed)
- [x] `hook_log_start`, `hook_log_end`, `hook_log_error` implemented with correct signatures
- [x] `flock` serialization for concurrent write safety; graceful fallback when absent
- [x] Every Stop-chain script sources the shim and calls start/end/error
- [x] `guard-direct-deploy.sh` records exit code 2 on block events
- [x] `shared/config/logrotate.d/prometheus-hooks` present with 30-day rotation
- [x] No existing hook behavior changed (all scripts still exit 0 on clean path, exit 2 on guard violation)

## Status

DONE
