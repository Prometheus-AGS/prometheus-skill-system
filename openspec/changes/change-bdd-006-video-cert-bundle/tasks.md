# Tasks — change-bdd-006-video-cert-bundle

- [ ] Update skills/testing/bdd-video-proof/SKILL.md to describe both IPFS and local bundle modes
- [ ] Write scripts/mint-certification-bundle.sh (assembles docs/certifications/<module>/<sha>/ layout)
- [ ] Include ffmpeg -c copy step to remux Playwright WebM → MP4 losslessly
- [ ] Generate manifest.json with SHA-256 of each artifact + git SHA + module fingerprint
- [ ] Generate report.html index (human-readable, embeds video paths + screenshot thumbnails)
- [ ] Document IPFS pinning as an optional post-step in references/IPFS.md
- [ ] Commit the change
