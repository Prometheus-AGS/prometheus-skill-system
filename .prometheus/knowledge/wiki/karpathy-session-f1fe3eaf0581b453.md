---
type: SessionRecord
id: karpathy-session-f1fe3eaf0581b453
title: Karpathy session f1fe3eaf0581
tags:
- karpathy
- session-learning
sources:
- session:9db42325-24eb-4238-98b2-20ac2824b479
timestamp: 2026-08-20T11:40:02.227987+00:00
created_at: 2026-08-20T11:40:02.227987+00:00
updated_at: 2026-08-20T11:40:02.227987+00:00
revision: 0
---

## Delta

Done. Cleaning up the stale generations turned into finding and fixing three real bugs — the disk was the symptom, not the problem.

## What I found

My starting hypothesis was wrong, and worth stating plainly: **the accumulated generations were not causing the Tier W failure.** A fresh daemon on a pruned plugin root reproduced it exactly. They were unrelated problems that looked like one.

**1. The prune could never delete anything.** `--prune-obsolete` ran full `verifyGeneration()` on every generation *before* checking whether it was even a candidate. So it aborted on unsigned generations — which predate signing and are by construction the oldest, i.e. exactly its targets — and on 12 current-version generations predating the `executionComponent` field that it would have skipped a line later anyway. Fixed to verify identity only; that proves the manifest describes its directory unedited, which is what deletion actually requires. Every safety gate (not current, not previous, not referenced, receipts signature-verified) is unchanged.

**2. Tier W was rejecting correct packaging as tampering** — the real cause of `doctor` exiting 1. The distribution generator ships `entity-graph-optimize/skill.wasm` at **three byte-identical paths**, and the check rejected any repeated `.wasm` digest. The evidence is clean: `dist/`-built generations carry 3 entries with 1 digest and fail; older source-tree ones carry 5 with 5 and pass.

**3. Abandoned staging directories were never reaped** — one dead since Aug 5. Now reaped only when the owning PID is gone; a live one is a concurrent install and is preserved.

## One thing worth flagging

My first attempt at the Tier W fix **introduced a worse bug** — it skipped verification for repeat paths, so tampering with the second of three copies went undetected. The regression test I'd written caught it. The shipped fix deduplicates the *count* but hashes *every* packaged path.

## Results

- **27 → 25 generations, 2.1 GB → 1.7 GB**; `current` and `previous` intact
- `/ready` now **`true`** with all 7 subsystems green — **Tier W had been down since Aug 14**
- `prometheus doctor` **exits 0** (was 1)
- 28/28 `exec-tier-w` tests with `--features estate`, 26/26 `prometheus-exec`, clippy and fmt clean
- All five gates pass; 13 commits pushed

The generalizable lesson, recorded in the audit as §18.7: a hygiene tool that refuses to run on the mess it exists to clean is worse than none, because its silence reads as "nothing to do." Both failures came from applying an *activation*-strength check to a *deletion* decision.

**Left for you:** 7 uncommitted files (`skills/imported/README.md`, the Feynman spec addendum, wiki entries). Those are the learning worker's autonomous writes at 06:14 reconciling the Flutter-vs-Tauri mobile decision — not mine, and outside this task, so I didn't commit them.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 9db42325-24eb-4238-98b2-20ac2824b479
- Captured: 2026-08-20T11:39:54.229875Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
