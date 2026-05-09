---
id: BDD-003
title: IPFS pin sweep job
status: ready
priority: P2
estimated_effort: 1d
agent_role: bdd-engineer
depends_on: []
unblocks: []
related: [BDD-001]
created_from_conversation_turn: 5-6
---

# BDD-003 — IPFS pin sweep job

## Problem

Each video run produces ~1MB-5MB per scenario × 250 scenarios = 250MB-1.2GB pinned to IPFS per full re-record. After multiple runs the pin set grows unbounded. IPFS dedup helps when content is genuinely unchanged, but Playwright video output is rarely byte-identical across runs.

The `videos:upload` script's "unchanged" status returns the existing CID when content matches — good. But CIDs that *were* pinned in earlier runs and are no longer referenced by *any* current manifest entry are orphans. They keep accumulating pin storage cost.

## Evidence

1. Inspect the IPFS pinning service usage trend over the last 30-90 days. Storage should be growing.
2. Inspect the historical manifest snapshots if any exist. CIDs that appeared in older versions but not current are orphans.

## Why it matters

Direct cost: pinning service fees grow. Indirect cost: nobody knows which CIDs are needed and which are not.

Not P0 because the cost is bounded by IPFS storage pricing and growth rate. Worth doing before the bill becomes annoying.

## Proposed fix

A scheduled `scripts/sweep-ipfs-pins.ts` script:

1. Reads the current `docs/videos-manifest.json` and gathers all referenced CIDs.
2. Reads the IPFS gateway's pin list (`ipfs pin ls` or pinning-service API equivalent).
3. Computes the difference: CIDs pinned but not in the current manifest.
4. For each orphan, **dry-run by default** prints the CID and a guess at when it was pinned.
5. With `--apply`, unpins each orphan.

Schedule it weekly (or run manually quarterly). Log each sweep to `tests/reports/ipfs-sweep-<date>.md`.

**Safety net:** before unpinning, optionally archive the CID metadata to a quarterly archive file so you can re-pin if the sweep was wrong.

## Trade-offs and risks

- **Risk: unpinning a CID still in use somewhere outside the manifest.** Mitigation: keep an allowlist for CIDs intentionally pinned outside the test pipeline (e.g. shared documentation videos). Sweep ignores allowlist entries.
- **Risk: dry-run output is overwhelming on first run.** Bound the unpin per sweep (e.g. max 100 unpins per run). Iterate over multiple runs to drain the backlog.
- **Cost: pin-list query on a large pin set is slow.** Acceptable for a weekly job.

## Acceptance criteria

- [ ] `scripts/sweep-ipfs-pins.ts` runs in dry-run by default.
- [ ] Reports orphan CIDs with size and pinned-at metadata.
- [ ] `--apply` unpins orphans.
- [ ] Allowlist mechanism for intentionally-orphaned-but-needed CIDs.
- [ ] Per-sweep log archive in `tests/reports/`.
- [ ] CI workflow runs the sweep weekly.

## Implementation steps

1. Identify the IPFS gateway endpoint and authentication.
2. Write the manifest read + pin-list fetch.
3. Compute the diff.
4. Implement dry-run output and `--apply` mode.
5. Add the allowlist file `scripts/ipfs-pin-allowlist.json`.
6. Schedule via GitHub Actions cron weekly.

## Dependencies

None functional. Recommended after BDD-001 so the manifest is clean before sweep.

## Open questions

- What's the right cadence for this — weekly or monthly? Start weekly; tune to monthly if the sweep finds little.
- Are there CIDs pinned by other systems in the same gateway? If yes, allowlist must be inclusive.
