## ADDED Requirements

### Requirement: Render the eight research components

HTTP and MCP rendering SHALL use the same component registry for graph_view, source_list, contradiction_panel, progress_ring, media_card, stage_timeline, markdown_viewer, and citation_list. Every fragment SHALL have its matching ID and support out-of-band swapping. The service SHALL serve the bundled HTMX runtime and extensions and Alpine.js.

#### Scenario: Render Markdown and progress components

- **WHEN** a client requests markdown_viewer and progress_ring using HTTP or MCP
- **THEN** the registry returns sanitized rendered Markdown or an SVG progress indicator with the expected swap target
