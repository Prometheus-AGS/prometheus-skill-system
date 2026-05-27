---
license: MIT
name: kbd-inject-agent-rules
version: '1.0.0'
description: >
  Idempotently inject a fenced "Agent rules" block (Karpathy's 4 think-first
  principles + Boris Cherny's 4 Claude Code workflow principles) into the
  target project's CLAUDE.md and/or AGENTS.md. Re-runnable — overwrites only
  the fenced region; everything else is byte-preserved. Supports --refresh
  to re-validate the cached source URLs, and --dry-run for a diff preview.
metadata:
  tags: [process, orchestration, docs, agent-rules]
---

# /kbd-inject-agent-rules

Inject the canonical Karpathy + Claude-Code-author rules into a project's
agent context files.

## What this does

1. Resolves target files from `--target` (default both) at `--path` (default `.`).
2. Optionally with `--refresh`, curl-probes each cached source URL for its anchor keyword and updates the cache's `Last fetched` dates. Refresh failures warn but don't abort.
3. Builds the fenced-region content from `references/template.md`.
4. For each target file:
   - If no marker present: appends the fenced region.
   - If valid marker pair present: replaces the region in place.
   - If markers corrupt (missing end, multiple starts): refuses to write.
5. With `--dry-run`, prints `diff -u` previews and modifies nothing.
6. Writes atomically (temp file + `mv`) so an interrupted invocation never leaves a half-rewritten file.

## When to use

Whenever you want a project's `CLAUDE.md` / `AGENTS.md` to carry the canonical Karpathy + Claude Code rule sets. Run once when setting up a new project, then re-run periodically (or after `--refresh`) to pick up updated cache content.

## Progress Signals (MANDATORY)

```
Starting kbd-inject-agent-rules — <target-summary>
Completed kbd-inject-agent-rules — <count> file(s) updated, <count> unchanged
```

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-inject-agent-rules/kbd-inject-agent-rules.sh" \
  [--target CLAUDE.md|AGENTS.md|both] \
  [--path <project-root>] \
  [--refresh] \
  [--dry-run]
```

## Examples

```
/kbd-inject-agent-rules                                    # both files at .
/kbd-inject-agent-rules --target CLAUDE.md                 # CLAUDE.md only
/kbd-inject-agent-rules --path /Users/me/my-project        # different root
/kbd-inject-agent-rules --dry-run                          # preview only
/kbd-inject-agent-rules --refresh                          # re-validate sources
```

## Refusal cases

- Target file contains a `<!-- agent-rules:start ... -->` marker without a matching `<!-- agent-rules:end -->` → refuse, suggest manual repair.
- Target file contains multiple `<!-- agent-rules:start ... -->` markers → refuse, suggest deduplication.
- `--target` value is not one of `CLAUDE.md` / `AGENTS.md` / `both` → usage error.

## Reference

- Cached rules + sources: `references/rules-cache.md`
- Fenced-region template: `references/template.md`
