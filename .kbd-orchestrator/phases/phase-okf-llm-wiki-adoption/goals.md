# Goals

- pk wiki entries conform to OKF v0.1: required 'type' frontmatter plus recommended title/description/resource/tags/timestamp, with unknown-key tolerance
- Reserved index.md and log.md maintained at wiki root per OKF sections 6-7, updated on every ingest
- Cross-links moved from frontmatter 'links' array to bundle-relative markdown body links, with Citations section convention per OKF sections 5 and 8
- Karpathy LLM Wiki operations (ingest, query, lint) exposed as first-class skills in this repo with a wiki schema document
- pk lint enforces OKF v0.1 conformance with permissive consumption semantics
