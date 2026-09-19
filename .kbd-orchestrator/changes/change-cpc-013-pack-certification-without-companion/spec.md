# change-cpc-013-pack-certification-without-companion

**Title:** Certify the pack fully with the Companion absent and update pack docs  
**Repository:** `prometheus-skill-pack`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-009-pack-removal`, `change-cpc-010-kbd-state-migration`  
**Backend:** native-kbd

## Why

Goal 6 and the governing model require proof that the open-source pack is complete on its own. That proof is pack-side: an install profile with no Companion, every kbd-* skill path the local runtime covers, `prometheus doctor`, and the OpenSpec, harness, and receipt-identity certification, with no Companion mention in any output.

## What Changes

- Add `scripts/certify-without-companion.sh` that runs the install profile without the Companion, the kbd-* skill paths, `prometheus doctor --json`, and the existing certification gates, asserting `grep -ci 'companion\|sovereign' certification.log` is 0.
- Update pack docs: `site/docs/operations/installation-and-upgrades.md`, `docs/guide/19-installation.md`, `docs/guide/20-updating.md`, and release notes.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `CHANGELOG.md`
- `docs/guide/19-installation.md`
- `docs/guide/20-updating.md`
- `scripts/certify-without-companion.sh`
- `site/docs/operations/installation-and-upgrades.md`

## Capabilities

- `control-plane-evidence` (existing; added by change-cpc-011)

## ADDED Requirements

### Requirement: Pack certifies with the Companion absent
WHEN the certification script runs on a machine with no Companion, THEN every gate passes and no output mentions the Companion or sovereign-sync.

#### Scenario: Absent
- **WHEN** the script runs
- **THEN** exit 0 and `grep -ci 'companion\|sovereign' certification.log` is 0

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Constraints C-01..C-05 apply; `npm run validate:codex` and `docs/codex-plugin.md` in the same change when plugin surfaces or install flow move; `shared/services.manifest.json` regenerated in the same change when a plist or unit changes.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).
