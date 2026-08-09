# BDD Video Proof — Setup

## Prerequisites

- Node.js ≥ 18
- `pnpm` or `npm`
- IPFS daemon (Kubo) running locally on port 5001 (`ipfs daemon`)
- Playwright installed (`pnpm playwright install`)
- Target project has `scripts/run-video-proof.ts`

## Installing the script

If the target project does not have `scripts/run-video-proof.ts`, copy it from:

```
${CLAUDE_PLUGIN_ROOT}/skills/testing/bdd-video-proof/scripts/run-video-proof.ts
```

Then add to `package.json`:

```json
{
  "scripts": {
    "video-proof": "ts-node scripts/run-video-proof.ts",
    "video-proof:dry": "ts-node scripts/run-video-proof.ts --dry-run"
  }
}
```

## Dependencies

```bash
pnpm add -D ts-node typescript @types/node playwright
pnpm add -D kubo-rpc-client  # IPFS HTTP client
```

## Environment Variables

| Variable | Required | Description |
| -------- | -------- | ----------- |
| `IPFS_API_URL` | No | IPFS API endpoint (default: http://127.0.0.1:5001) |
| `VIDEO_OUTPUT_DIR` | No | Where to write MP4 files (default: tests/videos/) |
| `VIDEO_TIMEOUT_MS` | No | Per-scenario timeout in ms (default: 30000) |
| `BASE_URL` | Yes | Application URL to record against |
