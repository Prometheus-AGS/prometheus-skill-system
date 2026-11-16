---
type: SessionRecord
id: karpathy-session-9efa6d3e7c3cd19b
title: Karpathy session 9efa6d3e7c3c
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T11:10:39.860055+00:00
created_at: 2026-11-16T11:10:39.860055+00:00
updated_at: 2026-11-16T11:10:39.860055+00:00
revision: 0
---

## Delta

The KBD registry is pruned: it went from 165 registrations to 20, using the CLI's own command, `prometheus kbd projects --prune-missing --apply`.

- **Removed, 139 test fixtures:** temp-dir projects under `/private/var/folders/…/T/` (`kbd-boundary-*`, `stage-gate-*`, `phase-close-*`, `contract-adopt-*`, and two `tmp.*`).
- **Also removed, 6 deleted worktrees:** the command prunes every registration whose checkout path is gone and has no filter, so it also took 6 worktree registrations whose directories no longer exist. Two belonged to `universal-agent-runtime` and four to `prometheus-skills-mini`; the main checkouts of both projects are still registered.
- **Checked:** none of the 145 removed paths exists on disk, a second inventory finds nothing left to prune, and `prometheus kbd status` still resolves this project at revision 198.

Only registry entries were removed. The command wrote a backup of the previous registry, with a checksum and receipt, under `~/Library/Application Support/prometheus/kbd/registry-maintenance-backups/20261116T111007.653371Z-7fb2a338-e41e-4ad5-bdea-eb99e6682486/`.

The per-project store directories were not touched: 141 of the 158 under `kbd/projects/` are now unreferenced, but together they hold almost no data, so I left them.

This does not bring the old `sovereign-sync` daemon back into play; it stays unloaded.

The exec pin is still uncommitted. Its second-checkout test build is running in the background, and I'll commit and push when it passes.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T11:10:37.592559Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- config/prometheus-exec-binary.json
- crates/prometheus-exec/rust-toolchain.toml
- dist/plugins/claude/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- dist/plugins/codex/prometheus-skill-pack/skills/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/build-prometheus-exec.sh
- scripts/install-prometheus-exec.sh
- scripts/tests/install-prometheus-exec.test.sh
- skills/process/delivery-cadence/scripts/refresh-skill-pack.sh
- scripts/prometheus-exec-pin.sh
