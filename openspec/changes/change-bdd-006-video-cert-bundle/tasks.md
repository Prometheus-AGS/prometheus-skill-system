# Tasks — change-bdd-006-video-cert-bundle

- [x] Update skills/testing/bdd-video-proof/SKILL.md to describe both IPFS and local bundle modes
- [x] Write scripts/mint-certification-bundle.sh (assembles docs/certifications/<module>/<sha>/ layout)
- [x] Include ffmpeg -c copy step to remux Playwright WebM → MP4 losslessly
- [x] Generate manifest.json with SHA-256 of each artifact + git SHA + module fingerprint
- [x] Generate report.html index (human-readable, embeds video paths + screenshot thumbnails)
- [x] Document IPFS pinning as an optional post-step in references/IPFS.md
- [x] Commit the change
