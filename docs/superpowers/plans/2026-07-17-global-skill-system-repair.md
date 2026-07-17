# Global Skill System and MCP Substrate Repair Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore a clean, globally installed Prometheus skill system on this Mac, establish the Karpathy learning loop and LLM wiki as the first operational repair gate, rebuild every core binary from the current pinned source, converge LaunchAgents on one canonical owner per service, eliminate Codex MCP startup failures, and leave a `kbd-doctor` skill plus an automated `prometheus doctor` repair/refresh command that can prevent recurrence.

**Architecture:** Treat this repository and its pinned submodule SHAs as the source of truth. After the security snapshot, bootstrap and prove the smallest viable `pk`/Forge/memory/Karpathy learning substrate before broad repairs, then use it to retrieve prior context and write back lessons throughout execution. Build Rust binaries serially, install them into deterministic paths, render client configuration from one service manifest, and manage long-running daemons through canonical `ai.prometheus.*` LaunchAgents. Keep the broad cowork skill warehouse separate from Codex's runtime catalog so Codex loads a deliberate, validated set of real directories rather than recursively ingesting every fixture and nested bundle. Extend the existing `prometheus doctor` command rather than adding a duplicate implementation, and make the `kbd-doctor` skill a thin orchestration layer over that canonical CLI.

**Tech Stack:** Bash 3.2-compatible installers, macOS `launchd`, Rust/Cargo, Codex TOML MCP configuration, JSON-RPC/MCP over stdio and Streamable HTTP, Node/npm for external MCP packages, cowork global skill management, and the Prometheus `pk`, `forge`, `liter-llm`, `surreal-memory-server`, `surface-bridge`, and `sovereign-sync` tools.

---

## Current Failure Map

The implementation must address these observed failures rather than treating the startup warning as one generic outage:

1. `surreal-memory` is registered in Codex at the legacy SSE URL, while current Codex expects Streamable HTTP for URL-based MCP servers. The server's current endpoint is `/mcp/http`.
2. `forge-rs` requires bearer authentication, but the Codex entry has no authorization header. The empty `401` response is surfaced as a missing-content-type startup error.
3. `liter-llm` is launched without its required `--config ~/.config/liter-llm/liter-llm-proxy.toml` argument and exits during initialization.
4. `template-forge-mcp` implements `tools/list` and `tools/call` but not the MCP `initialize` handshake or MCP-compliant tool result envelopes.
5. `BrowserClaw` is enabled at `127.0.0.1:9010`, but no managed service owns that port.
6. `sequential-thinking` and `tavily` are floating `npx` processes that close during startup; versions, Node compatibility, API-key presence, and startup timeouts are not controlled.
7. `com.prometheusags.surface-bridge` and `ai.prometheus.surface-bridge` both attempt to own port `7890`; the legacy label currently wins while the canonical label crash-loops.
8. `sovereign-sync` runs under a legacy label and is omitted from the canonical service installer and health table.
9. Installed copies of `surreal-memory-server`, `liter-llm`, `pk`, `pk-cherry`, `forge`, and `sycophancy-correction` have different hashes from the current source builds.
10. `~/.codex/skills` points at the broad cowork warehouse, exposing 592 recursive `SKILL.md` entries and two intentionally invalid `skill-tester` fixture files to Codex discovery.
11. The MCP configurator only appends missing sections. It cannot repair an existing stale section, so rerunning it preserves broken URLs and arguments.

## Definition of “Latest”

For the repair pass, “latest” means the code pinned by the current `main` branch after `git pull --ff-only` and `git submodule update --init --recursive`. Do **not** run `git submodule update --remote` during recovery; that would silently move dependency pins beyond the reviewed parent commit. Upgrading submodules beyond their pinned SHAs is a separate change requiring review and explicit approval.

## Continuous Learning Protocol During Execution

The Karpathy/wiki substrate is not a final smoke test; it is execution infrastructure for this repair. After Task 2 passes:

1. At the start of every subsequent task, run `pk focus "global skill system repair <task name>" --no-cache` and review relevant project/global memory before editing.
2. At each task completion, ingest a concise Delta → Root Cause → Corrective Actions note with a unique repair tag, then write the corresponding project/global memory through `shared/scripts/lib/memory-bridge.sh` or the first available memory backend.
3. Record failed approaches and stale-install fingerprints, not only successful outcomes, so `kbd-doctor --refresh` can learn the recurring failure signatures.
4. If `pk` or memory becomes unavailable later, stop broad execution, restore the Task 2 baseline, and only then continue.

---

### Task 1: Contain exposed credentials and capture a rollback snapshot

**Files:**
- Read: `~/.codex/config.toml`
- Read: `~/.claude/mcp.json`
- Read: `~/Library/LaunchAgents/*.plist`
- Create: `~/.prometheus/backups/2026-07-17-skill-system/`
- Create: `~/.prometheus/secrets/forge-mcp-token`

- [x] **Step 1: Rotate credentials exposed during diagnosis**

Rotate the Resend and Firecrawl API keys that appeared in terminal output. Rotate the current Forge local bearer token as part of the same repair, even though Forge is loopback-only. Do not paste replacement values into chat, logs, source files, or the plan.

- [x] **Step 2: Create a private machine-state snapshot**

Run:

```bash
umask 077
BACKUP="$HOME/.prometheus/backups/2026-07-17-skill-system"
mkdir -p "$BACKUP/config" "$BACKUP/launchagents" "$BACKUP/state"
cp -p "$HOME/.codex/config.toml" "$BACKUP/config/codex-config.toml"
[ ! -f "$HOME/.claude/mcp.json" ] || cp -p "$HOME/.claude/mcp.json" "$BACKUP/config/claude-mcp.json"
cp -p "$HOME/Library/LaunchAgents"/*prometheus*.plist "$BACKUP/launchagents/" 2>/dev/null || true
launchctl print "gui/$(id -u)" > "$BACKUP/state/launchctl-gui.txt"
lsof -nP -iTCP -sTCP:LISTEN > "$BACKUP/state/listeners.txt"
```

Expected: the backup directory is mode `700` or stricter, files are not group/world-readable, and no command prints secret values.

- [x] **Step 3: Preserve untracked submodule state before synchronization**

Run:

```bash
git status --short --branch > "$BACKUP/state/git-status.txt"
git submodule status --recursive > "$BACKUP/state/submodule-status.txt"
git -C tools/surreal-memory-server status --short > "$BACKUP/state/surreal-memory-status.txt"
[ ! -d tools/surreal-memory-server/.prometheus ] || \
  tar -C tools/surreal-memory-server -czf "$BACKUP/state/surreal-memory-prometheus.tgz" .prometheus
```

Expected: the existing `tools/surreal-memory-server/.prometheus/` state is recoverable and is never deleted by a cleanup command.

- [x] **Step 4: Record installed binary provenance**

Run:

```bash
for bin in prometheus forge pk pk-cherry liter-llm surreal-memory-server \
  sycophancy-correction template-forge template-forge-mcp surface-bridge sovereign-sync; do
  path="$(command -v "$bin" 2>/dev/null || true)"
  printf '%-28s %s\n' "$bin" "${path:-missing}"
  [ -z "$path" ] || shasum -a 256 "$path"
done > "$BACKUP/state/installed-binaries.txt"
```

Expected: the snapshot identifies PATH shadowing before any file is replaced.

---

### Task 2: Establish the Karpathy wiki and learning loops as a hard execution gate

**Files:**
- Read: `shared/scripts/forge-reflect-on-stop.sh`
- Read: `shared/scripts/pk-focus-on-prompt.sh`
- Read: `shared/scripts/pk-health.sh`
- Read: `shared/scripts/lib/memory-bridge.sh`
- Read: `shared/scripts/memory-writeback.sh`
- Test: `shared/scripts/tests/test-pk-health.sh`
- Test: `shared/scripts/tests/test-memory-bridge.sh`
- Test: `shared/scripts/tests/test-memory-writeback.sh`
- Test: `shared/scripts/tests/test-sycophancy-gate-e2e.sh`
- Test: `shared/scripts/tests/test-pipeline-smoke.sh`
- Create at runtime: `~/.prometheus/repair/karpathy-ready.json`

This is the first operational repair after the security/rollback snapshot. Do not begin checkout normalization, installer changes, catalog migration, broad service repair, or MCP rewrites until this task is green. Repairs in this task must be limited to the minimum `pk`/pk-cherry, Forge, memory, sycophancy, and hook dependencies needed to establish a learning substrate.

- [x] **Step 1: Capture a read-only learning-substrate baseline**

Record installed paths, versions, hashes, LaunchAgent labels, listening ports, and current health for `pk`, `pk-cherry`, `forge`, `surreal-memory-server`, and `sycophancy-correction`. Preserve all output under the Task 1 backup directory. Do not report green merely because a command exists; every required command must complete a real read/write/read-back probe.

- [ ] **Step 2: Prove the LLM wiki CLI and MCP path**

Create a uniquely tagged disposable note and run:

```bash
mkdir -p /tmp/prometheus-skill-system-learning-gate
cat > /tmp/prometheus-skill-system-learning-gate/gate-note.md <<'EOF'
# Global Skill Repair Learning Gate

Tag: 2026-07-17-karpathy-learning-gate
EOF
pk ingest /tmp/prometheus-skill-system-learning-gate/gate-note.md \
  --source installation-repair-gate --scope project --yes
pk search 2026-07-17-karpathy-learning-gate
pk focus "2026-07-17 karpathy learning gate" --no-cache
pk lint
```

Then initialize the pk-cherry MCP endpoint, call `tools/list`, and execute one read-only knowledge query. Expected: the unique tag is retrievable through both CLI and MCP, and `pk lint` does not expose a blocking integrity error.

- [ ] **Step 3: Prove memory and writeback semantics**

Write and retrieve one project-scoped and one global-scoped disposable memory through `shared/scripts/lib/memory-bridge.sh` or the first available backend. Verify project/global isolation and confirm the stop/writeback path stores a structured Delta → Root Cause → Corrective Actions note.

- [ ] **Step 4: Prove prompt, stop, Forge, and sycophancy hooks**

Invoke the installed prompt and stop hooks with disposable payloads and unique session IDs. Verify:

- `pk-focus-on-prompt.sh` returns focused context without blocking;
- `forge-reflect-on-stop.sh` calls `forge reflect` when a disposable `.forge/iterations` fixture exists;
- the stop hook falls back to `pk ingest` when no Forge iteration exists;
- `sycophancy-correction` initializes and processes the reflector path;
- missing optional services degrade explicitly without masking required failures.

- [ ] **Step 5: Run the existing learning-loop regression suite**

Run:

```bash
bash shared/scripts/tests/test-pk-health.sh
bash shared/scripts/tests/test-memory-bridge.sh
bash shared/scripts/tests/test-memory-writeback.sh
bash shared/scripts/tests/test-sycophancy-gate-e2e.sh
bash shared/scripts/tests/test-pipeline-smoke.sh
```

Expected: all five tests pass. A skipped required probe is not a pass.

- [ ] **Step 6: Apply only minimal bootstrap repairs if the gate fails**

If an installed learning binary is stale or missing, build and install only the required pinned source for `tools/prometheus-knowledge`, `tools/forge-rs`, `tools/surreal-memory-server`, and sycophancy support, serially. Restart only their directly owned LaunchAgents, rerun Steps 2–5, and record every change in the rollback snapshot. Do not yet reconcile unrelated MCP servers, skill catalogs, BrowserClaw, template-forge, or other LaunchAgents.

- [ ] **Step 7: Write and enforce the readiness artifact**

Create `~/.prometheus/repair/karpathy-ready.json` with the exact source SHAs, installed hashes, test results, endpoint health, timestamp, and unique wiki/memory probe IDs. The artifact must set `ready: true` only when all required probes pass. Every later task must verify this artifact and run `pk focus` before work; if the substrate regresses, return to this task before proceeding.

---

### Task 3: Normalize the checkout and initialize every pinned submodule

**Files:**
- Read: `.gitmodules`
- Read: all submodule worktrees under `tools/` and `skills/imported/`
- Preserve: `tools/surreal-memory-server/.prometheus/`

- [ ] **Step 1: Fast-forward the parent repository only**

Run:

```bash
git pull --ff-only
git status --short --branch
```

Expected: `main` matches `origin/main`; any local or untracked changes are still visible and untouched.

- [ ] **Step 2: Synchronize URLs and initialize pinned commits**

Run:

```bash
git submodule sync --recursive
git submodule update --init --recursive --jobs 1
git submodule status --recursive
```

Expected: `tools/cowork-skills` and `tools/disk-space-guardian` no longer have a leading `-`; no submodule is advanced beyond the SHA pinned by the parent repository.

- [ ] **Step 3: Verify source worktrees before editing**

Run:

```bash
git status --short --branch
git submodule foreach --recursive 'printf "\n[%s]\n" "$name"; git status --short'
```

Expected: only previously known local state is dirty. Stop and review if synchronization introduces unexpected modifications.

---

### Task 4: Create the kbd-doctor skill and upgrade the existing doctor command

**Files:**
- Create: `skills/process/kbd-process-orchestrator/skills/kbd-doctor/SKILL.md`
- Create: `skills/process/kbd-process-orchestrator/skills/kbd-doctor/references/check-catalog.md`
- Create: `skills/process/kbd-process-orchestrator/skills/kbd-doctor/references/repair-policy.md`
- Create if needed: `skills/process/kbd-process-orchestrator/skills/kbd-doctor/scripts/kbd-doctor.sh`
- Modify: `tools/prometheus-cli/crates/prometheus-cli/src/main.rs`
- Modify: `tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs`
- Create/test: `tools/prometheus-cli/crates/prometheus-cli/tests/doctor.rs`
- Modify: `package.json`
- Modify: `scripts/update-skill-pack.sh`
- Modify: `docs/future-work/03-cross-cutting/XC-004-prometheus-doctor-loop-test.md`
- Modify: `docs/guide/13-tools-reference.md`
- Modify: `docs/guide/19-installation.md`

Discovery found no existing `kbd-doctor` skill. The `prometheus doctor` CLI subcommand already exists, so this task must update that command rather than add a second implementation. The current command is read-only and can print `Surreal-memory... unreachable` followed by `All checks passed`; that false-green behavior is a required regression case.

- [ ] **Step 1: Add failing doctor behavior and output-contract tests**

Add deterministic tests for:

- unreachable required memory/knowledge services produce a red result and nonzero exit;
- missing skills directory, invalid runtime skills, duplicate LaunchAgent owners, stale installed hashes, stale MCP URLs/arguments, and unavailable required ports are reported distinctly;
- optional disabled services produce yellow/skipped results, not red startup failures;
- `--json` emits a stable versioned schema without ANSI or secrets;
- `--dry-run --fix` and `--dry-run --refresh` make no filesystem or service changes;
- a second safe repair/refresh pass is idempotent and reports no remaining actions.

Run the narrow CLI tests first and verify the current false-green case fails before implementation.

- [ ] **Step 2: Refactor doctor around a check and repair registry**

Introduce typed equivalents of `Check`, `CheckResult`, `Severity`, and `RepairAction` so one registry drives human output, JSON output, repair planning, and post-repair verification. Use stable check IDs grouped at least by `learning`, `skills`, `binaries`, `services`, `mcp`, `hooks`, and `state`. Red must exit nonzero; yellow may exit zero only when the condition is explicitly optional or degraded by policy.

- [ ] **Step 3: Implement safe command modes on the existing CLI**

Support:

```text
prometheus doctor
prometheus doctor --json
prometheus doctor --check <id-or-group>
prometheus doctor --fix [--dry-run] [--yes]
prometheus doctor --refresh [--dry-run] [--yes]
```

Behavior:

- default mode is read-only diagnosis;
- `--fix` repairs known idempotent configuration, permissions, curated skill sync, canonical LaunchAgent ownership, stale managed MCP sections, and disabled optional-server state;
- `--refresh` compares parent/submodule SHAs, source build hashes, installed hashes, the install manifest, skill catalog state, and LaunchAgent definitions, then rebuilds/reinstalls/reloads only stale managed components;
- every mutating pass creates a private backup, prints or emits a repair plan, applies only approved safe actions, then rescans;
- unresolved red checks remain nonzero.

- [ ] **Step 4: Enforce a deny-by-default repair policy**

Doctor may call existing repository installers/configurators but must not duplicate their shell logic. It may not automatically rotate credentials, expose tokens, overwrite unknown client sections, delete warehouse content, reset dirty submodules, move dependency pins, use `sudo`, or remove unknown LaunchAgents. Such findings must be red/manual actions requiring explicit user resolution. `--yes` suppresses prompts only for actions classified safe and reversible.

- [ ] **Step 5: Reuse canonical repair utilities**

Wire doctor actions to focused modes of:

- `scripts/codex-sync-skills.sh`;
- `scripts/configure-mcp-all-tools.sh`;
- `scripts/install-binaries.sh`;
- `scripts/install-mcp-services.sh`;
- `scripts/check-mcp-health.sh`.

Add focused flags to those scripts only where needed so doctor can repair one check group without running an unconditional full install. Persist a machine-readable install/refresh manifest containing source SHAs, build hashes, installed hashes, service plist hashes, catalog hash, and last successful refresh time; never include credentials.

- [ ] **Step 6: Create the kbd-doctor skill as the orchestration surface**

Use the PMPO skill-creator workflow to create a compliant nested KBD skill. Frontmatter must include `name: kbd-doctor`, a trigger-rich description, and an argument hint equivalent to:

```yaml
argument-hint: "[--json] [--check <id-or-group>] [--fix|--refresh] [--dry-run] [--yes]"
```

The skill must:

- invoke `prometheus doctor` as the source of truth rather than reimplement diagnostics;
- require Task 2's Karpathy readiness check before broad `--fix` or `--refresh` work;
- retrieve prior repair context with `pk focus` before acting;
- write Delta → Root Cause → Corrective Actions back to the wiki/memory after acting;
- explain safe versus manual actions and surface backups/rollback paths.

If a wrapper script is required, keep it thin, Bash 3.2-compatible, and executable.

- [ ] **Step 7: Replace the unconditional npm doctor wrapper and integrate refresh**

Change `npm run doctor` to invoke the compiled CLI in read-only mode. Add explicit `doctor:fix` and `doctor:refresh` scripts using dry-run-safe defaults. Update `scripts/update-skill-pack.sh` so a successful pinned-source update runs or offers `prometheus doctor --refresh --yes`, then rescans; it must not silently rebuild from a dirty checkout or cross a manual safety boundary. A scheduled LaunchAgent may run read-only `--json` health checks, but automatic background rebuilds remain out of scope.

- [ ] **Step 8: Validate the skill, CLI, and documentation**

Run:

```bash
npm run validate:strict skills/process/kbd-process-orchestrator/skills/kbd-doctor
RUSTUP_TOOLCHAIN=stable cargo test --manifest-path tools/prometheus-cli/Cargo.toml doctor
prometheus doctor --json
prometheus doctor --dry-run --fix
prometheus doctor --dry-run --refresh
```

Update XC-004 to record that the command now includes safe auto-repair/refresh requirements, document exit codes and JSON schema, and ensure all new skill scripts are executable. Expected: the false-green memory case is impossible, dry runs are non-mutating, safe actions are idempotent, and the skill delegates to the CLI.

---

### Task 5: Add recovery regression tests before changing installers

**Files:**
- Create: `scripts/tests/test-configure-mcp-all-tools.sh`
- Create: `scripts/tests/test-codex-sync-skills.sh`
- Create: `scripts/tests/test-install-mcp-services.sh`
- Create: `scripts/tests/test-mcp-protocol-smoke.py`
- Modify: `package.json`

- [ ] **Step 1: Write a failing MCP configuration reconciliation test**

Use a temporary `HOME` containing intentionally stale Codex sections. Assert that one repair run:

- rewrites `surreal-memory` to `http://127.0.0.1:23001/mcp/http`;
- preserves unknown user MCP entries;
- writes Forge authentication without printing the token;
- adds the Liter config path;
- can disable optional servers without deleting their user data;
- is idempotent on a second run;
- leaves a TOML file that `python3` can parse with `tomllib`.

Run:

```bash
bash scripts/tests/test-configure-mcp-all-tools.sh
```

Expected initially: FAIL because the current script appends only missing sections.

- [ ] **Step 2: Write a failing Codex catalog isolation test**

Create a temporary cowork warehouse containing a nested invalid fixture and a temporary `CODEX_HOME` whose `skills` path is a symlink to that warehouse. Assert that the sync script:

- archives/replaces the root symlink safely;
- creates a real runtime directory;
- copies only manifest-selected skills;
- never copies `assets/sample-skill/SKILL.md` fixtures;
- preserves non-pack content in the warehouse;
- leaves zero invalid `SKILL.md` files in the Codex runtime root.

Run:

```bash
bash scripts/tests/test-codex-sync-skills.sh
```

Expected initially: FAIL because the current script writes through the root symlink.

- [ ] **Step 3: Write a failing LaunchAgent ownership test**

Test rendered service metadata without loading real daemons. Assert that the installer declares:

- `ai.prometheus.sovereign-sync` on port `7892`;
- a canonical binary placeholder for `sovereign-sync`;
- migration cleanup for `com.prometheusags.surface-bridge` and `com.prometheusags.sovereign-sync`;
- a Forge token placeholder rather than a committed token;
- one health probe per canonical daemon.

Run:

```bash
bash scripts/tests/test-install-mcp-services.sh
```

Expected initially: FAIL because sovereign-sync is absent and legacy cleanup is not implemented.

- [ ] **Step 4: Add a protocol smoke harness**

Implement `scripts/tests/test-mcp-protocol-smoke.py` with two modes:

1. `--stdio <command...>`: send `initialize`, `notifications/initialized`, and `tools/list`; require a valid JSON-RPC response and non-empty tool list.
2. `--http <url>`: POST `initialize` using MCP-compatible `Accept` headers; optionally read a bearer token from a file; require a valid response without logging the token.

- [ ] **Step 5: Register one aggregate recovery test command**

Add to `package.json`:

```json
"test:installation": "bash scripts/tests/test-configure-mcp-all-tools.sh && bash scripts/tests/test-codex-sync-skills.sh && bash scripts/tests/test-install-mcp-services.sh"
```

Run:

```bash
npm run test:installation
```

Expected initially: FAIL for the documented root causes, not for test harness errors.

---

### Task 6: Make MCP configuration declarative, repairable, and secret-safe

**Files:**
- Modify: `scripts/mcp-port-table.json`
- Modify: `scripts/configure-mcp-all-tools.sh`
- Create: `scripts/render-codex-mcp.py`
- Create: `shared/scripts/lib/mcp-secrets.sh`
- Modify: `.mcp.json`
- Modify: `shared/launchagents/ai.prometheus.forge-mcp.plist`
- Modify: `shared/systemd/ai.prometheus.forge-mcp.service`

- [ ] **Step 1: Correct the service manifest**

Update `scripts/mcp-port-table.json`:

- `surreal-memory`: `type: "http"`, URL `http://127.0.0.1:23001/mcp/http`, transport `http`;
- `liter-llm`: add `--config`, `${HOME}/.config/liter-llm/liter-llm-proxy.toml`;
- external `npx` servers: use explicit package versions resolved during execution, not floating latest tags;
- add an `enabledByDefault` field so optional servers can remain installed but disabled;
- mark `BrowserClaw` optional and disabled unless a managed service is installed.

- [ ] **Step 2: Centralize Forge token generation**

Implement `ensure_forge_mcp_token` in `shared/scripts/lib/mcp-secrets.sh`:

```bash
~/.prometheus/secrets/forge-mcp-token
```

Requirements:

- generate once with `openssl rand -hex 24` or `python3 secrets.token_hex(24)`;
- set directory mode `700` and file mode `600`;
- never echo the token;
- reuse the same token across service and client renders;
- rotate only when an explicit `--rotate-forge-token` flag is supplied.

- [ ] **Step 3: Remove the committed Forge token from service templates**

Replace the literal value in both macOS and Linux templates with `__FORGE_MCP_TOKEN__`. Extend `scripts/install-mcp-services.sh` rendering to substitute the private token at install time.

- [ ] **Step 4: Replace append-only Codex updates with targeted reconciliation**

Implement `scripts/render-codex-mcp.py` to update only pack-managed MCP table names while preserving every unrelated setting and MCP entry. It must remove stale forms of the managed sections before writing canonical sections.

Canonical Codex entries:

```toml
[mcp_servers.surreal-memory]
url = "http://127.0.0.1:23001/mcp/http"

[mcp_servers.prometheus-knowledge]
url = "http://127.0.0.1:8942/mcp"

[mcp_servers.forge-rs]
url = "http://127.0.0.1:8943/mcp"
http_headers = { Authorization = "Bearer <rendered locally>" }

[mcp_servers.liter-llm]
command = "liter-llm"
args = ["mcp", "--transport", "stdio", "--config", "~/.config/liter-llm/liter-llm-proxy.toml"]
startup_timeout_sec = 30
```

Write the resulting `~/.codex/config.toml` with mode `600`. Never print the rendered Forge header.

- [ ] **Step 5: Reconcile JSON clients from the same source**

Modify `merge_json_mcp` so known Prometheus entries are updated rather than skipped. Resolve `${HOME}` and the Forge token at render time, keep backups, and preserve non-Prometheus entries.

- [ ] **Step 6: Make web MCP credentials optional and external**

Remove the unconditional `TAVILY_API_KEY` requirement from the top of the configurator. Configure Tavily and Firecrawl only when their environment variables are present; otherwise write them disabled or skip them with a clear message. Never persist newly rotated keys in repository files.

- [ ] **Step 7: Run the configuration tests**

Run:

```bash
npm run test:installation
python3 -m py_compile scripts/render-codex-mcp.py scripts/tests/test-mcp-protocol-smoke.py
```

Expected: the MCP configuration test passes; the remaining service/catalog tests may still fail until later tasks.

---

### Task 7: Make template-forge a compliant MCP stdio server

**Files:**
- Modify: `skills/imported/artifact-refiner/tools/template-forge-rs/crates/template-mcp/src/main.rs`
- Modify if needed: `skills/imported/artifact-refiner/tools/template-forge-rs/crates/template-mcp/Cargo.toml`

- [ ] **Step 1: Add failing unit tests for the MCP handshake**

Add tests covering:

- `initialize` returns `protocolVersion`, `capabilities.tools`, and `serverInfo`;
- `notifications/initialized` produces no response;
- `ping` returns an empty result object;
- `tools/list` remains available after initialization;
- `tools/call` returns an MCP result envelope with `content` and optional `structuredContent`;
- requests with no `id` are treated as notifications and never receive `id: null` responses.

Run:

```bash
cd skills/imported/artifact-refiner/tools/template-forge-rs
RUSTUP_TOOLCHAIN=stable cargo test -p template-forge-mcp
```

Expected initially: FAIL on `initialize` and result-envelope assertions.

- [ ] **Step 2: Implement the handshake and lifecycle methods**

Extend the existing request loop rather than replacing the whole server. Negotiate a supported protocol version, advertise tools, accept `notifications/initialized`, and keep stdout strictly JSON-RPC-only.

- [ ] **Step 3: Wrap tool success and error results correctly**

Successful calls should return text content plus structured JSON. Tool execution errors should return an MCP tool result with `isError: true` when appropriate; JSON-RPC errors remain reserved for invalid requests/methods.

- [ ] **Step 4: Verify stdio behavior with the shared smoke harness**

Run:

```bash
cd skills/imported/artifact-refiner/tools/template-forge-rs
RUSTUP_TOOLCHAIN=stable cargo test -p template-forge-mcp
cd -
python3 scripts/tests/test-mcp-protocol-smoke.py --stdio \
  skills/imported/artifact-refiner/tools/template-forge-rs/target/debug/template-forge-mcp
```

Expected: initialize and `tools/list` succeed without `unknown method 'initialize'`.

- [ ] **Step 5: Preserve the submodule change correctly**

Because artifact-refiner is a submodule, do not lose the local fix during later synchronization. Before any parent-repository commit, obtain explicit approval to commit/push the submodule change and then update the parent SHA. If approval is not given, document the dirty submodule and do not run commands that reset it.

---

### Task 8: Separate the Codex runtime catalog from the cowork warehouse

**Files:**
- Modify: `scripts/codex-sync-skills.sh`
- Modify: `config/codex-catalog.txt`
- Create: `config/codex-external-catalog.example.txt`
- Create at runtime: `~/.config/prometheus/codex-external-catalog.txt`
- Replace at runtime: `~/.codex/skills` symlink with a real directory
- Preserve: `~/.TOOLS/skills/codex`

- [ ] **Step 1: Add safe root-symlink migration**

When `CODEX_SKILLS` itself is a symlink, the script must:

1. resolve and record the warehouse target;
2. refuse to delete or modify that target;
3. archive the symlink metadata in the backup directory;
4. unlink only `~/.codex/skills`;
5. create a real `~/.codex/skills` directory.

The migration must be opt-in with `--migrate-root-symlink` and support `--dry-run`.

- [ ] **Step 2: Add an explicit external-skill allowlist**

Keep pack skills selected by `config/codex-catalog.txt`. Allow selected cowork skills to be copied from the warehouse through `~/.config/prometheus/codex-external-catalog.txt`. Do not default to copying all 305 top-level directories.

Before migration, generate a review file from the current catalog so the user can choose which external skills remain auto-triggerable.

- [ ] **Step 3: Prevent fixtures from becoming runtime skills**

Exclude nested fixture paths matching at least:

```text
*/assets/sample-skill/SKILL.md
*/fixtures/**/SKILL.md
*/testdata/**/SKILL.md
```

Do not “fix” an intentionally invalid test fixture by adding frontmatter. The fixture must remain valid test input while staying outside the Codex discovery root.

- [ ] **Step 4: Add post-sync validation gates**

After every sync, fail if any runtime `SKILL.md`:

- lacks YAML frontmatter delimiters;
- has invalid frontmatter;
- is located under `assets/`, `fixtures/`, or `testdata/`;
- causes the catalog entry count to exceed a configurable budget.

- [ ] **Step 5: Run the catalog test and a dry-run migration**

Run:

```bash
bash scripts/tests/test-codex-sync-skills.sh
bash scripts/codex-sync-skills.sh --migrate-root-symlink --dry-run --report
```

Expected: the dry-run lists preserved external skills, proposed copies, pruned pack-owned entries, and zero warehouse deletions.

- [ ] **Step 6: Perform the runtime migration after review**

Run only after the allowlist is accepted:

```bash
bash scripts/codex-sync-skills.sh --migrate-root-symlink --report
```

Expected: `~/.codex/skills` is a real directory, the cowork warehouse remains intact, and the invalid fixture count is zero.

---

### Task 9: Canonicalize binary and service ownership

**Files:**
- Modify: `scripts/install-binaries.sh`
- Modify: `scripts/install-mcp-services.sh`
- Modify: `scripts/check-mcp-health.sh`
- Create: `shared/launchagents/ai.prometheus.sovereign-sync.plist`
- Create: `shared/systemd/ai.prometheus.sovereign-sync.service`
- Deprecate at runtime: `~/Library/LaunchAgents/com.prometheusags.surface-bridge.plist`
- Deprecate at runtime: `~/Library/LaunchAgents/com.prometheusags.sovereign-sync.plist`

- [ ] **Step 1: Add sovereign-sync to the binary installer**

Build `substrate/sovereign-sync` serially and install `sovereign-sync` to `~/.local/bin`. On macOS, ad-hoc sign the installed binary using the existing `install_bin` helper.

- [ ] **Step 2: Add canonical service templates**

Create `ai.prometheus.sovereign-sync` templates using:

```text
sovereign-sync --mode daemon
```

Use the standard Prometheus log directory, HOME/USER/PATH environment, working directory, and port `7892` health endpoint.

- [ ] **Step 3: Migrate legacy labels before port reuse checks**

The installer currently reuses any process already serving a port, which lets a legacy label permanently block the canonical label. Before probing ports, boot out known legacy labels and archive/remove their plist files:

```text
com.prometheusags.surface-bridge
com.prometheusags.sovereign-sync
```

Only then render, bootstrap, enable, and kickstart canonical labels.

- [ ] **Step 4: Add sovereign-sync to service order and health output**

Place sovereign-sync after its required storage/network dependencies and before the final health summary. Add `http://127.0.0.1:7892/health` to `scripts/check-mcp-health.sh`.

- [ ] **Step 5: Strengthen service health semantics**

Use `/health` endpoints where available. For MCP-only endpoints, send a real initialize request rather than treating arbitrary `404` or `405` responses as healthy. Report three independent fields: service-manager state, listener owner PID, and protocol probe result.

- [ ] **Step 6: Run service installer tests**

Run:

```bash
bash scripts/tests/test-install-mcp-services.sh
npm run test:installation
```

Expected: all installation regression tests pass.

---

### Task 10: Build and install all pinned binaries serially

**Files:**
- Build from: `tools/forge-rs/`
- Build from: `tools/prometheus-knowledge/`
- Build from: `tools/liter-llm/`
- Build from: `tools/surreal-memory-server/`
- Build from: `skills/imported/sycophancy-correction/`
- Build from: `skills/imported/artifact-refiner/tools/template-forge-rs/`
- Build from: `substrate/sovereign-sync/`
- Build from: the source used for `surface-bridge`
- Install to: `~/.local/bin/`
- Also install where required: `/usr/local/bin/`

- [ ] **Step 1: Run focused tests before release builds**

Run the package-specific tests for every modified Rust workspace, one workspace at a time. Do not run concurrent Cargo builds; prior experience shows memory pressure can kill live services and create false outage signals.

- [ ] **Step 2: Run the repository binary installer**

Run:

```bash
bash scripts/install-binaries.sh
```

Expected: no “submodule not initialized” messages for required tools and no fallback downloads for cowork/dsg when their source submodules are present.

- [ ] **Step 3: Resolve PATH shadowing**

For binaries used by LaunchAgents, compare the rendered executable path with `command -v`. Ensure the freshly built `surreal-memory-server` and `sycophancy-correction` are updated in both `~/.local/bin` and `/usr/local/bin` when the service PATH prefers `/usr/local/bin`.

If `/usr/local/bin` is not writable, pause and request approval for the minimal copy/codesign commands rather than silently leaving stale shadow copies.

- [ ] **Step 4: Verify hashes and signatures**

Run:

```bash
for bin in forge pk pk-cherry liter-llm surreal-memory-server \
  sycophancy-correction template-forge template-forge-mcp surface-bridge sovereign-sync; do
  path="$(command -v "$bin")"
  shasum -a 256 "$path"
  codesign --verify --verbose "$path" 2>&1 || true
done
```

Expected: installed hashes match their freshly built artifacts; macOS binaries pass ad-hoc signature verification.

- [ ] **Step 5: Record versions and source SHAs**

Write a sanitized manifest to:

```text
~/.prometheus/install-manifest.json
```

Include parent commit, submodule SHAs, binary paths, versions, hashes, and installation timestamp. Exclude tokens and API keys.

---

### Task 11: Reinstall, restart, and reconcile all MCP clients

**Files:**
- Render: `~/Library/LaunchAgents/ai.prometheus.*.plist`
- Modify through installer: `~/.codex/config.toml`
- Modify through installer: other detected MCP client configs

- [ ] **Step 1: Install canonical LaunchAgents**

Run:

```bash
bash scripts/install-mcp-services.sh
```

Expected: legacy surface and sovereign labels are booted out; canonical labels are loaded exactly once.

- [ ] **Step 2: Restart in dependency order**

Use `launchctl kickstart -k` in this order:

1. `ai.prometheus.surrealdb-native`
2. `ai.prometheus.surreal-memory-native`
3. `ai.prometheus.pk-cherry`
4. `ai.prometheus.forge-mcp`
5. `ai.prometheus.surface-bridge`
6. `ai.prometheus.sovereign-sync`

Wait for each health probe before starting the next service.

- [ ] **Step 3: Render corrected MCP client configurations**

Run:

```bash
bash scripts/configure-mcp-all-tools.sh --tool codex
bash scripts/configure-mcp-all-tools.sh
```

Expected: existing stale sections are corrected, not skipped; backups are created; no credentials are printed.

- [ ] **Step 4: Quarantine absent optional servers**

Set `BrowserClaw` to `enabled = false` unless its package and a managed service on port `9010` are deliberately installed. Apply the same policy to Tavily, Firecrawl, and sequential-thinking when their prerequisite package/key checks fail.

- [ ] **Step 5: Pin and test external stdio servers**

Resolve current package versions during execution:

```bash
npm view @modelcontextprotocol/server-sequential-thinking version
npm view tavily-mcp version
npm view firecrawl-mcp version
```

Record exact versions in `scripts/mcp-port-table.json`. Test each under the active Node runtime with `scripts/tests/test-mcp-protocol-smoke.py`. If a package fails under Node 24, run it through a pinned Node 22 LTS wrapper rather than leaving a floating incompatible command.

- [ ] **Step 6: Confirm Codex sees only intended servers**

Run:

```bash
codex mcp list
```

Expected: every enabled core server is listed with the corrected transport; optional unavailable servers are disabled rather than failing startup.

---

### Task 12: Revalidate the memory, wiki, Forge, and Karpathy loops end to end

This is a full-stack revalidation of the Task 2 learning gate after all binaries, services, MCP registrations, hooks, and runtime catalogs have been reconciled. Compare the results with `~/.prometheus/repair/karpathy-ready.json`; do not treat this as the first proof that the learning substrate works.

**Files:**
- Read/write runtime state under: `~/.prometheus/`
- Use disposable validation input under: `/tmp/prometheus-skill-system-smoke/`
- Do not modify production wiki entries without an explicit test tag

- [ ] **Step 1: Validate all HTTP and stdio MCP handshakes**

Run:

```bash
bash scripts/check-mcp-health.sh
python3 scripts/tests/test-mcp-protocol-smoke.py --http http://127.0.0.1:23001/mcp/http
python3 scripts/tests/test-mcp-protocol-smoke.py --http http://127.0.0.1:8942/mcp
python3 scripts/tests/test-mcp-protocol-smoke.py --http http://127.0.0.1:8943/mcp \
  --bearer-token-file "$HOME/.prometheus/secrets/forge-mcp-token"
python3 scripts/tests/test-mcp-protocol-smoke.py --stdio \
  liter-llm mcp --transport stdio --config "$HOME/.config/liter-llm/liter-llm-proxy.toml"
python3 scripts/tests/test-mcp-protocol-smoke.py --stdio template-forge-mcp
python3 scripts/tests/test-mcp-protocol-smoke.py --stdio sycophancy-correction
```

Expected: initialize and `tools/list` pass for every enabled core MCP server.

- [ ] **Step 2: Validate scoped surreal-memory write/read**

Using the MCP tool surface or a local integration script:

1. add a uniquely tagged project memory;
2. search `user_id="prometheus-skill-pack"` and verify it returns;
3. add/search a uniquely tagged global memory;
4. remove the disposable test records if the server supports deletion, otherwise mark them `type: smoke-test`.

Expected: scoped project and global memory are distinguishable and searchable.

- [ ] **Step 3: Validate the LLM wiki CLI and MCP**

Run against disposable content:

```bash
mkdir -p /tmp/prometheus-skill-system-smoke
printf '# Installation smoke note\n\nTag: 2026-07-17-install-smoke\n' \
  > /tmp/prometheus-skill-system-smoke/wiki-note.md
pk ingest /tmp/prometheus-skill-system-smoke/wiki-note.md \
  --source installation-smoke --scope project --yes
pk search 2026-07-17-install-smoke
pk focus "2026-07-17 installation smoke" --no-cache
pk lint
```

Expected: ingest, search, focus, and lint complete; the pk-cherry MCP exposes the corresponding knowledge tools.

- [ ] **Step 4: Validate Forge CLI and MCP**

Create a disposable, intentionally simple source file and run:

```bash
forge validate /tmp/prometheus-skill-system-smoke/example.rs --language rust
forge --help
```

Then use the Forge MCP `tools/list` and one read-only validation tool call. If testing `forge enrich` or `forge reflect`, use a disposable OpenSpec fixture and a unique iteration ID so production state is not overwritten.

Expected: CLI and authenticated MCP use the same installed binary and can reach pk-cherry.

- [ ] **Step 5: Validate the Karpathy prompt/stop hooks**

Inspect the installed hook registrations and invoke their scripts with a disposable prompt/session payload:

- prompt hook calls `pk focus` and returns context without blocking;
- stop hook calls `forge reflect` when `.forge/iterations` exists;
- stop hook falls back to `pk ingest` when no Forge iteration exists;
- sycophancy correction initializes and handles the reflector path;
- missing optional services degrade gracefully without hiding core failures.

Run the relevant existing tests:

```bash
bash shared/scripts/tests/test-pk-health.sh
bash shared/scripts/tests/test-memory-bridge.sh
bash shared/scripts/tests/test-memory-writeback.sh
bash shared/scripts/tests/test-sycophancy-gate-e2e.sh
bash shared/scripts/tests/test-pipeline-smoke.sh
```

- [ ] **Step 6: Validate KBD and Feynman skill availability**

From a fresh Codex session, confirm these skills are discoverable or invokable:

```text
/kbd-status
/learn-about-system
/feynman-loop
/llm-wiki
/forge-related workflow skills
```

Use status/read-only commands for the smoke pass; do not start a durable learning cycle unless requested.

- [ ] **Step 7: Validate sovereign-sync and UI surface**

Run:

```bash
curl -fsS http://127.0.0.1:7890/health
curl -fsS http://127.0.0.1:7892/health
launchctl print "gui/$(id -u)/ai.prometheus.surface-bridge"
launchctl print "gui/$(id -u)/ai.prometheus.sovereign-sync"
```

Expected: both services are owned by canonical labels; no legacy process owns either port.

- [ ] **Step 8: Start a fresh Codex process and inspect startup**

Expected startup acceptance criteria:

- no “Skipped loading skill(s) due to invalid SKILL.md” warning;
- no failed startup for surreal-memory, Forge, Liter, or template-forge;
- no BrowserClaw warning when BrowserClaw is not installed;
- no duplicate LaunchAgent address-in-use loop;
- skill descriptions remain useful under the measured catalog budget.

---

### Task 13: Final verification, rollback drill, documentation, and memory

**Files:**
- Modify: `docs/guide/05-mcp-substrate.md`
- Modify: `docs/guide/13-tools-reference.md`
- Modify: `docs/guide/19-installation.md`
- Modify: `shared/references/surreal-memory-integration.md`
- Create: `docs/runbooks/global-skill-system-repair.md`
- Create/update mandatory memory using the first available memory backend

- [ ] **Step 1: Run repository validation**

Run:

```bash
npm run test:installation
npm run validate:strict
npm run validate:codex
npm test
bash scripts/smoke-test.sh
prometheus doctor --json
prometheus doctor --dry-run --fix
prometheus doctor --dry-run --refresh
```

Run targeted Rust tests for every modified workspace again. Do not fix unrelated failures; record them separately.

- [ ] **Step 2: Verify the live Codex catalog**

Run:

```bash
bash scripts/codex-sync-skills.sh --report
codex debug prompt-input | python3 scripts/codex-catalog-stat.py
python3 - <<'PY'
from pathlib import Path
root = Path.home() / '.codex' / 'skills'
invalid = []
for path in root.rglob('SKILL.md'):
    text = path.read_text(errors='replace')
    if not text.startswith('---\n'):
        invalid.append(path)
print(f'invalid_count={len(invalid)}')
for path in invalid:
    print(path)
raise SystemExit(1 if invalid else 0)
PY
```

Expected: zero invalid files and a catalog count within the agreed budget.

- [ ] **Step 3: Perform a rollback drill without disrupting the healthy stack**

Validate that backups can restore:

- `~/.codex/config.toml`;
- the previous `~/.codex/skills` symlink or directory;
- legacy/canonical LaunchAgent plist files;
- the previous binary paths from the hash manifest.

Document commands but do not actually roll back the successful installation.

- [ ] **Step 4: Update operational documentation**

Document:

- current MCP endpoints and transport types;
- Forge token file location and rotation procedure;
- canonical LaunchAgent labels and dependency order;
- how Codex runtime catalog isolation differs from the cowork warehouse;
- how to update pinned source safely;
- exact health and protocol smoke commands;
- `kbd-doctor` invocation, check groups, exit codes, JSON schema, repair policy, and backup locations;
- when to use `prometheus doctor --fix` versus `prometheus doctor --refresh`;
- how the post-update refresh path detects stale source/build/install/service/catalog state;
- rollback steps.

- [ ] **Step 5: Write mandatory project and global memories**

Project memory must record exact repository paths, canonical labels, endpoints, and the root causes fixed. Global memory must record reusable lessons:

- URL-based Codex MCP servers use Streamable HTTP, not legacy SSE registration;
- append-only config installers cannot repair stale sections;
- one stable secret must be shared between authenticated local server and clients;
- root skill-directory symlinks can leak an entire warehouse and fixtures into recursive discovery;
- serial Rust builds avoid resource-starvation false outages;
- known legacy service labels must be removed before port-reuse checks;
- health commands must count required-service failures before printing a green summary;
- refresh automation should compare source, build, installed, service, and catalog hashes and only replace stale managed artifacts;
- repair automation must be deny-by-default for credentials, dirty submodules, unknown config, unknown services, deletion, and privilege escalation.

- [ ] **Step 6: Produce the final handoff**

Report:

- parent commit and submodule SHAs;
- installed binary versions/hashes;
- canonical service PIDs and ports;
- enabled/disabled MCP servers and why;
- Codex catalog count and invalid-file count;
- validation commands and results;
- any changes still dirty inside submodules;
- any user action still required, especially credential rotation or upstream submodule commits.

---

## Acceptance Criteria

The repair is complete only when all of the following are true:

1. Task 2 produced `~/.prometheus/repair/karpathy-ready.json` with `ready: true` before broad repair work began, and every later task used the continuous learning protocol.
2. `git submodule status --recursive` has no uninitialized required submodules.
3. The `kbd-doctor` skill exists, validates strictly, is discoverable through the curated Codex catalog, and delegates to the existing `prometheus doctor` command.
4. `prometheus doctor` never emits a green summary for an unreachable required service, provides stable human/JSON output and nonzero red exit codes, and supports non-mutating `--dry-run` for `--fix` and `--refresh`.
5. Doctor safe fixes are reversible and idempotent; refresh replaces only stale managed binaries, skills, MCP sections, and LaunchAgent definitions while unsafe actions remain manual.
6. Installed core binary hashes match current pinned source builds.
7. Exactly one canonical LaunchAgent owns each Prometheus daemon port.
8. SurrealDB `28000`, memory `23001`, pk `8942`, Forge `8943`, surface `7890`, and sovereign-sync `7892` pass health/protocol checks.
9. Codex initializes surreal-memory, pk, Forge, Liter, template-forge, and sycophancy without startup errors.
10. Optional unavailable MCP servers are disabled, not allowed to fail every Codex startup.
11. `~/.codex/skills` is a real curated runtime directory, not a symlink to the full cowork warehouse.
12. Codex reports zero invalid `SKILL.md` files.
13. `pk ingest/search/focus/lint`, authenticated Forge MCP, memory scoped write/read, Karpathy hooks, KBD status, and Feynman/LLM-wiki skill discovery all pass before and after the broad repair.
14. Rotated credentials are no longer present in terminal logs, source-controlled files, or world-readable configuration.
15. A rollback snapshot, install/refresh manifest, runbook, per-task learning notes, and mandatory memories exist.

## Stop Conditions

Pause execution and request user review if any of these occur:

- a backup cannot be created or verified;
- the Task 2 Karpathy/wiki/memory hard gate cannot reach `ready: true` after minimal bootstrap repairs;
- the learning substrate regresses during a later task and cannot be restored to the recorded baseline;
- `git pull --ff-only` cannot fast-forward;
- a submodule contains unknown local changes;
- a required `/usr/local/bin` replacement needs elevated permission;
- the Codex external-skill allowlist would remove a relied-on skill;
- credential rotation has not been completed;
- doctor classifies a required action as unsafe/manual, including credential rotation, dirty-submodule reset, unknown config replacement, unknown service removal, deletion, or privilege escalation;
- the template-forge submodule fix needs an upstream commit/push;
- a canonical service cannot take ownership after legacy labels are booted out;
- tests expose unrelated repository failures that predate this repair.

