## Why

KBD planning currently recommends an agent per change, leaving individual tasks without a concrete model or usable dispatch route. The operator requires quality-first task recommendations across native harnesses and liter-llm-backed workers, with matching full/mini releases.

## What Changes

- Analyze every task and record a scoped model assignment with evidence, harness capability, execution route, and unresolved prerequisites.
- Preserve assignments through kbd-execute and kbd-apply without changing task identity or runtime schemas.
- Document Codex, Claude Code, OpenCode, DeepSeek Harness, and Kimi Code controls; distinguish inference from tool-enabled execution.
- Release mini 0.2.0 alongside full 1.11.0; redistribute only the full pack on this machine.

## Capabilities

### New Capabilities

- `kbd-task-model-planning`: Task-level model selection and execution handoff.

### Modified Capabilities

None.

## Impact

Skill instructions, bundled references, OpenSpec artifacts, version manifests, and generated plugin packages. No new agent runtime, daemon, or inference-provider configuration. The OpenSpec dependency alone advances to the current stable release. Existing plans remain supported.

## Scope addition: latest OpenSpec lifecycle

Operator request on 2026-10-03: standardize both packs on the latest stable upstream OpenSpec, verified as 1.14.0 through the official npm registry and upstream release, and automatically refresh existing projects at startup and phase entry. This includes a portable managed CLI runner and generated-artifact backups/receipts. No new service, worker runtime, or edits to versions.toml.
