---
type: SessionRecord
id: karpathy-session-ffe6cf11876873b8
title: Karpathy session ffe6cf118768
tags:
- karpathy
- session-learning
sources:
- resource: session:815e20bd-e2d6-4020-a972-f06fc10a8cac
generated:
  by: pk/1.11.0
  at: 2026-11-16T10:19:10.006088+00:00
created_at: 2026-11-16T10:19:10.006088+00:00
updated_at: 2026-11-16T10:19:10.006088+00:00
revision: 0
---

## Delta

The rules and docs implementer has finished. Its work is written but not yet run; that waits for the single end-of-work check. One of three implementers is still working.

**What it changed**
- **Rules:** A-9 is now "Complete the value phase, then verify once (VLBV)". Generated code runs only at B4 (once) or B5 (an owner-authorized repair). The hourly cadence and the compiler exception are both gone. The constitution also gained the policy block and a first-action step that works without hooks. The Rust and review rules match.
- **Rules re-rendered:** `node rules/build.mjs --check` passes. CLAUDE.md sits at 12,379 of its 12,500-character budget.
- **Context bootstrap:** it now generates the VLBV policy block. Its `--check` flags stale compiler-exception or `/opsx:` wording, but never rewrites text you wrote yourself.
- **Harness profiles:** new `config/harnesses/capabilities.json` with an evidence check (`check:harness-capabilities`), and `doctor profile` shows them. Only Codex's skill-root field is marked supported, backed by the full pack's evidence file.
- **Evals:** ported to Node for Claude Code, Codex, OpenCode and Kimi. Other harnesses are reported as not run.
- **liter-llm:** no change needed. Mini's Compose file already publishes the port on 127.0.0.1 only.
- **Docs:** `COMPARE.md` has a new "Exclusions" section, and there's a new `CHANGELOG.md` with a 1.16.0 entry.

**Passed to the VLBV implementer**
- **Rust skill:** `prometheus-rust-workspace` still describes the old compiler exception.
- **`method-context.md`:** still has the hourly wording.
- **Shared profile loader:** the handoff code should reuse the new profile loader instead of writing a second one.
- **Eval dependencies:** `kbd-boundary-authorize.mjs` and the closure-packet fields the evals depend on.

**Open questions**
- **Shipping:** whether `evals/`, `config/harnesses/` and `lib/harness-profiles/` should ship in the plugin packages. I'll decide that when I regenerate.
- **Missing appendix:** the old A-9 pointed at an "Appendix A" that never existed in mini. The new text points at `boundaries.md` instead.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 815e20bd-e2d6-4020-a972-f06fc10a8cac
- Captured: 2026-11-16T10:18:45.510063Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
