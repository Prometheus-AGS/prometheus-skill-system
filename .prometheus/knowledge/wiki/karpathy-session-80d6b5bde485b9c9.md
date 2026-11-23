---
type: SessionRecord
id: karpathy-session-80d6b5bde485b9c9
title: Karpathy session 80d6b5bde485
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T13:36:11.469421+00:00
created_at: 2026-11-17T13:36:11.469421+00:00
updated_at: 2026-11-17T13:36:11.469421+00:00
revision: 0
---

## Delta

I've read the installer and written a design for domain selection; no installer code has changed yet.

## The design

**The signed generation stays complete, and selection only filters what is projected.** The generation keeps every skill, the hook runtime and the shared scripts. Selection decides which skills are copied into `~/.agents/skills` and linked into `~/.claude/skills` and `~/.qwen/skills`. Hooks and rollback are unaffected, and a skill that is not selected can still be run as `/skill-name`.

**Domains come from the PR #231 plugin manifests.** At staging the installer writes a signed `indexes/domains.json` mapping each domain to its skills, and fails if any skill has no domain. The 21 imported skills (artifact-refiner and sycophancy-correction) need domain entries added.

**New flags:** `--domains`, `--add-domains`, `--remove-domains` and `--list-domains`. The selection is stored signed in the plugin store, so an update with no flag keeps it. Selection is per harness target, so Claude can be narrow while the Codex root stays wide.

**Receipts and recovery carry the selection.** Receipts move to a second schema that records the selection, and old receipts are read as "everything", so existing generations still verify. The activation journal records the selection so an interrupted install recovers to the same set.

## Two things the reading turned up

- **The installer never removes a projected skill.** A skill dropped from a later generation stays in the target folder until uninstall. Selection needs a removal step anyway, limited to entries the pack provably owns, and that step closes this gap too.
- **Skill references cross domains often.** On the PR #231 tree, 326 references stay inside a domain and 84 cross one. The process domain names skills in five other domains because the KBD orchestrator routes to them. Following every reference would pull nearly everything back in, so the design uses a short declared list of hard requirements instead.

## Defaults I would use

1. **No stored selection means install everything, as today.** No existing machine changes until its operator chooses. Doctor reports the listing size against the budget. The alternative is for a fresh interactive install to ask.
2. **Hard requirements: process requires iterative-evolver and testing (for VLBV).** Every other cross-domain reference is soft and reported by `--verify`, not enforced. The alternative is no hard requirements at all.
3. **Doctor warns when Claude Code has a pack plugin enabled and user-level pack skills as well.** Enabling a plugin on top of the installer lists the same skill more than once.

On this machine, `~/.claude/skills` holds 244 pack-owned links while the active generation has 219 skills. I have not inspected the extra 25. Nothing is touched until an install runs with the new removal step.

## Scope if you approve

The change touches about ten functions in `scripts/install-plugin-generation.js`, five caller scripts and the `cowork` CLI pass-through. Verification is one integration batch through the real installer in a scratch home with seven scenarios, then the Rust and KBD trigger cases against a narrow selection. It builds on PR #231, which is still open.

The design is in the scratch worktree and will not survive a reboot until it is committed:
`/private/tmp/claude-501/-Users-gqadonis--claude-spares-spare-IpXauP/1db84095-da34-4e96-ab6a-dfa84d559c8f/scratchpad/psp-domains/docs/plans/2026-11-17-installer-domain-selection.md`

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T13:35:38.328343Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
