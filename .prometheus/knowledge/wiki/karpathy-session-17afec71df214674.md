---
type: SessionRecord
id: karpathy-session-17afec71df214674
title: Karpathy session 17afec71df21
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T14:05:44.863604+00:00
created_at: 2026-11-17T14:05:44.863604+00:00
updated_at: 2026-11-17T14:05:44.863604+00:00
revision: 0
---

## Delta

Domain selection is implemented in the pack installer and works end to end in a scratch home. Nothing is committed, and I did not run it against your real install.

## What you can now do

```bash
node scripts/install-plugin-generation.js --list-domains
```

```bash
bash scripts/update-skill-pack.sh --domains=process,rust
```

`--add-domains=`, `--remove-domains=` and `--domains=all` work the same way, on both `update-skill-pack.sh` and `install-skills-flat.sh`. An install with no domain flag keeps the stored selection. With no stored selection, everything is installed as before.

## Verification

The new integration test drives the real installer from the packaged payload and passes all seven scenarios (5m23s):

- install everything;
- select one domain, with a hook still dispatching;
- add a domain and get the domains it requires;
- remove a domain, with a foreign skill left untouched;
- reinstall with no flag and keep the selection;
- kill an install mid-activation and recover to the journaled selection;
- roll back to a generation without a domain index and forward again.

Fourteen existing checks pass, including payload activation, interrupted-activation recovery, hook dispatch, the distribution tests and strict skill validation.

Two existing tests fail:

- **`test-plugin-generation.sh`**: its fixture lacks `scripts/check-plugin-source.js`, which `generate-harness-adapters.js` reads. I did not change either file.
- **`marketplace-bootstrap.test.sh`**: it expects 14 target receipts, the count from the old 14-target matrix; the installer now has 3 targets.

I have not run either on the base branch, so "already failing before this change" is my reading of the failure messages, not something I confirmed.

## What differs from the design

- **The domain map is a generated file, `shared/skill-domains.json`.** The installer usually runs from the packaged payload, where the plugin manifests no longer exist, so the map cannot be built at install time. `npm run generate:skill-domains` builds it, `npm run check:skill-domains` is the drift gate, and an install fails on a skill with no domain.
- **There are 21 domains**, declared in a new `domains` list in `skill-system.json`. One-skill plugins are folded into a neighbour, for example `disk-space-guardian` into `devops`.
- **Process is the largest domain** at 52 skills and about 17,000 listing characters. It requires `iterative-evolver`, `testing` and `sycophancy-correction`, which brings a process-only install to about 22,600 characters. Learn requires `sycophancy-correction`.

## Not built

- The `cowork` CLI has no flag; set `PROMETHEUS_SKILL_DOMAINS` instead, which the installer reads.
- `--verify` does not yet report references to skills in unselected domains.
- There is no doctor entry. The duplicate-listing warning comes from the installer and from `--list-domains`.
- I did not rerun `claude plugin eval` against a narrow selection, so the claim that triggering returns still rests on the earlier domain-plugin measurements.
- I did not check whether the later steps of `install-skills-flat.sh` (Codex catalogue, slash-command registration) cope with a narrow selection.

## Before you run it on this machine

The first install with this code removes pack-owned entries that the active generation no longer includes, even with no selection. Your `~/.claude/skills` has about 25 more pack links than the generation has skills, so expect those to go. They are listed afterwards in `receipts/unselected-cleanup.json` in the plugin store.

The work is on branch `feat/installer-domain-selection` in the scratch worktree, stacked on PR #231. The scratch directory does not survive a reboot, so it needs committing to be safe.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T14:05:26.229946Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
