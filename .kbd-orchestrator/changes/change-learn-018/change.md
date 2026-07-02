---
id: change-learn-018
title: learn-harness skill (per-harness capability orientation)
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-016
  - change-learn-002
  - change-learn-006
---

# change-learn-018 — learn-harness skill

## Summary

Add a `learn-harness` skill that orients operators to the capabilities of a
specific AI harness (Claude Code, OpenCode, Codex, Kimi, Zed). The `--harness`
flag selects the target; when omitted, the skill auto-detects the running
harness. A short-circuit mode emits a capability map without running the
Feynman loop. Reference files document each harness's skill, MCP, hook, and
AskUserQuestion support. A cross-harness parity table covers learn domain
skills.

## Motivation

Operators frequently have questions about what the skill pack can do on their
specific harness. This skill gives a direct, harness-specific answer and
optionally routes into the Feynman loop for deeper understanding.

## Scope

- New skill directory: `skills/learn/learn-harness/`
- Reference files for five harnesses
- Cross-harness parity table
- Short-circuit capability map option

## Tasks

- [x] Write `skills/learn/learn-harness/SKILL.md` with frontmatter, overview, `--harness` flag documentation (`claude-code` | `opencode` | `codex` | `kimi` | `zed`), auto-detection logic description, and short-circuit capability map option
- [x] Write `skills/learn/learn-harness/references/harness-claude-code.md` documenting: skills (flat install, `/skill-name` invocation), MCP servers, hooks (PreToolUse/PostToolUse/Stop), `AskUserQuestion` support, `plugin.json` structure, and learn domain skill availability
- [x] Write `skills/learn/learn-harness/references/harness-opencode.md`, `harness-codex.md`, `harness-kimi.md`, and `harness-zed.md` using the same section structure as `harness-claude-code.md`, noting capability gaps per harness
- [x] Implement short-circuit capability map option (`--map-only`): read the appropriate harness reference file and emit a structured capability summary without invoking the Feynman loop
- [x] Add cross-harness parity table to `skills/learn/learn-harness/references/parity-table.md` listing each learn domain skill and its availability (full / partial / unavailable) per harness with rationale
