# 17 · Platform and harness support

Support has separate layers: source packaging, installation, native discovery, hook delivery, service availability and actual operation. A copied skill or exported agent definition does not prove the remaining layers.

## Host installation

| Pack/path | Declared host support | Boundary |
|---|---|---|
| Full skills profile | macOS, Linux, Windows Git Bash/WSL | Shell-based installation and selected target projections |
| Full profile | macOS, Linux | Native binaries and platform-specific service templates |
| Mini | Windows, macOS, Linux with Node 22+ | Copy-only, reduced Node runtime and optional Compose services |
| Companion | Separate repository/release | Optional connected control and synchronization |

Full native service installation requires Bash 4+. Execution and liter-llm API templates are macOS-only even where a binary can run elsewhere. Full native memory and mini Compose use different databases. See [service operations](26-service-operations.md).

## Harness projections

`skill-system.json` defines current targets and modes. Full Codex and MiniMax use verified real-directory copies; other supported targets use the generation projection declared in that manifest. Mini uses copy mode for every target. Selected custom Codex homes are carried through installation and lifecycle operations.

Native plugins, skill directories, agent definitions and MCP registration are separate. Codex configuration lives under the effective `CODEX_HOME`; Claude, OpenCode, Kimi and other clients retain their own formats and permissions. Generated hook adapters express supported events but never gate ordinary tool mutation on KBD state.

Team exports preserve only controls supported by the inspected adapter. Native model aliases, reasoning settings, forks and per-member overrides vary by installed harness/provider. The [team native/model guide](24-agent-teams.md#reasoning-and-native-harness-limits) records those limits and primary sources. Record the actual route used by each worker instead of inferring it from planned policy.

## Recall and session continuity

Full role recall is scoped and bounded. Codex parent SessionStart receives only local team digest metadata; forks still inherit their parent conversation. A digest is not permission to retrieve a private lesson. Mini keeps its smaller Claude file tier/outbox and does not claim full Python/Cortex hook parity.

Local signed KBD and explicit team handoffs preserve workflow evidence across sessions. Native conversation transfer and another project's write authority need their own authorization and supported harness path.

## Execution platforms

Prometheus Exec is independent of ordinary harness shell permissions. Windows Tier P has no backend. Linux source/cross-build evidence is different from real kernel runtime acceptance. Tier W embedded/mobile profiles need their own size and physical-device evidence. Read [platform and evidence status](/docs/execution/platform-and-evidence-status) with its named historical release boundary; none of it certifies a newly installed host.

Previous: [CLI and Scripts](16-cli-and-scripts.md) · Next: [Plugins and Immutable Distribution](18-plugins-and-marketplace.md).
