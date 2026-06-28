---
id: change-learn-004
title: "content-grounding KB adapter (custom knowledge bases)"
type: design
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-003
---

# change-learn-004: content-grounding KB adapter (custom knowledge bases)

## Problem

Users with proprietary knowledge bases (Dify, local files, surreal-memory palace)
cannot use the public corpus service from `change-learn-003` without leaking
private content to external APIs.

## Proposal

Extend content-grounding with a `--kb` flag and a companion
`content-grounding-kb.sh` script that routes to one of three adapters: Dify
(via MCP tools), local file (via `palace_ingest`), or surreal-memory palace
(via `palace_search`/`palace_recall`). KB content is never forwarded to external
APIs.

## Outcome

A privacy-safe grounding path for custom knowledge bases, usable by any
learn-* skill via the same `--kb` interface.
