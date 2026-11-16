---
type: SessionRecord
id: karpathy-session-48c5a3ea45899832
title: Karpathy session 48c5a3ea4589
tags:
- karpathy
- session-learning
sources:
- resource: session:0ffe50ed-a878-4de2-b03a-ec3e58a38524
generated:
  by: pk/1.11.0
  at: 2026-11-15T23:25:47.548375+00:00
created_at: 2026-11-15T23:25:47.548375+00:00
updated_at: 2026-11-15T23:25:47.548375+00:00
revision: 0
---

## Delta

PR [Prometheus-AGS/prometheus-skill-system#224](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/224) is open: release 1.15.0, merges cleanly, CI isn't being used as evidence. It covers R0–R3 of the VLBV proposal, OpenSpec 2.0.0, Apple Foundation Models and the bootstrap alignment.

**What changed:**
- **OpenSpec 2.0.0** is pinned exactly and the pack never follows npm `latest` again. A newer major is reported as `migrationRequired`, not installed. No `/opsx:*` commands are emitted, and a new session only checks the OpenSpec files; it never rewrites them. This repo's half-finished migration was completed with backups reconciled, and running it twice changes nothing.
- **VLBV** is the default and only method. Boundaries are defined once (B0–B6), and generated code runs only at B4 (once) or after an authorized repair (B5). The compiler exception is gone from AGENTS.md and CLAUDE.md. The PR also adds a v2 adopted contract with a lint that rejects weak contracts, a closure packet, and delivery cadence tied to the same boundary.
- **One installed skill copy.** Skills install once to `~/.agents/skills`, with links for Claude Code and Qwen only. The real install on this Mac removed 2702 pack-owned entries from the 12 old per-tool folders; nothing else was touched. Codex now lists each pack skill once (70 catalog entries, healthy).
- **Apple Foundation Models** run on this Mac as a local service on port 11437, behind liter-llm, as the KBD `local` role. They answered direct and gateway requests, the reviewer roles refuse them, and on ineligible machines setup prints nothing.
- **prometheus-context-bootstrap** now generates AGENTS.md and CLAUDE.md that carry the VLBV policy block and pass its new checks.

**What failed, or wasn't run:**
- **Present on `main` too, so not caused by this PR:**
  - 102 of 158 `openspec validate` items, under 1.16 and 2.0 alike.
  - Two `kbd-phase-consistency` runtime tests.
  - The readiness-gate suite and three installer fixture tests.
  - The v4 rules-build drift check.
  - The format check (231 files already fail on `main`).
- **The cross-model review of the full diff** raised 14 critical findings. I fixed 10 as one repair under your "keep going until the PR is open", and rejected 4 with reasons in `docs/decisions/vlbv-single-method/review/diff-review.json`.
- **A Claude Code plugin shims `python3`** inside this session to "use `uv run python`". That broke the installer's Codex catalog step here; rerunning it with a real Python fixed it. It's an environment issue, not a pack bug.
- **Not run** (deferred to the R4 follow-up): per-harness protocol evals, fresh-session checks with hooks disabled, and the OpenCode and Claude `/skills` listings.
- **No canonical KBD phase.** KBD has `issue-195-closeout` active, so the decisions and review receipts are in `docs/decisions/vlbv-single-method/` instead.

The commits are split by area (generated OpenSpec files, `dist/`, source, repair) rather than by rollout stage, because the stages share files. The `flake-budget.sh` deletion ended up in the first commit.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 0ffe50ed-a878-4de2-b03a-ec3e58a38524
- Captured: 2026-11-15T23:25:34.060646Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
