# Skill Pack 1.10.0 release and deployment closeout

Date: 2026-09-19
KBD boundary: `skill-pack-1-10-recovery/release-skill-pack-1-10-recovery/recovery-6`
Task class: release

## Delivery delta

The recovery release was merged and published, but deployment found two macOS service defects after the first tag: `ai.prometheus.exec` could race an asynchronous launchd bootout, and cold MLX startup exceeded the memory service's 120-second deadline. PRs #86 and #87 repaired those observed failures. The memory executor now warms successfully within the five-minute bound, while the memory HTTP API remains degraded during storage/index initialization on this host under 28.6 GB of swap use. KBD memory writes must use the durable markdown/outbox fallback until that endpoint opens.

## Merged source

- PR #84: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/84
- PR #85: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/85
- PR #86: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/86
- PR #87: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/87
- Latest product merge before the KBD closeout projection: `489032e36fa81f01420a000b7fa57d08734021cf`
- Published release: https://github.com/Prometheus-AGS/prometheus-skill-system/releases/tag/v1.10.0

## Observed verification

- `node scripts/test-skills.js` — exit 0, 20/20 deterministic suites passed after the final service-template change.
- `bash scripts/tests/install-prometheus-exec-service.test.sh` — exit 0, launchd bootstrap retry regression passed.
- `bash scripts/tests/install-mcp-services.test.sh` — exit 0, the rendered plist contains `SURREAL_EXECUTOR_STARTUP_MS=300000`.
- `gitleaks git --staged --no-banner --redact .` — exit 0 for both service-fix commits, no leaks found.
- Isolated release-behavior critic — PASS for the memory startup budget and its production-entry regression.
- `bash scripts/install-mcp-services.sh --restart` — exit 0; SurrealDB, surreal-memory, pk-cherry, forge-mcp, surface-bridge, liter-llm, scheduled jobs, and prometheus-exec were rendered and loaded; the exec socket was verified.
- `curl http://127.0.0.1:28000/health` — HTTP 200.
- `curl http://127.0.0.1:8943/health` — HTTP 200 on a 15-second probe.
- `prometheus-exec` socket — listening at `~/.prometheus/run/prometheus-exec.sock`.
- Surreal memory log — `Embedding model warmed up and ready`, then `Current schema version: v21`; `/health` and `/ready` still returned HTTP 000 while storage/index initialization continued.
- `sysctl vm.swapusage` — 29,696 MB total, 28,559 MB used, 1,137 MB free at the degraded memory check.

## Release facts and limitation

The umbrella version is 1.10.0 and the release is published. Product code is merged on `origin/main`. The annotated tag and immutable 14-target installation are finalized after the generated KBD completion projection is merged, so the installed source receipt points at the final tagged commit. The memory HTTP endpoint is an explicit degraded-service limitation; progress recording continues through the required local log and durable outbox.

This release certifies the skill-pack recovery and deployment. It does not certify Universal Agent Runtime. production-ready has no date.

## Recovery task 7 — installed progress-memory fallback

The recovery-6 boundary initially returned `degraded`: `pk` timed out and the copied skill looked for `/Users/gqadonis/shared/scripts/enqueue-memory-operation.py`. Commit `fa9c028` resolves project, source, and immutable installed layouts, keeps degraded receipts retryable, and recognizes the already-written legacy `complete: true` plus `degraded` receipt as pending delivery.

- `python3 skills/process/karpathy-progress-memory/tests/progress-memory-integration.py` — exit 0, 13 scenarios passed; the installed-copy scenario proves `degraded → queued → duplicate`, receipt completion, and a stable outbox count.
- `node scripts/validate-skills.js --strict --exclude-submodules skills/process/karpathy-progress-memory` — exit 0, one skill valid with no warnings.
- `node scripts/test-skills.js` — exit 0, 20/20 deterministic suites passed on the final artifact.
- `gitleaks git --staged --no-banner --redact .` — exit 0, no leaks found.
- Critic round 1 — PASS for resolver ordering and bounded outage behavior.
- Critic round 2 — BLOCK because the integration did not re-read the recovered receipt or replay it once more. The final integration adds both assertions and passes. No third review was requested under the two-round cap.
- Real recovery-6 replay — `queued`, stable event `kpm-fdbbdff4696c984d0e4d66d0a24cf858`, durable operation `d3ef35debdfbfd948bd50281168ae6d5bbd90145e596eb5fcc9b6c9d1a1d4b62`.
- Recovery-7 boundary — `queued`, stable event `kpm-21cd028300ef297c322cef9eccb2473a`, durable operation `a481b1ad6f0c8f341b4d8ea21fdd5dc8586afa1fd2abb99a2c509bd46431595b`.
