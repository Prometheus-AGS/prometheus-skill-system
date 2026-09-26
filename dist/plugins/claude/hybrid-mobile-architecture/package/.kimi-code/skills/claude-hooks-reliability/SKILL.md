---
name: claude-hooks-reliability
description: Diagnose, fix, and prevent silent hook failures in the agent hook chain. Use when a hook is not firing, when a skill did not trigger, when a matcher is too narrow or too broad, when a hook runner leaks processes, or when hook stdout corrupts a tool decision. Triggers on hook not firing, hook unreliable, matcher issue, hook timeout, PostToolUse, SessionStart, UserPromptSubmit, SubagentStop, hook bundle, hook runtime, hook script, dispatcher hash, hooks.json, settings.json hooks.
---
<!-- TJ-ARCH-MOB-001 compliant -->

# Hook Chain Reliability

Hooks fail silently. A hook that does not fire produces no error — the rule
simply never reaches the model, and the system "feels broken" with nothing in
any log. Every fix below converts a silent failure into a loud one.

Apply with `scripts/install-hooks-reliability.mjs <target>`; check with
`scripts/verify-hooks-reliability.mjs <target>`. The verifier is the inverse of
the installer and is what a consumer's `doctor` command runs.

## The 9 fixes

### W6.1 — Inline `bash -c '…'` is fragile

A long inline script buried in JSON has an unreadable quoting chain and cannot
be tested in isolation. Extract it to a checked-in file and have the JSON call
that file with arguments.

```json
{ "type": "command", "command": "bash $BUNDLE_ROOT/hooks/sessionstart-control.sh \"$1\" \"$2\"" }
```

Each script then tests standalone: `echo '{"event":"…"}' | bash the-script.sh`.

### W6.2 — Dispatcher SHA re-validated on every hook

Five SessionStart hooks pay five `shasum` calls before the first prompt. Cache
the validated digest briefly and skip re-hashing within the window.

```bash
CACHE="${TMPDIR:-/tmp}/.prom-hook-cache-${BUNDLE_ID}"
if [ ! -f "$CACHE" ] || [ "$(( $(date +%s) - $(stat -f %m "$CACHE" 2>/dev/null || stat -c %Y "$CACHE") ))" -ge 60 ]; then
  actual="$(shasum -a 256 "$DISPATCHER" | awk '{print $1}')"
  [ "$actual" = "$DISPATCHER_SHA" ] || exit 1
  printf '%s\n' "$actual" > "$CACHE"
fi
```

### W6.3 — Subagent matchers are fragile

Bare-string matchers match one exact agent name. A later `planner-v2` silently
never fires. Anchor matchers as regex alternations, and give the fallback entry
a real matcher so it does not fire for every subagent.

```json
{ "matcher": "^(planner|plan|planner-v2)$" }
```

### W6.4 — Hook stdout pollutes tool decisions

A `WARN:` line on stdout is parsed as the decision payload and the real
decision is discarded — the hook "ran but did not block". Redirect diagnostics
before any work so stdout carries only the decision.

```bash
#!/usr/bin/env bash
exec 2>>"$LOG_DIR/hooks.log"
```

Nothing but decision JSON may reach stdout.

### W6.5 — Hooks leak processes on timeout

A hook that backgrounds work and exits returns inside the timeout but leaks the
child. Spawn in a new process group, kill the group on timeout, then reap.

```bash
setsid bash -c "exec $HOOK_SCRIPT" & hook_pid=$!
wait "$hook_pid" || kill -KILL -"$hook_pid" 2>/dev/null || true
wait "$hook_pid" 2>/dev/null || true
```

### W6.6 — `SessionStart` matchers are too broad

A `*` matcher fires every SessionStart hook for every session, multiplying
cold-start cost across sessions that will never use them. Narrow the matcher to
the harness that actually consumes the hook.

### W6.7 — No structured hook-result log

With the bundle, the script, and the dispatcher each logging elsewhere, a
misfire has no single observable. Append one line per invocation to a single
NDJSON file.

```json
{"ts":"2026-08-20T12:34:56Z","hook_id":"sessionstart-control","harness":"claude-code","exit":0,"dur_ms":42}
```

### W6.8 — `UserPromptSubmit` entries omit `matcher`

An entry with no matcher is unconditional. It works while it is the only entry
and silently double-fires the moment a second is added. Give every entry an
explicit matcher, `"*"` included, so the intent survives the next edit.

### W6.9 — A `bash` interpreter sits in every hook path

Even with extracted scripts, a malformed environment variable or missing binary
breaks the whole chain. Route hooks through one small bundled dispatcher binary
that clears the environment, passes only a known-safe set, and is ABI-versioned.
Until that binary ships, treat W6.1 (extracted scripts) and W6.4 (stdout off
the decision channel) as the mitigation.

> **Ownership:** the dispatcher binary is **not** built in this package. This
> repository ships the guidance skill, not the hook runtime — it has no
> `crates/` and its own hooks live in `.claude/settings.json`. The binary
> belongs to `prometheus-skill-system`, which owns the runner it would replace.


## Verification

```bash
node scripts/verify-hooks-reliability.mjs .
```

Exits 0 when all nine hold, and names the violated fix otherwise.

## When not to use this

Plugin-installation problems and skill-registration problems are not hook
problems. Check that the skill is in the canonical registry first.
