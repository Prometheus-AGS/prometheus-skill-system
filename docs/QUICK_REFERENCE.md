# Quick reference

Use the canonical guide for commands with current production contracts:

| Purpose | Reference |
| --- | --- |
| Install skills or native services | [Installation](guide/19-installation.md) |
| Update an owned generation | [Updating](guide/20-updating.md) |
| Select a native tool or MCP | [Tools reference](guide/13-tools-reference.md) |
| Author a skill | [Contributor workflow](../CONTRIBUTING.md) and [template](SKILL_TEMPLATE.md) |
| Track tasks, ownership and handoffs | [Agent teams](guide/24-agent-teams.md) |
| Diagnose independently configured services | [Service operations](guide/26-service-operations.md) |
| Resolve generated output ownership | [Generated outputs](generated-output-ownership.md) |

`skills/` contains authored skill payloads; `shared/` holds collaborating runtime helpers; `scripts/` contains installers, generators and diagnostics. Generated marketplace listings and `dist/` payloads are derived artifacts, not places to repair source.

Complete all planned phase production before authoring or running tests, validators, formatters, generation or review. At the final local boundary, strict skill validation and real integration evidence have distinct purposes. A catalog, a unit result or a helper-only invocation does not certify installed behavior. Push only after applicable local gates pass; merging and publication retain their own authorization and evidence boundaries.
