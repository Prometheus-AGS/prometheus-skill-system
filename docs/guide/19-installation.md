# 19 · Installation

Install from approved source and its pinned dependencies. Skills, native binaries and services have independent versions and evidence. Installation is a mutation; preserve existing ownership receipts and data before changing a machine.

## Select a profile

```bash
git clone https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
./install.sh --profile skills
```

Git and compatible Node are required; the configured development line is Node 22 and the root engine requirement is in `package.json`. The root installer initializes the exact imports for the profile. It offers detected clients and prints its mutation plan. Automated selection is explicit:

```bash
./install.sh --profile skills --targets detected --non-interactive --yes
./install.sh --verify --targets detected --non-interactive
```

Release and minimum-active versions come from `skill-system.json`. `--best-effort` permits partial development outcomes and is not certification. Full skills on Windows require Git Bash or WSL; mini is the Node-only alternative. Full profile is supported on macOS/Linux only:

```bash
./install.sh --profile full
```

Native service installation requires Bash 4 or newer. macOS `/bin/bash` 3.2 is insufficient. Service availability follows actual templates: execution and liter-llm API have macOS templates; other operating systems need their own supported setup. A binary's platform support does not guarantee a service template.

## Selected home and immutable payload

A nonempty inherited `CODEX_HOME` selects Codex's root, otherwise the selected `--home` user's `.codex` is used. Preserve this choice through installation, verification, rollback and removal. Logical plugin-store receipts and the selected Codex projection are different boundaries.

```bash
CODEX_HOME="/absolute/custom-codex" ./install.sh --profile skills --targets codex
CODEX_HOME="/absolute/custom-codex" ./install.sh --verify --targets codex --non-interactive
```

The installer verifies signed payloads and selected projections; copy targets retain ownership evidence. Unknown copies and unrelated skills are preserved and diagnosed. Never edit generated packages, immutable installed generations or harness caches by hand.

## Services and optional components

The [service operations guide](26-service-operations.md) is the canonical runbook for binary/service install, transport, namespace/database, credentials and recovery. Building learner/bridge binaries does not start their service. Companion owns cross-device sync and is not installed by this pack.

Codex memory policy disables `features.memories`, `memories.generate_memories` and `memories.use_memories`, archives only known v1/v2 summary files, and preserves other memory data. Existing sessions retain configuration; start a new session and account for explicit overrides. See [memory tiers](memory-tiers.md).

## Local evidence

Maintainers finish every phase production change before tests, formatting, builds or review. Use isolated homes, data, keys and ports. Serialize Cargo/rustc machine-wide and keep separate worktree target directories; use `sccache` for reuse.

After source completion, exercise the real installed production entrypoints and archive exact local results. Version strings, configured endpoints, listening ports and build success each prove only their own observation. Hosted testing is forbidden; permitted documentation packaging does not replace local certification.

Previous: [Plugins and Immutable Distribution](18-plugins-and-marketplace.md) · Next: [Updating](20-updating.md).
