---
id: SP-017
title: Slash command merge strategy (skill-pack vs prometheus-knowledge)
status: ready
priority: P2
estimated_effort: 1d
agent_role: skill-pack-maintainer
depends_on: []
unblocks: []
related: [SP-001, SP-016]
created_from_conversation_turn: 3-4
---

# SP-017 — Slash command merge strategy

## Problem

Both `prometheus-skill-pack` and `prometheus-knowledge` ship slash commands (e.g. `/focus`, `/ingest`). When both are installed in the same Claude Code environment, there is no merge strategy. Behavior depends on load order — last-loaded wins, which is non-deterministic and version-dependent.

## Evidence

1. List `.claude/commands/*.md` in each repo. Note overlapping command names.
2. Confirm: no documented precedence rule; no version-coupling.

## Why it matters

Slash commands are the user's primary control surface. Inconsistent dispatch means a user types `/focus` and gets different behavior on different machines or after different upgrades.

## Proposed fix

Three-part fix:

**1. Audit and rename.** For overlapping commands, rename to be unambiguous. Convention: pack-level commands get a generic name; knowledge-specific commands get a `pk-` prefix. Concretely:

- `/focus` (currently in both) → keep in skill-pack as `/focus` (the higher-level user-facing one); rename in pk to `/pk-focus` for the librarian-internal version.
- `/ingest` → keep in pk as `/pk-ingest` (it's pk-specific anyway); remove from skill-pack if shipped there.

**2. Manifest convention.** Document in `prometheus-skill-pack/CLAUDE.md` (per SP-001's canonical location) that the pack owns generic top-level command names and downstream packages use prefixes. Update `prometheus-knowledge/CLAUDE.md` with the prefix rule.

**3. Conflict detection.** A small script `scripts/detect-command-conflicts.sh` that, given the union of `.claude/commands/` from any installed pack(s), reports overlapping command names. Run as a CI check on the pack itself; recommend users invoke in their environments.

## Trade-offs and risks

- **Risk: renaming breaks user muscle memory.** Mitigation: ship aliases for the renamed commands for one deprecation cycle (e.g. `/focus` in pk continues to work but emits a deprecation message pointing to `/pk-focus`).
- **Risk: future packs ship more conflicting commands.** Mitigation: the prefix convention is documented; maintainer review of new packs catches violations.

## Acceptance criteria

- [ ] All slash commands across pack and pk have unambiguous names.
- [ ] CLAUDE.md documents the prefix convention.
- [ ] CI script detects new conflicts.
- [ ] Deprecation aliases ship for renamed commands (one cycle minimum).

## Implementation steps

1. List all current commands in pack and pk.
2. Identify overlaps.
3. Apply rename convention.
4. Add deprecation aliases.
5. Update documentation.
6. Add CI script.

## Dependencies

None functional. Recommended after SP-001 so the canonical CLAUDE.md is the documented place for the convention.

## Open questions

- Are there third-party packs in the wild that already use generic command names? If so, the convention may need adjusting. Inventory the ecosystem if it has more packs than just the two named.
