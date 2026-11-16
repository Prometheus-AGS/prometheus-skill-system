---
title: Targets and stable dispatchers
description: The canonical-root target matrix, the Claude Code and Qwen Code shims, retired targets, and hook routing.
---

# Targets and stable dispatchers

The manifest declares three logical skill targets (owner decision D3). Skills
are installed **once** into the canonical root `~/.agents/skills` as real
directories, and two symlink shims serve the harnesses that do not read it:

| Target | Mode |
| --- | --- |
| `.agents/skills` | verified copy |
| `.claude/skills` | symlink |
| `.qwen/skills` | symlink |

`~/.agents/skills` is read directly by Codex (verified on codex-cli 0.166.0,
which ignores symlinked skill directories, hence copies), OpenCode, Kimi Code,
MiniMax, Cursor, Gemini CLI, Zed, Cline and the other AgentSkills harnesses.
When `~/.agents/skills` is itself a symlink to a shared skills tree, the
installer resolves it with `realpath` and installs into the directory it names;
the link is never replaced. Claude Code and Qwen Code do not read the canonical
root, so their shims link each skill through the active generation.

Every target receives an Ed25519-signed receipt that binds its payload mode and
hash to the generation and canonical skill-index hash. A destination collision
is preserved and reported; the installer does not overwrite unrelated user
content.

## Retired targets

The former per-harness roots are retired: `.opencode/skills`,
`.kimi-code/skills`, `.minimax/skills`, `.cursor/skills`, `.codex/skills`
(resolved through a nonempty `CODEX_HOME`), `.gemini/skills`, `.roo/skills`,
`.devin/skills`, `.codeium/windsurf/skills`, `.config/zed/skills`,
`.zed/skills` and `.cline/skills`. On install the generation installer removes
only entries the pack provably owns there (a symlink into the plugin root, or a
copy whose generation marker verifies) and records them in
`<pluginRoot>/receipts/retired-target-cleanup.json`. Everything else, and the
directories themselves, stay in place. A retired root aliased onto a live one is
never cleaned. Generations signed with the earlier 14-target matrix remain
verifiable for rollback and retirement.

Stable dispatchers live under
`~/.prometheus/plugins/prometheus-skill-pack/stable/`. They resolve required
scripts and helpers through `current`, including hook dispatch, project
detection, memory outbox flush, learning enqueue, `pk` health, and the skill
index. Host configuration points to these stable paths, so activation and
rollback do not require rewriting hook registrations.

Signed store receipts and actual selected projection checks are distinct. A syntactically valid generation marker alone does not make an unrelated directory managed. Ownership must bind to a known verified generation or applicable prior ownership receipt. Unknown, malformed and unmanaged entries are preserved with diagnostics.

Verification rejects missing or unsigned receipts, wrong modes or hashes, index
parity drift, a dispatcher that resolves outside `generations/`, or a target
still resolving through a stale version path.
