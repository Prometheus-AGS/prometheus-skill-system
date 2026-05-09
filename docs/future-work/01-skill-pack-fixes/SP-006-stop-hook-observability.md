---
id: SP-006
title: Stop hook observability log
status: ready
priority: P0
estimated_effort: 1d
agent_role: hooks-engineer
depends_on: []
unblocks: [SP-014, SP-018, XC-004]
related: [SP-012]
created_from_conversation_turn: 3-4
---

# SP-006 — Stop hook observability log (`~/.prometheus/hooks.log`)

## Problem

The Stop hook chain runs four scripts (`forge-reflect-on-stop.sh`, `pk-lint-cron.sh`, `pk-focus-cleanup.sh`, `mem0-compress-on-stop.sh`) with `|| true` everywhere. Failures are silently swallowed. There is no record of which hook ran, which succeeded, which failed, or how long each took. Debugging hook misbehavior requires running each script manually and comparing.

## Evidence

Read `shared/scripts/forge-reflect-on-stop.sh` and the other Stop-chain scripts. Note the `|| true` pattern at the end of each invocation. There is no log file written; nothing tells the user "this hook fired, this one didn't."

## Why it matters

Several downstream tasks depend on hooks running reliably:

- SP-013 wires sycophancy correction into a SubagentStop hook; if it silently fails, the entire correction pipeline is bypassed without notice.
- SP-014 verifies a fallback matcher claim; without observability, "we asserted it works" is the only evidence.
- XC-004 (`prometheus doctor`) needs a hook-run log to validate end-to-end loop health.

This is P0 because it gates so much else, and because the *current state* of the system has no diagnostics.

## Proposed fix

Add a small bash logging shim sourced by every hook script. The shim writes to `~/.prometheus/hooks.log` in JSONL format:

```json
{"ts":"2026-05-09T10:23:11Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"event":"start","session_id":"..."}
{"ts":"2026-05-09T10:23:14Z","hook":"Stop","script":"forge-reflect-on-stop.sh","pid":12345,"event":"end","exit_code":0,"duration_ms":2987}
```

Plus an `event:"error"` line on non-zero exit.

The shim provides three functions: `hook_log_start`, `hook_log_end`, `hook_log_error`. Each script wraps its body with start/end calls.

## Trade-offs and risks

- **Disk-space growth.** The log grows unbounded. Mitigation: a daily logrotate config writing to `~/.prometheus/hooks.log.YYYY-MM-DD.gz` and keeping 30 days. The logrotate config ships with the skill-pack.
- **Concurrency.** Multiple hooks may write simultaneously. Use `flock` on the log file to serialize writes; performance impact is negligible (microseconds).
- **Sensitive content.** Some scripts process prompts that may contain sensitive content. The log records script names and exit codes only — not script *output*. This is deliberate and documented.

## Acceptance criteria

- [ ] `~/.prometheus/hooks.log` exists and contains JSONL entries after a session.
- [ ] Every Stop-chain script writes a start and end event with matching pids.
- [ ] Failed scripts (non-zero exit) write an error event with a stderr snippet.
- [ ] Log rotation works (test by simulating older logs).
- [ ] `prometheus doctor` (XC-004) can read the log to validate hook health.

## Implementation steps

1. Write `shared/scripts/lib/hook-log.sh` with the three functions and `flock` serialization.
2. Source it from each Stop-chain script. Wrap the script body.
3. Replace the trailing `|| true` with `|| hook_log_error "$LINENO"` to capture the failure mode.
4. Add a logrotate config to `shared/config/logrotate.d/prometheus-hooks` and document how to install it (`sudo cp` step).
5. Test by invoking a Stop event manually.
6. Test by intentionally breaking one script (`exit 1`) and confirming the error event appears.

## Dependencies

None.

## Open questions

- Should `~/.prometheus/hooks.log` also include UserPromptSubmit and PreToolUse hook events? Recommend yes — same shim, same format. Scope: include them in this task's implementation since they share the shim.
- Does `prometheus doctor` need a tail-N command to view recent hook events? Yes, but that's an XC-004 concern.
