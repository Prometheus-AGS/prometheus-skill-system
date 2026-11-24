## ADDED Requirements

### Requirement: Honor the Pages deployment base path

Docusaurus SHALL produce assets and links under the configured site origin and base path, default to the Prometheus-AGS/prometheus-skill-system GitHub Pages location, point edit links at the same repository, and fail the build for broken site or Markdown links.

#### Scenario: Build for a repository subpath

- **WHEN** a maintainer builds with SITE_URL and BASE_URL configured for a repository-scoped deployment
- **THEN** HTML asset URLs carry the configured prefix, and broken internal links cause a nonzero build result
