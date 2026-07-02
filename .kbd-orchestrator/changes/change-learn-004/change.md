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

## Tasks

- [x] Implement `--kb` flag in `content-grounding-kb.sh` with adapter dispatch logic
- [x] Add Dify KB adapter using `dify_search` and `dify_list_documents` MCP tools
- [x] Add local file adapter that ingests via `palace_ingest` into surreal-memory
- [x] Add surreal-memory palace adapter using `palace_search` and `palace_recall`
- [x] Document privacy model: KB content stays local, never forwarded to external APIs or LLM grounding calls
