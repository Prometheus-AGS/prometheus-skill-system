# All-tools rebuild and reinstall — 2026-08-23

## Scope and source

The parent repository and every recursive submodule were fetched before the
rebuild. Each checkout was already at its remote default-branch tip, so this run
required no submodule repin. The deployment was produced from the clean
`codex/all-tools-reinstall-20260823` branch after commit `de68e8c`.

No test suite or hosted workflow was run. This record covers only source
reconciliation, release builds, installation readback, service restart, and
local operational health.

## Installer correction

The first clean build exposed a reproducibility defect in the
`prometheus-exec` installation contract. Its toolchain file selected the
floating `stable` channel while the installer required one committed binary
SHA-256. When stable advanced to Rust 1.97.1, unchanged source produced a new
binary and installation stopped at the stale hash gate.

The toolchain is now pinned to exact Rust 1.97.1 and
`config/prometheus-exec-binary.json` records the corresponding unsigned release
binary SHA-256 `a67de3b6c4f41a471e00bcfb683de5afa3164ddbf96af6b1ed4e434d9d079b56`.
The signed installed binary has its own readback hash because ad-hoc macOS code
signing mutates the Mach-O file.

The all-binary installer also claimed release-download fallbacks for cowork and
disk-space-guardian, but `set -euo pipefail` terminated the script on a failed
source build before either fallback could execute. Both source-build branches
now fall through explicitly when Cargo fails or omits the expected binary.

## Installed tools

The canonical all-binary installer rebuilt and installed:

- `prometheus` 1.7.0 and `prometheus-exec` 1.7.0
- Forge CLI/MCP
- `pk` 1.7.0, `pk-cherry`, and `prometheus-learning-worker`
- `learner-model`, `surface-bridge`, and `sovereign-sync`
- `liter-llm` 1.18.1 and `openai-proxy`
- `surreal-memory-server` 1.7.0 in both required PATH locations
- `sycophancy-correction` 1.0.0 in both required PATH locations
- `template-forge` and `template-forge-mcp`
- `dsg` (reports 0.1.0) and `prometheus-research` 0.1.0

The network intermittently presented an untrusted self-signed Arris router
certificate for crates.io and GitHub. TLS verification was never disabled and
the certificate was not trusted. Cowork 0.2.0 and rust-mcp-filesystem 0.4.3 were
therefore installed from their upstream macOS arm64 release archives when their
cold source builds needed unavailable registry objects. Both artifacts were
ad-hoc signed after copying; rust-mcp-filesystem was installed to
`~/.local/bin` and `/usr/local/bin` with matching hashes.

## Skills, plugins, and services

`scripts/update-skill-pack.sh --force` installed immutable generation
`1d95ae1c8c795097951a635bd3285dee3f56eba313487eda420976b38585bd89`:

- 163 skills across all 14 configured target payloads
- refreshed Claude Code marketplace and native plugin installation
- refreshed Codex marketplace and affected plugin installations
- refreshed Kimi Desktop plugin payload

All managed LaunchAgents were rendered again and restarted from the final
installed binaries. A transient launchd `Bootstrap failed: 5` occurred for
`ai.prometheus.exec` immediately after bootout; the dedicated service installer
was rerun and verified the loaded socket service successfully. The research
LaunchAgent was also replaced and restarted by the binary installer.

## Local operational evidence

- `scripts/check-mcp-health.sh`: SurrealDB, surreal-memory readiness,
  pk-cherry, Forge, surface-bridge, sovereign-sync, and prometheus-exec healthy
- `prometheus doctor`: exit 0, four optional warnings, no required failures
- managed binary doctor: 6/6 executable, hashed, and signed
- `pk doctor --json`: 6 passed, 0 warned, 0 failed
- learning state: 940 completed jobs, 960 completed memory operations, and zero
  pending, processing, retry, rejected, or dead-letter records
- prometheus-research `/health`: HTTP 200
- installed immutable generation verification: exact match

The four optional Prometheus doctor warnings are unchanged operational
advisories: an unauthenticated liter-llm probe receives the expected HTTP 401,
the KBD compatibility HTTP diagnostic is unavailable while the canonical
sovereign-sync Unix-socket health probe passes, production rollout observation
has not started, and harness discovery budgets are not measured. None is a
required service failure.
