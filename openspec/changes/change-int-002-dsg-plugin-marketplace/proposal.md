---
id: change-int-002-dsg-plugin-marketplace
title: dsg plugin.json + marketplace listing
phase: cowork-integration
priority: P1
effort: S
wave: 5
agent: general-purpose
status: done
gap_id: G-04-dsg, G-05-dsg
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - tools/disk-space-guardian/.claude-plugin/plugin.json (new — inside submodule)
  - .claude-plugin/marketplace.json (add dsg entry)
---

# change-int-002 — dsg plugin.json + marketplace listing

## Context

The dsg submodule is now registered. It needs a Claude Code plugin manifest
(`plugin.json`) so it can be installed as a standalone plugin, and an entry in
the skill-pack's marketplace.json for discovery.

## Strategy

1. Create `tools/disk-space-guardian/.claude-plugin/plugin.json` — minimal
   plugin manifest declaring the dsg CLI tool
2. Add dsg entry to `.claude-plugin/marketplace.json`
3. Commit the submodule pointer update + marketplace change together

## Scope

1. Create `tools/disk-space-guardian/.claude-plugin/plugin.json`
2. Update `.claude-plugin/marketplace.json` with dsg entry
3. Update KBD orchestrator
4. Commit (submodule pointer + skill-pack changes)

## Verification

- `tools/disk-space-guardian/.claude-plugin/plugin.json` is valid JSON
- `.claude-plugin/marketplace.json` contains disk-space-guardian entry
- `git submodule status` shows updated pointer
