---
id: SP-015
title: hooks.json symlink fix
status: ready
priority: P2
estimated_effort: 0.5d
agent_role: skill-pack-maintainer
depends_on: []
unblocks: []
related: []
created_from_conversation_turn: 3-4
---

# SP-015 — hooks.json symlink fix

## Problem

Two `hooks.json` files exist in the skill-pack:

- `hooks/hooks.json`
- `.claude-plugin/hooks/hooks.json`

These are committed as **identical content** rather than as a symlink + target. Every change must be made in both places. Drift is inevitable as one gets updated and the other forgotten.

## Evidence

```
$ ls -la hooks/hooks.json .claude-plugin/hooks/hooks.json
# Both report regular files of identical size and content
$ diff hooks/hooks.json .claude-plugin/hooks/hooks.json
# (no output — currently identical)
```

The drift hasn't happened yet; this task prevents it.

## Why it matters

Several upcoming tasks (SP-006, SP-009, SP-011, SP-012, SP-013) modify hook entries. If the maintainer forgets one of the two locations, the modification fires in some contexts and not others, producing inconsistent enforcement.

## Proposed fix

Make one of the two the canonical file; symlink the other to it.

The Claude Code plugin contract typically expects the file inside `.claude-plugin/hooks/hooks.json`. Keep that as the canonical. Replace `hooks/hooks.json` with a symlink:

```bash
cd /Users/gqadonis/Projects/prometheus/prometheus-skill-pack
rm hooks/hooks.json
ln -s ../.claude-plugin/hooks/hooks.json hooks/hooks.json
git add -A && git commit -m "fix(SP-015): symlink hooks.json to single canonical source"
```

Verify the symlink is preserved through git (it should be — git tracks symlinks correctly).

## Trade-offs and risks

- **Risk: symlinks behave differently on Windows.** If any developer in the project is on Windows, the symlink becomes a regular file and the drift returns. Mitigation: document the convention; CI check (one line) that `hooks/hooks.json` is a symlink.
- **Risk: the legacy `hooks/` location is referenced by something we miss.** Mitigation: grep `.claude-plugin/`, `shared/`, `scripts/` for the path strings; verify both resolve correctly via the symlink.

## Acceptance criteria

- [ ] `hooks/hooks.json` is a symlink to `../.claude-plugin/hooks/hooks.json`.
- [ ] Reading either path yields the same content.
- [ ] CI check rejects PRs that replace the symlink with a regular file.
- [ ] No tooling references break (verified by running a sample session and observing both paths resolve).

## Implementation steps

1. Verify both files have identical content. If not, reconcile first (bring the canonical to its merged best state).
2. Replace `hooks/hooks.json` with the symlink.
3. Add a one-line CI check (e.g. `test -L hooks/hooks.json`).
4. Update any documentation that mentions the dual-file pattern.

## Dependencies

None.

## Open questions

- Is there a Claude Code-internal contract that expects `hooks/hooks.json` to exist as a regular file? Verify by reading the plugin manifest schema. If yes, swap which is canonical.
