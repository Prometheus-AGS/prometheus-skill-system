## Why

An `openspec update` rewrote the vendored harness trees. This is NOT churn: the diff is
+4016/-1473, and of 54 changed lines in the one file inspected, only 2 are `generatedBy`.
The rest is upstream feature content. Adopting it deliberately, in its own commit, keeps
the large diff legible and separates it from the two real decisions in this phase.

## What Changes

- Adopt 70 modified files: 40 `SKILL.md` mirrors + 30 command/workflow files across
  `.agent`, `.agents`, `.cursor`, `.opencode`.
- Verify the 30 command/workflow files carry the same upgrade — only a *skill* was
  inspected line-by-line; their content is still inference.
- Record the `generatedBy` 1.3.1 → 1.10.0 bump and the shipped features in the message.

## Impact

- Unblocks part of the 98-file tree that stalls `update-skill-pack.sh`.
- C-01 NOT triggered: verified no C-01 source (`.claude-plugin/*`, `.mcp.json`,
  `hooks/hooks.json`, `build-codex-plugin.js`) is in this change.
