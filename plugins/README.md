# Adjacent plugins

This directory contains complete, independently versioned plugins that the
Prometheus marketplace distributes beside the umbrella skill pack. Their
skills stay in their native package and are not flattened into
`prometheus-skill-pack`.

## Hybrid Mobile Architecture / KnowMe Builder

- Repository: <https://github.com/Know-Me-Tools/hybrid-mobile-architecture-skill>
- Package: `hybrid-mobile-architecture`
- Version: `2.0.0-alpha.4`
- Distribution: native Claude and Codex payloads built by the package's own
  portable stager, pinned by exact Git commit in `skill-system.json`

The current pin is the release candidate used to integrate portable tooling
and project evolution. Advance the gitlink and the matching `commit` in
`skill-system.json` together after that candidate merges. The distribution
gate rejects any mismatch.
