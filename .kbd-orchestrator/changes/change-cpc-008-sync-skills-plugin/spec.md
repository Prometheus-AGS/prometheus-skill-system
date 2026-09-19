# change-cpc-008-sync-skills-plugin

**Title:** Ship the three sync-* skills as the Companion's skill plugin, the first connected-skill-package  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-001-integration-contract`, `change-cpc-004-relocate-sovereign-sync`  
**Backend:** native-kbd

## Why

sync-status, sync-peers, and sync-push are the sharing feature; without the Companion they are non-functional, which violates the open-core rule. They ship from the Companion as its own skill plugin declared through `skill-package.json`, and the Companion owns the `sovereign-sync --mode mcp` registration (D-02, D-12). Pack-side removal is change-cpc-012.

## What Changes

- Create the Companion `skills/` plugin with the three skills, rewritten to the Companion socket and MCP binary; add `skill-package.json` per the contract.
- Companion installer registers its MCP stdio binary and skills into the harnesses via the declaration, validated with the pack's `prometheus contract validate`.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `scripts/install-skill-package.sh`
- `skill-package.json`
- `skills/sync-peers/SKILL.md`
- `skills/sync-push/SKILL.md`
- `skills/sync-status/SKILL.md`

## Capabilities

- `companion-skill-plugin` (new)

## ADDED Requirements

### Requirement: Companion skill package validates and installs through the contract
WHEN the Companion's `skill-package.json` is validated with `prometheus contract validate`, THEN it passes; WHEN the Companion installer runs, THEN the three skills and the MCP registration appear in each harness.

#### Scenario: Validation
- **WHEN** validate runs
- **THEN** exit 0

#### Scenario: Install
- **WHEN** the installer runs on a machine with the pack
- **THEN** `/sync-status` is invokable and the installer's registration output (its `--dry-run` plan and, on Claude Code, the `~/.claude/mcp-servers.json` entry it writes and owns) names the Companion MCP binary

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- New dependencies are a human pin: any crate this change introduces is staged in `docs/decisions/versions-pins-proposal.md` (change-cpc-003) and the change stays BLOCKED until the operator's `versions.toml` edit includes it.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).
