## Local verification

Date: 2026-08-25

### Passing gates

- `openspec validate initialize-kbd-runtime-on-first-mutation --strict` — valid.
- `cargo test --manifest-path substrate/kbd-runtime/Cargo.toml projections_are_atomic_canonical_and_replayable` — 1 passed.
- `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli ensure_runtime_initializes_from_legacy_once` — 1 passed.
- `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd` — 3 passed, including empty and legacy-populated first-mutation paths.
- `cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli` — 28 passed before the final empty-project scenario was added; the complete 3-test process target passed afterward.
- `cargo fmt --manifest-path substrate/kbd-runtime/Cargo.toml --all -- --check` — passed.
- `cargo fmt --manifest-path tools/prometheus-cli/Cargo.toml --all -- --check` — passed.
- `cargo clippy --manifest-path substrate/kbd-runtime/Cargo.toml -- -D warnings` — passed.
- `cargo clippy --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli -- -D warnings` — passed.
- `cargo build --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli` — passed.
- `cargo build --release --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli` — passed in 4m 39s.
- `git diff --check` — passed.

### Installed-binary proof

- Release artifact before installation: SHA-256 `0b182d7f2efa6dad14992648ca34b41dc7777cd0da447dfbd2b6fc5e484db129`.
- Installed, ad-hoc signed CLI: `/Users/gqadonis/.local/bin/prometheus`, SHA-256 `1f3d8d5a35c7012bdc43a89ef87cae0002808abc9b4a7af5933cd742c659fc70` after signing; `codesign --verify --verbose=2` passed.
- Rollback copy: `/Users/gqadonis/.local/bin/backups/prometheus-20260825T105512Z`, preserving the prior SHA-256 `72fb22a2472a68f05596854b0d0e8dc97798ec9a932d59cdee699d1e2f277b3c`.
- `PROMETHEUS_CLI_TEST_BINARY=/Users/gqadonis/.local/bin/prometheus cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd` — 3 passed against the installed binary.
- `ai.prometheus.sovereign-sync` remained running with PID 26260 and run count 5 before and after installation. The daemon binary was not changed or restarted.

### Broader-suite baseline

`cargo test --manifest-path substrate/kbd-runtime/Cargo.toml` reported 66 passed,
1 failed, and 6 ignored. The failing repository-fixture test is
`every_repository_ledger_shape_migrates_in_a_recoverable_copy`; its copied legacy
ledgers contain aggregate completion counters that the safety guard refuses to
rewrite when no equivalent per-change rows exist.

The same focused test was run from a clean detached `origin/main` worktree at
`1308e4b7a5d023e50bc0676ce497003b0bf7597b` and failed for the same guard, naming
30 projection-ahead phases. This patch does not weaken that guard. Refreshing the
post-import state reduces the mismatch set to the five genuinely ambiguous
empty-row ledgers, which remain rejected rather than losing completion data.
