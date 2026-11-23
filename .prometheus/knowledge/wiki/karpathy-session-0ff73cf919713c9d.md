---
type: SessionRecord
id: karpathy-session-0ff73cf919713c9d
title: Karpathy session 0ff73cf91971
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T12:10:57.887006+00:00
created_at: 2026-11-17T12:10:57.887006+00:00
updated_at: 2026-11-17T12:10:57.887006+00:00
revision: 0
---

## Delta

I stopped before building, because generated domain packages would not fix the triggering problem for anyone who installs the pack the documented way. Nothing was changed this turn.

## What I found

**The pack installer puts every skill at user level, whatever plugins are enabled.** The signed installer (`cowork install` / `update-skill-pack.sh`) copies every pack skill into `~/.agents/skills` and links `~/.claude/skills` to them. Plugins are a second, separate route to the same skills.

On this machine:

- `~/.agents/skills` has 991 entries and `~/.claude/skills` has 739.
- Both `prometheus-skill-pack` and `prometheus-process-skills` are enabled as plugins on top of that.
- So a KBD skill is listed three times here: bare, under the full-pack plugin, and under the process plugin.

A domain plugin shrinks the listing only for someone who installs through the Claude marketplace and never runs the installer.

**Hooks only work after the installer has run.** The hook entry point refuses to dispatch unless an operator has activated a verified generation with the installer. So anyone who wants the hooks (VLBV, learning, KBD) has to run the installer, and the installer gives them every skill.

**The installer has no domain selection.** It selects by harness target, not by skill domain.

Put together: the people who need hooks are exactly the people who get the full listing, and a generated domain package cannot change that.

## What would fix it

| Option | What it does | Cost |
|---|---|---|
| **A. Domain selection in the installer** | The installer takes a list of domains and installs only those skills at user level. Hooks and the shared runtime are unaffected, since they live in the activated store. | Touches the signed installer (3,299 lines), its receipts, uninstall and doctor. Largest change, and the only one that fixes the documented install. |
| **B. Generated domain packages** (what I was about to build) | Self-contained plugin packages with the shared runtime. Helps marketplace-only users. | Moderate. Does not help anyone who runs the installer. |
| **C. Stop the installer linking skills into `~/.claude/skills`** | Claude gets skills only through plugins; the installer keeps the store, hooks and other harnesses. | Smaller than A, but changes what an install means on Claude Code and needs your decision. |

I recommend A. It fixes the path your documentation tells people to use, it keeps hooks working, and the domain plugin manifests from PR #231 can serve as the domain definitions the installer reads.

One thing B would also have solved: flattening skills into a generated package lets KBD be its own plugin, since the hooks-file conflict only arises when the source directory is the plugin root. That is a small benefit and doesn't change the recommendation.

## Decision for you

Should I add domain selection to the installer (option A) instead of building the generated domain packages? Because it changes the signed installer, I'd start by reading how targets, receipts and uninstall work and come back with a concrete design before editing it.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T12:10:44.461767Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
