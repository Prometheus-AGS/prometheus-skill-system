# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-04-15

### Added

**Skills (61 total, 0 errors, 0 warnings)**

- React entity management suite (27 skills): setup, CRUD, GraphQL, Prisma, realtime, optimization
- Process orchestration (20 skills): KBD orchestrator, iterative evolver, PMPO skill creator
- GitOps CI/CD (4 skills): bootstrap, transform, ArgoCD multi-cloud, Kustomize overlays (TJ-CICD-001)
- BDD testing (1 skill): Cucumber.js + Playwright with video recording
- Imported: artifact-refiner (9 skills) via git submodule

**Nested PMPO Pipeline**

- Iterative-evolver outer loop delegates to KBD inner loop for software domain execution
- KBD auto-detects OpenSpec for structured change management
- Artifact-refiner QA gate per completed change
- Evolver bridge file (`evolver-bridge.json`) maps plan items to KBD changes
- KBD reflect reports back to evolver with artifact quality metrics

**Rust CLI (tools/prometheus-cli/)**

- 4-crate workspace: prometheus-cli, prometheus-agents, prometheus-learn, prometheus-cedar
- 15 subcommands: install, uninstall, list, search, audit, verify, doctor, status, generate, validate, build, memory, evolve, learn, optimize
- 10-platform adapter library (Claude Code, OpenCode, Cursor, Codex, Gemini CLI, Roo Code, Windsurf, Amp, Cline, Kilo Code)
- Cross-platform `TraceCapture` protocol for self-learning pipeline
- Cedar Skill Mutation PEP: gates skill.mutate/generate/promote/trace.capture
- Self-learning pipeline: trace capture, evaluation, knowledge compilation scaffolding, dspy-rs optimization scaffolding

**Surreal-Memory Integration**

- Root `.mcp.json` with surreal-memory, tavily, sequential-thinking servers
- Entity mapping patterns for all skill domains (evolver, KBD, GitOps, artifact-refiner)
- Comprehensive integration reference (`shared/references/surreal-memory-integration.md`)
- Graceful degradation when surreal-memory unavailable

**OpenCode Support**

- 3 TypeScript tool definitions (`.opencode/tools/`): evolve, kbd, gitops
- `.opencode/package.json` for auto-dependency installation
- Compatibility declared for 8 platforms in plugin.json

**Distribution**

- Marketplace with 5 granular plugin entries (full pack + domain-specific)
- TypeScript multi-platform installer (`scripts/install-platforms.ts`)
- GitHub Actions CI: validate skills, check formatting, cargo check + clippy
- Skills.toml/Skills.lock format (cowork-compatible)

**Governance & Architecture**

- Cedar default policies: development (permit all), staging (require validation), production (deny mutations)
- Self-learning architecture reference (`shared/references/self-learning-architecture.md`)
- UAR-embeddable library design (prometheus-learn as library, CLI as thin wrapper)
- Unified hooks.json with 5 events: SessionStart, PreToolUse, PostToolUse, SubagentStop, Stop

### Changed

- Recursive skill validator now scans sub-skills at any nesting depth
- Validator excludes backslashes inside code blocks from path separator warnings
- Validator adds line count checks (warning at 500, error at 800)

### Fixed

- Missing SKILL.md in prometheus-entity-skills container directory
- Duplicate pmpo-skill-creator in skills/creation/ (removed, kept skills/process/)
- Backslash path separators in bdd-testing, entity-crud-table, entity-realtime-channel
- pmpo-skill-creator sub-skill directory names mismatched frontmatter (clone→clone-skill, etc.)
- Empty skill categories (rust, ui-ux, devops, documentation) removed from manifests
- Plugin.json upgraded to full schema (author object, mcpServers, hooks path)

[1.1.0]: https://github.com/Prometheus-AGS/prometheus-skill-system/releases/tag/v1.1.0
