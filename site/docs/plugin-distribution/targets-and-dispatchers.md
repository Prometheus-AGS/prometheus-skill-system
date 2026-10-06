---
title: Targets and stable dispatchers
description: The 14-target matrix, copy targets, symlink targets, and hook routing.
---

# Targets and stable dispatchers

The manifest declares fourteen logical skill targets. Codex resolves a nonempty inherited `CODEX_HOME` first, otherwise the selected `--home` or process home plus `.codex`; its row below describes the logical target, not a promise that a custom root lives under the default home. Paths are normalized consistently across installation, verification, rollback and uninstall:

| Target | Mode |
| --- | --- |
| `.claude/skills` | symlink |
| `.opencode/skills` | symlink |
| `.kimi-code/skills` | symlink |
| `.minimax/skills` | verified copy |
| `.cursor/skills` | symlink |
| `.codex/skills` | verified copy |
| `.gemini/skills` | symlink |
| `.roo/skills` | symlink |
| `.windsurf/skills` | symlink |
| `.codeium/windsurf/skills` | symlink |
| `.agents/skills` | symlink |
| `.config/zed/skills` | symlink |
| `.zed/skills` | symlink |
| `.cline/skills` | symlink |

Codex and MiniMax require real directories, so the installer copies their
skills. Every target receives an Ed25519-signed receipt that binds its payload
mode and hash to the generation and canonical skill-index hash. Every other
target links through the active generation. A destination collision is
preserved and reported; the installer does not overwrite unrelated user
content.

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
