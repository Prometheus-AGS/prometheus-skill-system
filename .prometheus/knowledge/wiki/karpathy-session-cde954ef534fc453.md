---
type: SessionRecord
id: karpathy-session-cde954ef534fc453
title: Karpathy session cde954ef534f
tags:
- karpathy
- session-learning
sources:
- resource: session:cfc5bf77-8f81-45e6-975a-0283553c3962
generated:
  by: pk/1.11.0
  at: 2026-10-09T14:51:20.359975+00:00
created_at: 2026-10-09T14:51:20.359975+00:00
updated_at: 2026-10-09T14:51:20.359975+00:00
revision: 0
---

## Delta

The dirty-tree PR is open: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/174

The other open PR is the catalog one: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/173

**Merge note:** both PRs regenerate the same hook-bundle files. Whichever merges second needs them regenerated, and I'll do that when you tell me which one merged first. After both are in, the installed `prometheus` CLI needs a rebuild: it embeds `install-plugin-generation.js`, and until then the doctor's capability-hash check will complain, as it did earlier for the Codex-memory warning. The PR body says this too.

I'm ready for the new issue.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: cfc5bf77-8f81-45e6-975a-0283553c3962
- Captured: 2026-10-09T14:51:18.695606Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- tools/surreal-memory-server
