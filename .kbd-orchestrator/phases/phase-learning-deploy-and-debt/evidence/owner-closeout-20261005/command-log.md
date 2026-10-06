# Recorded validation runs — historical evidence only

No command in this log was re-executed during the owner-directed closeout. These are prior failed batches, not release certification.

## initial-20261005T210725Z

Start: 2026-10-05T21:07:25.353605+00:00; end: 2026-10-05T21:45:01.539961+00:00; exit: 1.

| Case | Result | Exit | Exact invocation |
|---|---|---|---|
| build-native-cli | PASS | 0 | cargo build --locked -p prometheus-cli --bin prometheus |
| build-hook-runtime | PASS | 0 | cargo build --locked --bin prometheus-hook |
| build-memory-candidate | PASS | 0 | cargo build --locked --no-default-features --features server-only,local-embeddings --bin surreal-memory-server |
| build-memory-prior | FAIL | -2 | cargo build --locked --no-default-features --features server-only,local-embeddings --bin surreal-memory-server |
| build-companion-hosts | BLOCKED | 2 | another Cargo/rustc process owns the machine |
| build-cache-integration | BLOCKED | 2 | another Cargo/rustc process owns the machine |
| build-companion-integration | BLOCKED | 2 | another Cargo/rustc process owns the machine |
| full-distribution | PASS | 0 | node scripts/generate-skill-system-distribution.js --check |
| full-harnesses | PASS | 0 | node scripts/check-harness-adapters.js |
| full-release-version | PASS | 0 | node scripts/check-release-version-matrix.mjs |
| full-workflow-policy | PASS | 0 | node scripts/check-workflow-policy.mjs |
| full-docs-sync | PASS | 0 | node scripts/docs-sync.mjs --check |
| mini-distribution | PASS | 0 | node scripts/generate-skill-system-distribution.mjs --check |
| prepare-install-source | BLOCKED | 2 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/prepare-learning-deploy-candidate.mjs --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-o6r1eblg/install-fixture --output /private/tmp/ldd-local-o6r1eblg/install-fixture/candidate --legacy-output /private/tmp/ldd-local-o6r1eblg/install-fixture/legacy |
| hook-bytecode | FAIL | 1 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-hook-bytecode-integration.sh --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-o6r1eblg --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T210725Z --hook-runtime-bin /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/crates/prometheus-hook/target/debug/prometheus-hook --legacy-payload /private/tmp/ldd-local-o6r1eblg/install-fixture/legacy |
| installer-prerequisites | BLOCKED | 2 | scratch-contained actual installer source preparation failed |
| partition | PASS | 0 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-memory-partition.sh |
| rebase-generated-ownership | FAIL | 1 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-rebase-regenerate.sh |
| real-cortex | BLOCKED | 2 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/shared/scripts/tests/test-cortex-mirror.sh --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-o6r1eblg --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T210725Z |
| mini-production | BLOCKED | 2 | node /Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final/scripts/tests/learning-deploy-integration.test.mjs |
| cache-prerequisites | BLOCKED | 2 | explicit existing absolute input missing: priorMemoryBinary |
| companion-prerequisites | BLOCKED | 2 | explicit existing absolute input missing: companionHeadlessBinary |
| full-production-build | FAIL | 1 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/site/node_modules/@docusaurus/core/bin/docusaurus.mjs build --out-dir /private/tmp/ldd-local-o6r1eblg/full-site |
| mini-production-build | FAIL | 1 | node /Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final/site/node_modules/@docusaurus/core/bin/docusaurus.mjs build --out-dir /private/tmp/ldd-local-o6r1eblg/mini-site |
| generated-byte-mode-certification | PASS | 0 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/certify-generated-materializations.mjs --manifest /private/tmp/ldd-approved-integration-inputs/materializations.json --scratch /private/tmp/ldd-local-o6r1eblg --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T210725Z |
| final-parent-remote-install-graph | PENDING | None | owner-approved dependency certification/merge precedes final exact parent pin approval and fresh-clone certification |

## initial-20261005T215759Z

Start: 2026-10-05T21:57:59.933416+00:00; end: 2026-10-05T22:20:25.978025+00:00; exit: 1.

| Case | Result | Exit | Exact invocation |
|---|---|---|---|
| build-native-cli | PASS | 0 | cargo build --locked -p prometheus-cli --bin prometheus |
| build-hook-runtime | PASS | 0 | cargo build --locked --bin prometheus-hook |
| build-memory-candidate | PASS | 0 | cargo build --locked --no-default-features --features server-only,local-embeddings --bin surreal-memory-server |
| build-memory-prior | PASS | 0 | cargo build --locked --no-default-features --features server-only,local-embeddings --bin surreal-memory-server |
| build-companion-hosts | PASS | 0 | cargo build --locked -p prometheus-substrate -p sovereign-sync --no-default-features --features prometheus-substrate/sovereign --bins |
| build-cache-integration | FAIL | 101 | cargo test --locked --no-default-features --features server-only,local-embeddings --test query_cache_production --no-run --message-format=json |
| build-companion-integration | PASS | 0 | cargo test --locked -p prometheus-substrate --no-default-features --features sovereign --test connected_headless_processes --no-run --message-format=json |
| full-distribution | PASS | 0 | node scripts/generate-skill-system-distribution.js --check |
| full-docs-sync | PASS | 0 | node scripts/docs-sync.mjs --check |
| prepare-install-source | PASS | 0 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/prepare-learning-deploy-candidate.mjs --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-07t76e9q/install-fixture --output /private/tmp/ldd-local-07t76e9q/install-fixture/candidate --legacy-output /private/tmp/ldd-local-07t76e9q/install-fixture/legacy |
| initial-committed-protected-integrity | BLOCKED | 2 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/certify-initial-protected-candidate.mjs --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --mini /Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final --surreal /Users/gqadonis/Projects/prometheus/worktrees/ldd-memory-final --companion /Users/gqadonis/Projects/prometheus/worktrees/ldd-companion-final --prepared-full /private/tmp/ldd-local-07t76e9q/install-fixture/candidate --scratch /private/tmp/ldd-local-07t76e9q --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T215759Z |
| hook-bytecode | PASS | 0 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-hook-bytecode-integration.sh --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-07t76e9q --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T215759Z --hook-runtime-bin /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/crates/prometheus-hook/target/debug/prometheus-hook --legacy-payload /private/tmp/ldd-local-07t76e9q/install-fixture/legacy |
| codex-memory | FAIL | 1 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-codex-memory-integration.sh --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-07t76e9q --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T215759Z --doctor-bin /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/tools/prometheus-cli/target/debug/prometheus --install-source /private/tmp/ldd-local-07t76e9q/install-fixture/candidate --codex-bin /opt/homebrew/Caskroom/codex/0.158.0/bin/codex --codex-provider-config /private/tmp/ldd-approved-integration-inputs/codex-provider.toml |
| codex-home | FAIL | 1 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-codex-home-integration.sh --full /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt --scratch /private/tmp/ldd-local-07t76e9q --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T215759Z --doctor-bin /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/tools/prometheus-cli/target/debug/prometheus --install-source /private/tmp/ldd-local-07t76e9q/install-fixture/candidate |
| research-merge-threads | BLOCKED | 2 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/skills/research/deep-research/tests/merge-threads.sh |
| research-report-assembly | PASS | 0 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/skills/research/deep-research/tests/report-assembly.sh |
| rebase-generated-ownership | PASS | 0 | bash /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/test-rebase-regenerate.sh |
| mini-production | FAIL | 1 | node /Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final/scripts/tests/learning-deploy-integration.test.mjs |
| cache-prerequisites | BLOCKED | 2 | explicit existing absolute input missing: cacheIntegrationBinary |
| real-companion-connected | PASS | 0 | bash /Users/gqadonis/Projects/prometheus/worktrees/ldd-companion-final/scripts/tests/test-companion-connected-integration.sh --companion /Users/gqadonis/Projects/prometheus/worktrees/ldd-companion-final --scratch /private/tmp/ldd-local-07t76e9q --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T215759Z --headless-bin /Users/gqadonis/Projects/prometheus/worktrees/ldd-companion-final/target/debug/prometheus-companion-headless --sync-bin /Users/gqadonis/Projects/prometheus/worktrees/ldd-companion-final/target/debug/sovereign-sync --prometheus-bin /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/tools/prometheus-cli/target/debug/prometheus --companion-integration-bin /Users/gqadonis/.cargo-build/0e/f6f33dfee29efd/debug/deps/connected_headless_processes-a9e0450ab6824d95 |
| full-production-build | FAIL | 1 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/site/node_modules/@docusaurus/core/bin/docusaurus.mjs build --out-dir /private/tmp/ldd-local-07t76e9q/full-site |
| mini-catalog-generation | PASS | 0 | node /Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final/site/scripts/generate-skills-catalog.mjs |
| mini-production-build | PASS | 0 | node /Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final/site/node_modules/@docusaurus/core/bin/docusaurus.mjs build --out-dir /private/tmp/ldd-local-07t76e9q/mini-site |
| mini-site-prerequisites | FAIL | 1 | new product topic absent: handoff |
| generated-byte-mode-certification | PASS | 0 | node /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/scripts/tests/certify-generated-materializations.mjs --manifest /private/tmp/ldd-approved-integration-inputs/materializations-correction.json --scratch /private/tmp/ldd-local-07t76e9q --evidence /Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt/.kbd-orchestrator/phases/phase-learning-deploy-and-debt/evidence/local-integration/initial-20261005T215759Z |
| final-parent-remote-install-graph | PENDING | None | owner-approved dependency certification/merge precedes final exact parent pin approval and fresh-clone certification |
| requested-case-coverage | BLOCKED | 2 | requested cases did not execute |

