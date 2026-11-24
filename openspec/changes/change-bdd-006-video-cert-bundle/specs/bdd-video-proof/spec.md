## ADDED Requirements

### Requirement: Local video certification bundle

The video-proof skill SHALL assemble `docs/certifications/<module>/<sha>/` with a digest manifest, Cucumber report, MP4 recordings, screenshots, and an HTML index. IPFS publication SHALL remain optional.

#### Scenario: Mint local proof from a recorded scenario

- **GIVEN** a run produced Playwright WebM video, screenshots, and a Cucumber report
- **WHEN** the author mints a local bundle
- **THEN** the video is remuxed without re-encoding and the manifest records each file hash, Git SHA, and module fingerprint; the HTML index links the evidence
