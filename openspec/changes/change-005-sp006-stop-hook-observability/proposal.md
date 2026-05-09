## Why

The Stop hook chain runs four scripts with `|| true` everywhere. Failures are silently swallowed — no log, no record, no diagnostics. `~/.prometheus/hooks.log` does not exist. No hook script writes to any log file (confirmed by assessment: zero grep matches for `hooks.log` across all `.sh` files).

This is P0 because it gates multiple downstream tasks:
- SP-013 (sycophancy gate) silently fails without logging — no way to know the gate ran.
- SP-014 (fallback matcher verification) requires a log to validate behavior.
- XC-004 (`prometheus doctor`) needs hook-run records to validate end-to-end health.

Without observability, debugging hook misbehavior requires running each script manually. The current state has zero diagnostics.

## What Changes

- Write `shared/scripts/lib/hook-log.sh` providing three functions:
  - `hook_log_start <hook-type> <script-name>` — writes JSONL start event
  - `hook_log_end <exit-code> <duration-ms>` — writes JSONL end event
  - `hook_log_error <lineno>` — writes JSONL error event with stderr snippet
  - Uses `flock` to serialize concurrent writes.
  - Writes to `~/.prometheus/hooks.log` in JSONL format.
- Source the shim from every Stop-chain script. Replace `|| true` with `|| hook_log_error "$LINENO"`.
- Add `logrotate` config at `shared/config/logrotate.d/prometheus-hooks` keeping 30 days of gzipped daily logs.
- Extend the shim to also cover `UserPromptSubmit` and `PreToolUse` hook scripts (same format, same log file).

JSONL format:
```json
{"ts":"2026-05-09T10:23:11Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"event":"start","session_id":"..."}
{"ts":"2026-05-09T10:23:14Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"event":"end","exit_code":0,"duration_ms":2987}
```

## Capabilities

### New Capabilities
- `hook-observability-log`: JSONL log at `~/.prometheus/hooks.log` recording every hook start, end, and error event with timing and session context.
- `hook-log-rotation`: Daily logrotate config keeping 30 days of hook logs.

### Modified Capabilities
- `stop-hook-chain`: All Stop-chain scripts now emit log events rather than silently swallowing failures.

## Impact

- `shared/scripts/lib/hook-log.sh` — new library (sourced by all hook scripts)
- `shared/scripts/forge-reflect-on-stop.sh` — add shim sourcing + wrap body
- `shared/scripts/pk-focus-on-prompt.sh` — add shim sourcing
- `shared/scripts/guard-direct-deploy.sh` — add shim sourcing
- `shared/scripts/validate-gitops-write.sh` — add shim sourcing
- `shared/scripts/subagent-checkpoint-fallback.sh` — add shim sourcing
- `shared/config/logrotate.d/prometheus-hooks` — new logrotate config
- `~/.prometheus/hooks.log` — created at runtime (not committed)
