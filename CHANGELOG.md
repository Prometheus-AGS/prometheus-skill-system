# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- **Git submodule support** for imported skills from external repositories
- First imported skill: `artifact-refiner` (v1.1.0) from git@github.com:GQAdonis/artifact-refiner-skill.git
- Comprehensive submodule management documentation:
  - `docs/SUBMODULES.md` - Complete guide to managing git submodules
  - `docs/IMPORTING_SKILLS.md` - Step-by-step import process
  - `skills/imported/README.md` - Imported skills registry and operations
- `.gitattributes` for consistent line endings and file handling
- AgentSkills.io compliant directory organization
- Claude Code plugin support with marketplace distribution
- Validation tooling (`scripts/validate-skills.js`)
- Build tooling (`scripts/build-marketplace.js`)
- Installation scripts for user and project scopes
- Comprehensive documentation:
  - CLAUDE.md for AI assistant guidance
  - README.md for users and contributors
  - CONTRIBUTING.md for development guidelines
  - SKILL_TEMPLATE.md for creating new skills
- Example skill demonstrating best practices
- Shared resources structure (`shared/scripts`, `shared/templates`, `shared/utils`)
- Category directories for organizing skills:
  - `skills/react/` - React and frontend skills
  - `skills/rust/` - Rust programming skills
  - `skills/ui-ux/` - UI/UX design skills
  - `skills/devops/` - DevOps workflow skills
  - `skills/testing/` - Testing methodology skills
  - `skills/documentation/` - Documentation skills
- Dual-format support (agentskills.io + Claude Code plugin)
- Marketplace configuration for distribution
- MIT License

## [1.0.0] - TBD

### Added
- Initial release with core skill collections

[Unreleased]: https://github.com/gqadonis/prometheus-skill-pack/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/gqadonis/prometheus-skill-pack/releases/tag/v1.0.0
