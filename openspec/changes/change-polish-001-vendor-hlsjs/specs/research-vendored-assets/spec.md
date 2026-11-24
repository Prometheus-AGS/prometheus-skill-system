## ADDED Requirements

### Requirement: Serve a functional bundled HLS player

The research service SHALL vendor the minified HLS.js v1 runtime in its static assets and serve the live browser export used by the media card.

#### Scenario: Open a supported HLS media result

- **WHEN** a browser that supports HLS.js loads the served static asset
- **THEN** the asset exposes a working Hls.isSupported and the media card can initialize the player
