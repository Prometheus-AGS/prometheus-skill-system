# Artifact-refiner QA — add-kbd-registry-prune

## Specification

- Artifact reviewed: the completed registry-prune Rust library, CLI, external integration scenarios, operator documentation, and OpenSpec evidence.
- Target: an explicit, recoverable cleanup path for replica registrations whose checkout paths are absent.
- Required behavior: dry-run byte immutability; locked apply-time re-evaluation; write-ahead backup, checksum, receipt, and rollback guidance; runtime-history and valid-registration preservation; repeat-run idempotence; explicit CLI apply authority.

## Deterministic constraint evaluation

| Constraint | Status | Evidence |
|---|---|---|
| C-01 generated artifacts | Satisfied for this change; parent gate remains open | No generator or generated plugin surface was changed. `execution.md` names `reconcile-kbd-control-plane-projections` task 2.1 as the required owner and explicitly withholds distribution certification. |
| C-02 no committed secrets | Satisfied | The reviewed Rust, integration, documentation, and OpenSpec surfaces contain only filesystem paths, hashes, UUIDs, and synthetic fixture identities; no credential value was introduced. |
| C-03 documentation parity | Satisfied | Both CLI indexes and the registry runbook use `prometheus kbd --path <project> projects --prune-missing [--apply] [--json]`, matching the parent-global `--path` and the `projects` subcommand flags. The runbook includes backup and rollback semantics. |
| C-04 generator idempotence | Not applicable locally; parent gate retained | The change modifies no generator or generated plugin output. The named reconciliation change retains the required two-run hash proof. |
| C-05 Bash 3.2 | Not applicable | The change modifies Rust, external Rust integration targets, Markdown, and OpenSpec artifacts; it changes no launchd-reachable shell script. |
| Registry-prune specification | Satisfied | Static inspection confirms shared-lock dry run, exclusive-lock apply re-evaluation, fail-closed `try_exists` errors, pre-mutation synced evidence, same-directory atomic registry replacement, exact-key removal, no runtime-tree deletion, and non-mutating repeat behavior. |

## Integration and certification evidence

- `cargo test --manifest-path substrate/kbd-runtime/Cargo.toml --test registry_prune`: 2/2 external integration scenarios passed.
- `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd`: 7/7 external binary integration scenarios passed, including registry pruning.
- Package/compiler gates passed with warnings denied for both affected Rust surfaces.
- Release builds passed for `control-plane-recover` and `prometheus 1.7.0`; exact gate receipts and SHA-256 values are in `openspec/changes/add-kbd-registry-prune/execution.md`.
- Protected-test verification, strict OpenSpec validation, public-doc validation, Rust formatting, and `git diff --check` passed locally.
- The broader docs-contract gate is truthfully deferred because its generated plugin input is owned by the next reconciliation change; it is not claimed as passing.

## Reflection

- Correctness: pass.
- Recovery evidence: pass.
- Regression check: ordinary registry listing remains unchanged; maintenance mode is explicitly selected; runtime history and existing shared-project replicas are preserved.
- Documentation: pass.
- Blocking violations: 0.

## Verdict

PASS. The change converged in one deterministic QA iteration. Proceed to strict KBD/OpenSpec verification and archive. Parent-phase distribution reconciliation, installed-surface refresh, live registry application, service restart probes, and final certification remain owned by `reconcile-kbd-control-plane-projections`.
