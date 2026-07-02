---
id: change-okf-006-okf-lint
title: OKF v0.1 conformance rules in pk lint with permissive-consumption semantics
phase: phase-okf-llm-wiki-adoption
gaps: [Goal5]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: pending
model_class: medium
depends_on: [change-okf-003, change-okf-005]
scope:
  - prometheus-knowledge-rs (lint pass implementation)
---

# change-okf-006 — OKF lint

## Context

pk lint checks pk-native invariants only. OKF §9 defines conformance: every
non-reserved .md has parseable frontmatter with non-empty `type`; reserved
files follow §6/§7 structure. Permissive consumption means optional-field
gaps, unknown types, and broken links are warnings — never rejections.

## Tasks

- [ ] Error: unparseable frontmatter; empty/missing type on non-reserved .md
- [ ] Warning: missing recommended fields; broken cross-links; orphan pages;
      index.md entries missing descriptions
- [ ] Reserved-file structure checks (index.md no frontmatter except bundle
      root okf_version; log.md ISO date headings)
- [ ] Auto-fixables wired into pk lint --fix (e.g. derive title from filename)
- [ ] Verify: lint fixture bundle — conformant passes clean; violations
      classified at correct severity
