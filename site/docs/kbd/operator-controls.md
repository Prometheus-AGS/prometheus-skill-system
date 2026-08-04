---
id: operator-controls
title: Operator Controls
sidebar_label: Operator Controls
---

# Operator Controls

Operator intent outranks agent continuation. KBD exposes explicit pause,
revise, resume, cancel, audit, and watch commands through portable skills, the
CLI, MCP, and REST.

## Status

```bash
prometheus kbd --path "/path/to/project" status
prometheus kbd --path "/path/to/project" status --json | jq .
```

Status reports the run and project IDs, committed revision, lifecycle, plan
revision, checkpoint, exact next work, active path, completion dimensions, and
actor/device provenance.

## Pause

```bash
prometheus kbd --path "/path/to/project" pause \
  --reason "Pause before database maintenance"
```

Pause records an advisory operator checkpoint containing:

- prior lifecycle;
- last completed work;
- exact next work;
- decisions and blockers;
- dirty-work summary;
- plan revision.

Pause does **not** intercept Bash, Python, Edit, or Write. Agents are expected to
honor the checkpoint, while concurrent command safety remains enforced by the
exclusive journal transaction and causal-frontier validation. Resume changes
the lifecycle state; it does not unlock shell tools because they were never locked.

## Revise an active plan

Never overwrite a prior plan decision. Record a new immutable revision:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "/path/to/project" revise \
  --reason "The upstream API removed the selected endpoint" \
  --exact-next-work "Adopt the supported batch endpoint and update acceptance tests"
```

Revision `N+1` supersedes the previous next-work pointer while preserving the
auditable history.

## Resume

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "/path/to/project" resume
```

To pin the expected plan revision:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "/path/to/project" resume --plan-revision 4
```

Resume refuses a plan-revision mismatch.

## Cancel

```bash
prometheus kbd --path "/path/to/project" cancel \
  --reason "Operator abandoned this run"
```

Cancellation is terminal and preserves the full audit history. Start a new run
for future work.

## Audit and watch

```bash
# Human-readable immutable history
prometheus kbd --path "/path/to/project" audit

# JSON from a revision or event ID
prometheus kbd --path "/path/to/project" audit --since 20 --json | jq .

# Follow newly committed events
prometheus kbd --path "/path/to/project" watch
```

Audit is allowed while paused. Read-only inspection never grants write
authority.

## Portable skills

The same operator contract is available as:

```text
/kbd-status
/kbd-pause
/kbd-resume
/kbd-cancel
/kbd-audit
```

These skills call the canonical CLI when available and preserve a bounded
compatibility fallback for legacy projects. A normal assistant response,
missing footer, or Stop-hook event is never interpreted as a resume command.
