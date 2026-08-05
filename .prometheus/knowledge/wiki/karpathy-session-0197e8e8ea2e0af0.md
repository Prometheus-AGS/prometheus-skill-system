---
type: SessionRecord
id: karpathy-session-0197e8e8ea2e0af0
title: Karpathy session 0197e8e8ea2e
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T00:58:56.886491+00:00
created_at: 2026-08-05T00:58:56.886491+00:00
updated_at: 2026-08-05T00:58:56.886491+00:00
revision: 0
---

## Delta

Committed as `858e6e2`. Done.

## The fix

`install-skills-flat.sh` had two bugs at its three substrate copy sites — and notably, **`install-binaries.sh` already did this correctly**, so the fix was making the two installers consistent rather than inventing an approach.

**1. Missing re-sign after copy.** `cp` invalidates an ad-hoc/linker-signed arm64 binary's signature; macOS then SIGKILLs anything that execs it. All three copy sites now route through one `install_substrate_bin` helper mirroring `install_bin`.

**2. `cmp` false negative.** The OS was killing `cmp` itself while reading the freshly-copied file. Swapped for `shasum -a 256`.

One ordering detail I had to get right: **verify before signing.** Signing mutates the file, so hashing afterward would always mismatch — the trap my earlier manual re-sign attempt fell into. The helper's comment records why the sequence can't be rearranged.

## Verification

I proved the fix rather than assuming it. First I deliberately re-broke both signatures (`sovereign-sync --version` → exit 137), then ran with the **strict** failure policy — the same invocation that failed three times earlier:

```
EXIT=0
✅ learner-model:  installed artifact matches the local release build
✅ surface-bridge: installed artifact matches the local release build
✅ sovereign-sync: installed artifact matches the local release build
```

All three execute (exits 0, 101, 2 — no 137), `codesign -v` passes on each, and `surface-bridge` still serves HTTP 200. Syntax checked under bash 5.3 and `/bin/bash` 3.2, since CLAUDE.md requires launchd-invoked scripts stay 3.2-compatible.

## Two things to flag

**The stale `.git/index.lock` recurred.** Same signature as before: 0 bytes, orphaned at 12:25 — during the install runs. I verified no live git process before removing it. Twice in one session, both times during installer activity, points at a hook crashing mid-operation. Worth tracking down; it will keep blocking commits.

**This fix isn't the whole class.** I fixed the three substrate copies in `install-skills-flat.sh`. I did **not** audit other scripts for the same `cp`-without-`codesign` pattern — `install-mcp-services.sh` and any others could carry it. Happy to sweep for that.

The commit is local only. Want me to push?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T00:58:08.109429Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
