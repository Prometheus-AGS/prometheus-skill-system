# change-exec-004 final installation certification

Certified locally on 2026-08-05 after merging remote `main` into
`codex/prometheus-exec`. GitHub Actions, installed KBD, the KBD doctor wrapper,
KBD-backed memory, and Sovereign Sync were not invoked.

## Certified source

- Root merge commit: `477c39bd20e666780d264c1eeadbf405e81ad8ab`
- Prometheus Exec tested implementation/evidence commit: `73056af9b215847cdc62e89359e6bfb24ad80693`
- Knowledge pin: `cea7b9063bd0c8b2fe4c2a59f04e5e1eee87d844`
- Memory pin: `c8719ace0b8d778acb590250301d541ebae6c3c2`
- The prometheus-exec Rust tree is byte-identical between `73056af` and the
  merge commit. The merge-affected worker and installation surfaces were
  revalidated separately.

## Installed binaries

All six binaries return exact version `1.7.0`, pass strict macOS code-signature
verification, and have identical bytes at `<home>/.local/bin` and
`/usr/local/bin`. Unsigned build and signed installation hashes are recorded
in `installed-binaries.json`.

The binaries were signed once under canonical identifiers and then copied
atomically. This corrected an observed staging-filename defect where signing
the same Memory binary independently could produce different installed bytes.

## Runtime and distribution

- Prometheus Exec LaunchAgent is loaded, its owner-only Unix socket is mode
  `0600`, same-UID health succeeds, and all 13 focused doctor checks pass.
- Surreal Memory reports full ledger, tokenizer, model-executor, search-index,
  ingestion, and search readiness.
- The learning worker has zero pending, processing, retry, submitting,
  accepted, rejected, or dead-letter records and 11 terminal memory receipts.
- Signed generation
  `5a5d675963ae2b4b33b81868a88663f7e6efd92008daef083511f08981005873`
  was produced from a clean detached worktree at the root merge commit.
- Bundle
  `19bdc79888062f3a07a2ef2dc9cb52b307f1554914955ce701400cfc1b743e2b`
  and all 14 AI-tool target receipts verify.
- The source, active immutable generation, bundle index, stable dispatchers,
  and installed Codex hook cache agree on all 30 hooks.

## Commands and results

- Release builds used stable Rust, `RUSTFLAGS=-Dwarnings`, an internal-SSD
  Cargo home, and explicit workspace target directories.
- `cargo test -p pk-learning-worker`: 14 passed, 0 failed.
- Prior full prometheus-exec phase certification remains applicable because the
  Rust tree is unchanged; the real MCP, Tier P, Tier W, response-loss, restart,
  portable-evidence, and two-peer results are archived in
  `../change-exec-004-real-use-cases/`.
- `npm run validate:harness-adapters`: 30 hooks across both harness manifests
  match the merged bundle.
- `bash scripts/tests/install-policy.test.sh`: strict, best-effort,
  skills-only, and false-green policies pass.
- `node scripts/install-plugin-generation.js --verify`: active generation
  verifies.
- Canonical root `doctor --json`: 0 required failures, 12 passes, 1 warning,
  3 optional skips.
- Canonical root doctor fix and refresh modes were run with `--dry-run`;
  both returned no required failures and made no changes.
- Focused `prometheus-exec doctor --format json`: healthy, 13/13 required
  checks pass.

Every doctor invocation excluded these scopes before check construction:

- `control.kbd-runtime`
- `state.kbd-orchestrator`
- `control.kbd-rollout`
- `service:sovereign-sync`

## Warning dispositions

- `skills.discovery-budget`: non-required measurement warning. The host has
  not yet accumulated discovery measurements for four harness families. This
  does not weaken installation evidence: nine installed agents are detected
  and the signed generation independently verifies all 14 target receipts.
- `mcp.config`: optional reserved check, skipped because declarative MCP
  reconciliation is not implemented. No success claim is inferred from it.
- `state.evolver` and `learning.trace-store`: optional, unused surfaces;
  both remain explicitly skipped.
- Metal local-embedding warmup exposed an Apple compiler
  `XPC_ERROR_CONNECTION_INTERRUPTED` in Candle 0.9.2. The installed Memory
  release uses the same local BGE model on the supported CPU backend; a
  read-only warmup completed and full readiness is green. No GPU-readiness
  claim is made.
