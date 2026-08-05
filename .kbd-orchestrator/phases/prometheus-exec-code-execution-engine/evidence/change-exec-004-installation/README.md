# change-exec-004 final installation certification

Certified locally on 2026-08-05 after merging remote `main` into
`codex/prometheus-exec`. GitHub Actions, installed KBD, the KBD doctor wrapper,
KBD-backed memory, and Sovereign Sync were not invoked.

## Certified source

- Final product commit: `8d93ecd78ec759f49301782002be75dbf25b5679`
- Root merge commit: `477c39bd20e666780d264c1eeadbf405e81ad8ab`
- Prometheus Exec tested implementation commit: `68f960f26d7aa3aad058042dfb2aa3a73e8b0b29`
- Knowledge pin: `cea7b9063bd0c8b2fe4c2a59f04e5e1eee87d844`
- Memory pin: `c8719ace0b8d778acb590250301d541ebae6c3c2`
- The main merge preserved the previously certified execution Rust tree.
  Independent final review then found and drove focused corrections for MCP
  response-loss replay, remote target enrollment, the estate feature boundary,
  strict installer hash certification, execution-time dispatch expiry, live
  MCP runner readiness, and placeholder certification hashes. Those corrected
  surfaces were recompiled and checked locally before final installation.

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
  `0600`, same-UID health succeeds, and all 14 focused doctor checks pass,
  including explicit LaunchAgent loaded-state verification.
- Surreal Memory reports full ledger, tokenizer, model-executor, search-index,
  ingestion, and search readiness.
- The learning worker has zero pending, processing, retry, submitting,
  accepted, rejected, or dead-letter records and 11 terminal memory receipts.
- Signed generation
  `3f71ba5e9b617bea70f40eae55549f9d4e3c0eec5c887ff78ac995dba82e2a95`
  was produced from a clean detached worktree at the final product commit.
- Bundle
  `19bdc79888062f3a07a2ef2dc9cb52b307f1554914955ce701400cfc1b743e2b`
  and all 14 AI-tool target receipts verify.
- The source, active immutable generation, bundle index, stable dispatchers,
  and installed Codex hook cache agree on all 30 hooks.

## Commands and results

- Release builds used stable Rust, `RUSTFLAGS=-Dwarnings`, an internal-SSD
  Cargo home, and explicit workspace target directories.
- `cargo test -p pk-learning-worker`: 14 passed, 0 failed.
- Prior full prometheus-exec phase certification remains archived with the real
  MCP, Tier P, Tier W, response-loss, restart, portable-evidence, and two-peer
  results in `../change-exec-004-real-use-cases/`. Final-review corrections add
  focused passing regressions for MCP same-ID replay/hash conflict, unknown
  remote targets before queue insertion, non-estate dependency selection, and
  strict installer hash mismatch handling. The final remediation additionally
  passed focused regressions proving execution-time expiry does not invoke the
  executor, a stale Unix socket is not treated as readiness, and the checked
  certification-status JSON serializes only real archived hashes.
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
- Focused `prometheus-exec doctor --format json`: healthy, 14/14 required
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
