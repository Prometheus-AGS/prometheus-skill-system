# Proposal — change-bdd-006-video-cert-bundle

Extend `skills/testing/bdd-video-proof/` with the local certification
bundle format: `docs/certifications/<module>/<sha>/` layout containing
`manifest.json` (SHA-256 per artifact + git SHA + module fingerprint),
`cucumber-report.json`, `videos/*.mp4` (ffmpeg-remuxed from Playwright
WebM), `screenshots/**/*.png`, and `report.html`. IPFS pinning stays
optional. Signing is git SHA + SHA-256 for this phase; GPG/Sigstore
deferred.

## Library candidates

- **cand-008**: `ffmpeg` (system) — adopt for MP4 remux via lossless stream copy

## Goal
G-04 — Video-proof certification skill (local bundle).
