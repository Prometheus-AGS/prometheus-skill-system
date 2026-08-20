---
type: SessionRecord
id: karpathy-session-b630101ee84ccd41
title: Karpathy session b630101ee84c
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-13T20:56:18.875832+00:00
created_at: 2026-08-13T20:56:18.875832+00:00
updated_at: 2026-08-13T20:56:18.875832+00:00
revision: 0
---

## Delta

Everything green and pushed to **`b47b2a8`**.

## Result

| Check | Result |
|---|---|
| **All skills, all tools** | **2282/2282** (163 skills × 14 targets) |
| Red/green gate suite | 13 passed, 0 failed |
| Generation seal | `--verify` OK |
| `deep-research` | Full 13,519-byte skill with all 8 subdirs, everywhere |

The April stub that blocked your Codex session is gone. `deep-research` resolves in Claude, Codex, and OpenCode.

## What the cleanup actually removed

**19 occupants**, not the 3 I reported. Sixteen were symlinks into a live source checkout — they resolved fine, which is why my audit called them healthy, but pointed at April/July content instead of the generation. `artifact-refiner` was serving four-month-old content across six targets.

All 19 archived reversibly at `~/.prometheus/skill-collisions-archive-20260813/` with a manifest and the original symlink targets recorded.

## The install can no longer lie

Three changes, each verified failing before I trusted it:

- **Collisions exit non-zero**, listing occupant, kind, and mtime. Proven by the 19 real ones.
- **`verify-skill-install.js`** enumerates from the generation — 163 × 14, denominator printed every run. It cannot be accidentally scoped to whatever directory I happen to be in, which is the exact mistake that produced my false "0 drift."
- **It runs as the install's last step**, so "install ran" and "install is correct" are now the same statement.

Six failure modes planted and observed failing: unowned dir, absent skill, symlink outside the generation, dangling link, stale copy file, missing copy file.

## Two of my own defects fixed along the way

**I had broken the generation's integrity seal.** My earlier `stampProvenance()` wrote two files *inside* the signed, content-addressed generation. `--verify` was failing. Moved beside it; now passes.

**The reminder text I wrote was wrong.** It printed the project-wide 47/72 under a line claiming "THIS phase's counter" — the phase was 0/6. Caught by running the binary, not by the passing unit test.

## Still outstanding

- **`~/.local/bin/prometheus` predates the label fix** — needs a rebuild to carry it.
- **The §D-6-gated rule change** — you chose "run the gates first," so the rule text goes through adversarial review and the sycophancy gate before it lands.
- **The postmortem record** — canonical entry in `.prometheus/postmortems/` with pointers, per your "one canonical + pointers" choice.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-13T20:50:45.719412Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
