# 19 · Installation

This page is the procedure for getting from a fresh clone to a running, self-improving loop. It covers prerequisites, the toolchain, the one-command install, the MCP services, and verification — in the order you actually run them.

## Prerequisites

Two things are hard requirements; the rest are installed or built for you.

**Required:** Node.js ≥ 18 and Git.

**For the Rust tools and Rust skills:** the Rust toolchain. This is the one prerequisite the pack cannot install silently, because it is what builds everything else.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown   # for librefang-wasm and the native-agent WASM build
rustup show
```

**For Go skills:** a Go 1.22+ toolchain (backs `go/go-base-patterns` and the Flint Go SDK). **For graph memory:** Docker (the recommended way to run surreal-memory). The prerequisite script detects all of these and reports what is missing.

```bash
# Check everything at once — toolchains, services, Prometheus binaries
bash shared/scripts/detect-toolchain.sh
bash shared/scripts/detect-toolchain.sh --json   # machine-readable
```

## The install flow

```mermaid
graph TD
    A[git clone --recurse-submodules] --> B[check-prerequisites.sh --build-tools]
    B --> C[install-skills-flat.sh — skills to all platforms]
    C --> D[install-mcp-services.sh — launchd / systemd --user]
    D --> E[configure-mcp-all-tools.sh — per-tool MCP config]
    E --> F[prometheus-services.sh load]
    F --> G[check-mcp-health.sh — verify]
    G --> H[forge init — initialize a project]
```

### Step 1 — clone with submodules

The imported skills and three of the tools are git submodules, so a plain clone is not enough.

```bash
git clone --recurse-submodules https://github.com/Prometheus-AGS/prometheus-skill-pack
cd prometheus-skill-pack

# If you already cloned without --recurse-submodules:
git submodule init && git submodule update
```

### Step 2 — build the tools

```bash
# Check/install prerequisites and build all six tool binaries to ~/.local/bin/
bash scripts/check-prerequisites.sh --install --build-tools

# Read-only doctor surface from the compiled CLI
npm run doctor

# Optional dry-run repair planning surfaces
npm run doctor:fix
npm run doctor:refresh
```

After this, `~/.local/bin/` holds `prometheus`, `forge`, `pk`, `pk-cherry`, `liter-llm`, `surreal-memory-server`, and `prometheus-rust-auditor`. Make sure `~/.local/bin` is on your `PATH`.

### Step 3 — install the skills to every platform

```bash
# Flat-symlink install into each detected platform's skills dir
bash scripts/install-skills-flat.sh
```

This installs to Claude Code (`~/.claude/skills/`), Kimi (`~/.kimi-code/skills/`), MiniMax (`~/.minimax/skills/`), OpenCode (`~/.opencode/skills/`), Codex (`~/.codex/skills/`), Cursor (`~/.cursor/skills/`), and the others — turning each skill into a slash command — and configures kimi-code MCP along the way. For full plugin support on OpenCode and the richer per-platform install, use `npm run install:platforms`.

### Step 4 — bring up the MCP services

```bash
# macOS: render LaunchAgents into ~/Library/LaunchAgents and bootstrap them
bash scripts/install-mcp-services.sh

# Configure all MCP servers into every installed AI tool's native config
bash scripts/configure-mcp-all-tools.sh

# Load and check
bash scripts/prometheus-services.sh load
bash scripts/prometheus-services.sh status
bash scripts/check-mcp-health.sh
```

The Sovereign Sync installer also creates:

- `$HOME/.config/sovereign-sync/config.toml` with a random P2P `operator_id`;
- `$HOME/.config/sovereign-sync/device-key.json` with mode `0600`;
- `ai.prometheus.sovereign-sync` on loopback port `7892`.

The KBD REST bearer token is separate. It is generated per project in the
platform data directory when the canonical runtime first opens. Do not use
`operator_id` or the Ed25519 device key as the bearer token. The detailed path,
format, rotation, and verification procedure is in
[Tokens and authentication](/docs/kbd/tokens-and-authentication).

On macOS the `launchd` agents manage `pk-cherry` on `127.0.0.1:8942` and `forge mcp` on `127.0.0.1:8943`. surreal-memory is Docker-managed on `127.0.0.1:23001`:

```bash
cd tools/surreal-memory-server && docker compose up -d
curl -s http://localhost:23001/health | jq .
```

On Linux the same `bash scripts/install-mcp-services.sh` renders `systemd --user` units into `~/.config/systemd/user/`, enables lingering, and `systemctl --user enable --now`s each daemon — the binaries and ports are identical; only the service manager differs, and the installer picks it automatically. The bundled SurrealDB runs on `127.0.0.1:28000` (separate from any external instance on `:8000`), and already-running services are detected and reused rather than double-started.

### Step 5 — verify

```bash
npm run doctor                      # compiled CLI, read-only diagnosis
npm run doctor:fix                  # dry-run safe repair planning
npm run doctor:refresh              # dry-run pinned-source refresh planning
bash scripts/check-mcp-health.sh   # launchctl + HTTP probe per service
prometheus doctor --json           # machine-readable CLI health
prometheus doctor --check learning # scoped CLI health
```

For a controlled project, also verify identity and canonical runtime:

```bash
PROJECT_ROOT="/path/to/project"

prometheus kbd --path "$PROJECT_ROOT" migrate --check
prometheus kbd --path "$PROJECT_ROOT" migrate --apply
prometheus kbd --path "$PROJECT_ROOT" status --json | jq .
```

When the daemon must control a repository other than its service working
directory, configure `KBD_FOCUS_PROJECT_PATH` and fully reload the service.
Changing only the token does not retarget Sovereign Sync.

## First run

With everything up, initialize forge in a project and define your first loop:

```bash
# Initialize the enrichment engine in your project
forge init

# Define and tick a continuous quality loop
cat > loop.json << 'EOF'
{
  "name": "quality-gate",
  "goal": {
    "description": "All tests pass and no CRITICAL/HIGH lint errors",
    "measurable_criteria": ["npm test exits 0", "npm run lint exits 0"]
  },
  "feedback_sources": [
    { "type": "command", "run": "npm test", "interpret": "exit-code" },
    { "type": "command", "run": "npm run lint", "interpret": "exit-code" }
  ],
  "termination": { "max_ticks": 20, "goal_satisfied": true, "max_no_progress_ticks": 2 },
  "escalation_points": [{ "type": "threshold", "value": 3 }],
  "cadence": { "mode": "background", "schedule": "interval:30m" },
  "evolution_name": "quality-gate"
}
EOF

/loop-define --from loop.json
/loop-tick quality-gate
```

For structured development work, `/create-native-agent` scaffolds a complete agent and the KBD orchestrator handles the full lifecycle. Claim the writer lease for the active harness before mutations:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "$PROJECT_ROOT" claim
```

Claude Code/Claude Desktop uses `claude-code`; other stable IDs are `codex`,
`opencode`, and `kimi`. A valid token does not override a paused, blocked, or
terminal lifecycle.

## Platform-specific quick starts

| Platform | Command |
|---|---|
| Claude Code | `bash scripts/install-skills-flat.sh` (or `npm run install:user`) |
| Kimi Code | `bash scripts/install-skills-flat.sh` — skills load from `~/.kimi-code/skills/` |
| MiniMax | `bash scripts/install-skills-flat.sh` — skills + `_meta.json` in `~/.minimax/skills/` |
| OpenCode | `npm run install:platforms -- --platform opencode` (full plugin) |
| Codex | `bash scripts/install-skills-flat.sh` — skills to `~/.codex/skills/` |
| Cursor / Windsurf / others | `bash scripts/install-skills-flat.sh` |

## When a service is missing

Learning and memory features degrade gracefully: memory features queue or
no-op when surreal-memory is unreachable, and `pk focus` does nothing when
`pk` is absent. The KBD adapter is observational: when it cannot reach the
control plane it queues the event and exits successfully rather than blocking a
tool call. See [KBD troubleshooting](/docs/kbd/troubleshooting).

---

*Previous: [← 18 · Plugins & Marketplace](18-plugins-and-marketplace.md) · Next: [20 · Updating →](20-updating.md)*
