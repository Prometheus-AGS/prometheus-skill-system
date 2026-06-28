# Tasks — change-learn-018

- [ ] Write `skills/learn/learn-harness/SKILL.md` with frontmatter, overview, `--harness` flag documentation (`claude-code` | `opencode` | `codex` | `kimi` | `zed`), auto-detection logic description, and short-circuit capability map option
- [ ] Write `skills/learn/learn-harness/references/harness-claude-code.md` documenting: skills (flat install, `/skill-name` invocation), MCP servers, hooks (PreToolUse/PostToolUse/Stop), `AskUserQuestion` support, `plugin.json` structure, and learn domain skill availability
- [ ] Write `skills/learn/learn-harness/references/harness-opencode.md`, `harness-codex.md`, `harness-kimi.md`, and `harness-zed.md` using the same section structure as `harness-claude-code.md`, noting capability gaps per harness
- [ ] Implement short-circuit capability map option (`--map-only`): read the appropriate harness reference file and emit a structured capability summary without invoking the Feynman loop
- [ ] Add cross-harness parity table to `skills/learn/learn-harness/references/parity-table.md` listing each learn domain skill and its availability (full / partial / unavailable) per harness with rationale
