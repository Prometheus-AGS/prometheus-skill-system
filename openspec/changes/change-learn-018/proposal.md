---
id: change-learn-018
title: learn-harness skill (per-harness capability orientation)
type: skill
status: DONE
phase: phase-learn-feynman
depends_on:
  - change-learn-016
  - change-learn-002
  - change-learn-006
---

# change-learn-018 — learn-harness skill

## Summary

Add a `learn-harness` skill that orients operators to the capabilities of a
specific AI harness (Claude Code, OpenCode, Codex, Kimi, Zed). The `--harness`
flag selects the target; when omitted, the skill auto-detects the running
harness. A short-circuit mode emits a capability map without running the
Feynman loop. Reference files document each harness's skill, MCP, hook, and
AskUserQuestion support. A cross-harness parity table covers learn domain
skills.

## Motivation

Operators frequently have questions about what the skill pack can do on their
specific harness. This skill gives a direct, harness-specific answer and
optionally routes into the Feynman loop for deeper understanding.

## Scope

- New skill directory: `skills/learn/learn-harness/`
- Reference files for five harnesses
- Cross-harness parity table
- Short-circuit capability map option
