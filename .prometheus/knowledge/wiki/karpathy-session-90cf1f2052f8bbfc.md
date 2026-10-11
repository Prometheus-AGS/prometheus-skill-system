---
type: SessionRecord
id: karpathy-session-90cf1f2052f8bbfc
title: Karpathy session 90cf1f2052f8
tags:
- karpathy
- session-learning
sources:
- resource: session:e2256471-a793-4282-b09d-2295a42e1e28
generated:
  by: pk/1.9.0
  at: 2026-10-04T13:15:27.103579+00:00
created_at: 2026-10-04T13:15:27.103579+00:00
updated_at: 2026-10-04T13:15:27.103579+00:00
revision: 0
---

## Delta

Under Claude Code, every mini hook was running without its payload. Claude Code passes hooks a socket on stdin, and the old check only read pipes and files. The fix is in PR [Prometheus-AGS/prometheus-skills-mini#30](https://github.com/Prometheus-AGS/prometheus-skills-mini/pull/30), ready for you to merge.

**1. Probe results.** I ran a hook that records what stdin is, with scratch `HOME`, `CODEX_HOME` and `PROMETHEUS_PLUGIN_ROOT`. No real config or plugin cache was touched.

| Harness | stdin type | Payload | Closed at end |
|---|---|---|---|
| Claude Code 2.1.289 (exec form and shell form) | **socket** | yes | yes |
| codex-cli 0.158.0 | pipe (FIFO) | yes | yes |

Under Claude Code, every hook received `{}` and still exited 0. Codex was not affected. To make Codex run unreviewed hooks for the probe, I used `codex exec --dangerously-bypass-hook-trust`; otherwise it skips them.

**2. Fix in `scripts/hook-entry.mjs`.**
- Pipes and files are still read to the end, as before.
- A socket is now read until it closes, it goes quiet for 500 ms, or 16 MiB has arrived, whichever comes first. The socket `node --test` provides never closes, so it now costs about 500 ms instead of hanging.
- Once the read finishes, the hook stops reading the socket but leaves it open, so the same descriptor can't be reused later in the process.
- Hooks keep exec form and seconds timeouts. `versions.toml` is untouched.
- I regenerated `dist/`, and `check:distribution` reports no drift.

**3. Tests in `scripts/hook-entry.test.mjs`.** The new tests spawn the real hook-entry with a socket stdin:
- **Payload arrives:** the payload's `cwd` names a paused project while the process runs in another directory. The hook only prints `KBD REANCHOR` if it actually got the payload.
- **Never closed:** a payload is written to a socket that is never closed; it still arrives and the process exits.
- **No hang:** a socket that is never written or closed gives empty input and exits within the idle limit.
- **In-process:** a dispatch with no stdin option doesn't hang under `node --test`.

I also checked the two payload tests against the old code, and both fail there.

**Test results.** All 16 tests in `hook-entry.test.mjs` pass. The full suite has 1053 tests: 1033 pass and 18 fail, and all 18 are on your 19-test baseline list, so there are no new failures. The 19th, "validation passes on this repository…", now passes. The first full run showed 9 extra OpenSpec failures because the new worktree had no `node_modules`; after `npm ci` they went away.

**Not changed:**
- `record-progress.mjs` and the two adversarial-review scripts use the same pipe-or-file check. Skills run them through a shell pipe, not as harness hooks, so I left them alone and only corrected a comment in `record-progress.test.mjs` that described the old behaviour.
- A memory from an earlier probe says codex-cli 0.158 ignores the `args` field of a hook. If that's right, mini's exec-form hooks never run in Codex at all. That needs its own check; I've noted it in the PR.

The branch is `fix/hook-entry-socket-stdin`, in a new worktree at `/Users/gqadonis/Projects/prometheus/worktrees/mini-socket-stdin`. Your main mini checkout, which has uncommitted changes, wasn't touched. I also saved the socket-versus-pipe finding to memory for future hook work.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: e2256471-a793-4282-b09d-2295a42e1e28
- Captured: 2026-10-04T13:15:18.560371Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.claude/worktrees/goofy-ritchie-6e4a6c

## Changed Paths

- No changed paths detected.
