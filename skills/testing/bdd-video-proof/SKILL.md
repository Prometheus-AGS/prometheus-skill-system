---
name: bdd-video-proof
version: '2.0.0'
description: >
  Assemble a signed certification bundle from a cucumber test run — cucumber
  JSON report, MP4 videos (ffmpeg-remuxed from Playwright WebM), screenshot
  manifest, and a SHA-256 manifest keyed to the git commit. Two modes: local
  bundle under docs/certifications/<module>/<sha>/, or IPFS-pinned. Use
  when producing release evidence, reviewer-facing test proof, or
  audit trails.
license: MIT
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [bdd, cucumber, video, proof, certification, audit, evidence, ipfs]
---

# BDD Video Proof

Produces a **certification bundle** from a cucumber test run — a
self-contained folder (or IPFS pin) containing everything a reviewer needs
to trust that "these scenarios passed against this commit":

- `manifest.json` — SHA-256 of each artifact + git SHA + module fingerprint
- `cucumber-report.json` — raw cucumber output
- `videos/*.mp4` — one per scenario (WebM → MP4 via `ffmpeg -c copy`)
- `screenshots/**/*.png` — on-failure snapshots + explicitly requested captures
- `report.html` — human-readable index with embedded video paths

## When to invoke

- "Mint a certification bundle for the checkout module"
- "Produce release evidence for v3.2.0"
- "Record BDD video proof and pin to IPFS"
- "Generate audit trail for the auth flow"

## Two modes

### Mode A: Local bundle (default in 2.0)

Writes `docs/certifications/<module>/<sha>/` — a directory that ships
in-repo and is reviewed like any other file. Signing = git SHA +
SHA-256 hash of every artifact recorded in `manifest.json`.

No IPFS node required. Reviewers watch videos with any browser. Bundle
size is bounded by scenario count × video length; typical range is 5-50 MB
per module per commit.

### Mode B: IPFS-pinned (legacy 1.0 workflow, still supported)

Pins each MP4 to IPFS and writes only CIDs to `docs/videos-manifest.json`.
Useful when videos are too large to ship in-repo or immutability guarantees
are required beyond git.

## Quick start

### Mode A — local bundle

```bash
# Mint a bundle for the current commit against a specific module
bash "${CLAUDE_PLUGIN_ROOT}/skills/testing/bdd-video-proof/scripts/mint-certification-bundle.sh" \
    --module auth \
    --cucumber-json tests/reports/cucumber.json \
    --videos-dir tests/reports/videos \
    --screenshots-dir tests/reports/screenshots

# Dry-run: show plan without writing
bash mint-certification-bundle.sh --module auth --cucumber-json ... --dry-run
```

Output:
```
docs/certifications/auth/<git-sha>/
├── manifest.json
├── cucumber-report.json
├── videos/
│   ├── sign-in-happy-path.mp4
│   └── sign-in-invalid-password.mp4
├── screenshots/
│   └── ...
└── report.html
```

### Mode B — IPFS pinned (legacy 1.0)

```bash
npx ts-node scripts/run-video-proof.ts             # all scenarios
npx ts-node scripts/run-video-proof.ts --feature tests/features/ui/x.feature
npx ts-node scripts/run-video-proof.ts --dry-run
npx ts-node scripts/run-video-proof.ts --no-pin    # skip IPFS
```

## Manifest schema

`manifest.json` inside the bundle:

```json
{
  "schema_version": 1,
  "generated_at": "2026-07-09T14:00:00Z",
  "git_sha": "1009aed4b3...",
  "module": "auth",
  "module_fingerprint": "sha256:1a2b3c...",
  "artifacts": [
    {
      "path": "cucumber-report.json",
      "sha256": "e3b0c44...",
      "bytes": 12943
    },
    {
      "path": "videos/sign-in-happy-path.mp4",
      "sha256": "abc123...",
      "bytes": 843201,
      "scenario": "User signs in via the browser :: Happy path — valid credentials land on dashboard"
    }
  ],
  "runtime": {
    "cucumber_version": "13.0.0",
    "playwright_version": "1.48.0",
    "runner": "cucumber-js"
  }
}
```

**`module_fingerprint`** is SHA-256 of a sorted concatenation of the
module's source file hashes — reviewers can confirm the videos correspond
to a specific implementation state, not just a specific commit.

**`git_sha`** is `git rev-parse HEAD` at the time of minting.

## Video format

Playwright records **WebM (VP8)**. The mint script remuxes to MP4 via
`ffmpeg -c copy` (lossless stream copy — no re-encode):

```bash
ffmpeg -i input.webm -c copy output.mp4
```

Requires `ffmpeg` on `PATH`. Install:
- macOS: `brew install ffmpeg`
- Debian/Ubuntu: `apt install ffmpeg`

## Where bundles live

```
docs/certifications/
├── auth/
│   ├── 1009aed/          ← per-commit
│   │   ├── manifest.json
│   │   ├── report.html
│   │   ├── videos/
│   │   └── ...
│   └── b855dff/
├── checkout/
│   └── ...
└── INDEX.md              ← optional, generated
```

Reference from the module's `README.md`:

```markdown
Latest certification bundle: [auth @ 1009aed](../docs/certifications/auth/1009aed/report.html)
```

## Signing (roadmap)

v2.0 signs bundles with **git SHA + SHA-256**. Future versions may add:

- **GPG signature** on `manifest.json` for external verification
- **Sigstore keyless signing** via cosign for public artifacts

Both are deferred — the current bundle is auditable because every artifact
is hash-committed to the manifest, which is itself part of the git
commit tree.

## See also

- [bdd-cucumber-js](../bdd-cucumber-js/SKILL.md) — cucumber-js authoring
- [bdd-cucumber-rs](../bdd-cucumber-rs/SKILL.md) — cucumber-rs authoring
- [bdd-lifecycle-loop](../bdd-lifecycle-loop/SKILL.md) — the four-phase workflow
- [references/SETUP.md](references/SETUP.md) — installation and prereqs
- [references/IPFS.md](references/IPFS.md) — Mode B pinning workflow
- `docs/future-work/02-bdd-testing-evolution/BDD-004-video-skill-productization.md`
