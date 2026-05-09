# IPFS Pinning Workflow

## Why IPFS?

Each scenario video is content-addressed. The CID (Content Identifier) is a cryptographic hash of the video content. Once pinned, the evidence is:

- Immutable — the CID changes if the file changes
- Auditable — any party can verify the recording matches the CID
- Distributed — pinned copies survive node failure

## Pin lifecycle

```
run-video-proof.ts
  │
  ├─ records scenario → tests/videos/<slug>.mp4
  ├─ adds file to IPFS node: ipfs add --pin <file>
  ├─ writes CID to docs/videos-manifest.json
  └─ prints IPFS gateway URL: https://ipfs.io/ipfs/<CID>
```

## Manifest format

```json
{
  "generated_at": "2026-05-09T18:00:00Z",
  "videos": [
    {
      "scenario": "Acquisition buyers management route:shows buyer list",
      "feature_file": "tests/features/ui/acq-buyers-route.feature",
      "cid": "QmXxx...",
      "gateway_url": "https://ipfs.io/ipfs/QmXxx...",
      "recorded_at": "2026-05-09T17:45:00Z",
      "duration_ms": 4200
    }
  ]
}
```

## Sweeping orphaned pins

Use `scripts/ipfs-pin-sweep.ts` (BDD-003) to remove CIDs from the IPFS node
that are no longer referenced in `docs/videos-manifest.json`:

```bash
npx ts-node scripts/ipfs-pin-sweep.ts --dry-run
npx ts-node scripts/ipfs-pin-sweep.ts --execute
```
