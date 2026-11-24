## ADDED Requirements

### Requirement: Search the guides and render Mermaid

The docs site SHALL index its guide routes for local search and render Mermaid fences with the exactly matching Docusaurus theme version. The historical selection SHALL use Docusaurus and Mermaid theme 3.10.1 with local search 0.55.2.

#### Scenario: Look up a known guide diagram

- **WHEN** a reader searches for a known guide heading and opens the result
- **THEN** the local index returns the guide and its Mermaid fence renders as a diagram
