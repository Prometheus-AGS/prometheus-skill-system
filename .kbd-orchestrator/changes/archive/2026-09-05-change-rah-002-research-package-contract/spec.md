# change-rah-002-research-package-contract

**Title:** Make research-package-spec.md the single normative contract with a JSON Schema, one output root, and a drift check
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G1
**Depends on:** `change-rah-001-compile-baseline-and-timestamps`
**Backend:** native-kbd

## Why

Assessment found three incompatible manifest shapes (spec int, template array, export script neither), two output roots (`~/.research-jobs/` in the daemon and four hooks, `~/.prometheus/research/` in scripts), `plan.json` expected by the synthesizer while stage-01 writes `plan.md`, a dead `references/a2ui-components.md` link, a stale skill.toml deferral comment, and a v1.6.0 claim against a 0.1.0 crate. Analysis D-02, D-03, D-13 chose one normative spec, a schema validated in Rust and shell, one root, and a drift-check script instead of a generator.

## What Changes

- Author `references/schemas/research-manifest.schema.json` (draft 2020-12) from `research-package-spec.md`; the spec becomes normative, drops the `.research` directory suffix, names `plan.md`, and documents the package directory as `<slug>-<yyyymmdd>-<4hex>` with the daemon job id inside the manifest.
- Delete `templates/research-package-manifest.json`; make the SKILL.md manifest example a literal copy of a schema-valid manifest; rewrite `scripts/export-package.sh` to emit every schema field from real package state (no hardcoded `feynman_grade: null`, `contradictions_resolved: 0`, `graph_nodes: 0`) and to accept the `--ingest-palace` flag that `research-package-spec.md` already documents and the script ignores (assessment research gap 9, documentation drift under G1 and constraint C-03).
- One output root `~/.prometheus/research/` in `config.rs`, `checkpoint.rs`, and the four hooks; `RESEARCH_OUTPUT_DIR` remains the override; daemon `status` output names the legacy `~/.research-jobs/` path when it exists (D-14, no migration).
- Fix documentation drift: skill.toml comment, SKILL.md version text, A2UI component list matched to `a2ui/registry.rs`, new `references/a2ui-components.md` with the eight real prop schemas; `agents/report-synthesizer.md` reads `plan.md`.
- Add `scripts/check-research-package.sh`: validates the SKILL.md example and a fresh `export-package.sh` output against the schema with python3 `jsonschema` when importable; when it is not, the jq fallback compares the document's full key set (not only required keys) and each declared type against the schema's `properties`, and the check reports `PASS WITH NOTES: jsonschema unavailable, structural check only`; exits non-zero on any mismatch under either path.
- Author OpenSpec capability `openspec/specs/research-pipeline-execution/spec.md` covering package layout, manifest schema, output root, and driver contract (the driver requirements are added by change-rah-003); `openspec validate` passes.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/research/deep-research/references/research-package-spec.md`
- `skills/research/deep-research/references/schemas/research-manifest.schema.json`
- `skills/research/deep-research/references/a2ui-components.md`
- `skills/research/deep-research/templates/research-package-manifest.json`
- `skills/research/deep-research/SKILL.md`
- `skills/research/deep-research/skill.toml`
- `skills/research/deep-research/scripts/export-package.sh`
- `skills/research/deep-research/scripts/check-research-package.sh`
- `skills/research/deep-research/hooks/pre-research.sh`
- `skills/research/deep-research/hooks/post-stage.sh`
- `skills/research/deep-research/hooks/on-contradiction.sh`
- `skills/research/deep-research/hooks/post-export.sh`
- `skills/research/deep-research/agents/report-synthesizer.md`
- `substrate/prometheus-research/src/config.rs`
- `substrate/prometheus-research/src/job/checkpoint.rs`
- `substrate/prometheus-research/src/main.rs`
- `openspec/specs/research-pipeline-execution/spec.md`

## Capabilities

- `research-pipeline-execution (new)`

## ADDED Requirements

### Requirement: One manifest shape
The manifest written by `export-package.sh` and the example in SKILL.md SHALL both validate against `research-manifest.schema.json`. Under the jsonschema path any schema violation exits non-zero with the failing path named; under the jq fallback any key-set or type mismatch exits non-zero and the result is PASS WITH NOTES.

#### Scenario: Drift check
- **WHEN** `scripts/check-research-package.sh` runs with python3 jsonschema importable
- **THEN** it validates both documents and exits 0; removing one field from the example exits non-zero with that field's path named

### Requirement: One output root
WHEN no `RESEARCH_OUTPUT_DIR` is set, THEN every hook, script, and the daemon resolve `~/.prometheus/research/<package-dir>/`.

#### Scenario: Grep gate
- **WHEN** the hook and daemon sources are searched for `.research-jobs`
- **THEN** no hook contains it, and every daemon source line that contains it also contains the word `legacy` (the helper that names the legacy root and the status notice); documentation may describe the legacy root by name

### Requirement: Documentation matches code
SKILL.md SHALL list exactly the A2UI components registered in `a2ui/registry.rs` and SHALL not claim a daemon version that differs from `Cargo.toml`.

#### Scenario: Component parity
- **WHEN** the eight registry names are compared with the SKILL.md table
- **THEN** the sets are identical

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command; if another build is active, wait or record BLOCKED, never start a competing build. `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason, never described as passed.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1); capability is discovered, never assumed (rule 2).
- Constraint C-01 (generated artifacts): `SKILL.md` and `skill.toml` files edited in this phase are inputs only to `generate:skills-index`; `skill-system.json`, the harness adapters, and the service manifest are not edited by any change in this phase. `change-rah-011-integration-evidence-and-docs` is the named reconciliation change: it regenerates the skills index and runs `check:distribution`, `check-harness-adapters.js`, and `check:services-manifest` at certification. No earlier change claims distribution certification.

## Open Questions

- Whether `research-package-spec.md` keeps the OKF extension table or moves it into the schema's `description` fields (default: keep both, schema is authoritative).

## Unresolved review findings

Adversarial review (diff mode, judge k3 via rest-gateway, `cross_model_check: verified-distinct`) ran two rounds against this change's scoped diff. Round 1 returned 10 CRITICAL: five were packet visibility (the four new files and the template deletion were invisible because a stale `.git/index.lock` from an orphaned session blocked the intent-add; lock removed, files intent-added, round 2 saw them), and five were substantive. Round 2 returned 5 CRITICAL. Under the two-round cap they are carried here with their disposition; each fix below was proven by a command before archive, not re-vetted by the judge.

- CRITICAL (round 2, `export-package.sh`): hook failure swallowed because `$?` after `if !` is the negation's status. Confirmed by probe (a hook exiting 7 produced exit 0). Fixed: the hook runs with `set -e` suspended, its status is captured directly, and the script exits with it. Probe now returns 7.
- CRITICAL (round 2, `check-research-package.sh`): no jq fallback, python3 required. Fixed: `validate_manifest_jq` compares required keys, full key set, const, enum, and JSON type via jq when python3 is absent (`CHECK_RESEARCH_PACKAGE_NO_PYTHON=1` forces it for testing); a first version had a jq scoping bug and rejected valid manifests, found by the forced-path probe and fixed. Negative probe reports missing key, bad enum, and wrong type; valid manifests report PASS WITH NOTES.
- CRITICAL (round 2, `check-research-package.sh`): identity check compared parsed JSON, not bytes. Fixed: `cmp -s` on the extracted blocks, diff shown on mismatch.
- CRITICAL (round 2, schema): slug limit not enforced. Fixed: `package_id` pattern is `^[a-z0-9]+(-[a-z0-9]+){0,4}-[0-9]{8}-[0-9a-f]{4}$`; a six-word id is rejected, five accepted (probe).
- CRITICAL (round 2, `hooks/pre-research.sh`): directory created from the job id. Fixed: the hook takes `RESEARCH_PACKAGE_ID`, validates it against the same pattern, falls back to the job id with a visible NOTE, and creates `$OUTPUT_DIR/$PACKAGE_ID`. Probes: valid id creates the directory; six-word id exits 1.
- Round 1 substantive findings also fixed in round 2: `report-synthesizer.md` inputs now all use `<package_id>/`; the `.research-jobs` grep requirement was reworded (hook sources clean, daemon lines must carry the word `legacy`, documentation may name the path) in this spec and in the OpenSpec scenario, matching the gate the task already used.
