---
name: bdd-video-proof
version: '1.0.0'
description: Record video evidence for Cucumber BDD scenarios. Captures MP4 proof of each passing test scenario and pins it to IPFS for immutable audit trails. Triggers on "run video proof", "record BDD evidence", or "capture test video".
license: MIT
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [bdd, cucumber, video, proof, ipfs, testing, evidence]
---

# BDD Video Proof

Records MP4 video evidence for Cucumber scenario runs and pins each recording to IPFS for an immutable, auditable test trail.

## When to invoke

- "Run video proof for the acquisition workflow"
- "Record BDD evidence before the release"
- "Capture test video for the buyer management scenarios"
- "Generate video proof of all passing scenarios"

## Quick Start

```bash
# Run all scenarios with video capture
npx ts-node scripts/run-video-proof.ts

# Run specific feature file with video capture
npx ts-node scripts/run-video-proof.ts --feature tests/features/ui/acq-buyers-route.feature

# Dry run — show what would be recorded without capturing
npx ts-node scripts/run-video-proof.ts --dry-run

# Skip IPFS pinning (CI environments without IPFS node)
npx ts-node scripts/run-video-proof.ts --no-pin
```

## How to invoke

1. Confirm the target project has `scripts/run-video-proof.ts` present (see [Setup](references/SETUP.md))
2. Check the IPFS node is reachable (or pass `--no-pin` to skip pinning)
3. Run the capture command for the target scope (all, feature file, or tag filter)
4. Review the output manifest at `docs/videos-manifest.json`
5. Verify IPFS CIDs are pinned: `ipfs pin ls --type recursive`
6. Report video CIDs alongside test results

## Output

| Artifact | Path |
| -------- | ---- |
| Video files | `tests/videos/<scenario-slug>.mp4` |
| IPFS CIDs | `docs/videos-manifest.json` |
| Run summary | `tests/reports/video-proof-summary.json` |

## Flags

| Flag | Default | Description |
| ---- | ------- | ----------- |
| `--feature <path>` | all | Limit capture to a single .feature file |
| `--tag <tag>` | all | Filter scenarios by Cucumber tag |
| `--dry-run` | false | Show capture plan without running |
| `--no-pin` | false | Skip IPFS pinning step |
| `--timeout <ms>` | 30000 | Per-scenario timeout |

## Detailed Documentation

- [Setup and dependencies](references/SETUP.md)
- [IPFS pinning workflow](references/IPFS.md)
- [CI integration](references/CI.md)
