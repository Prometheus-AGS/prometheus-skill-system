# Native Agent Generate Phase

## Role

You are the Generate Phase Controller for the native-agent skill. Your job is to
render all project files from the Tera templates using the spec produced by the
Specify phase, write them to the output directory, then validate the generated
project and optionally trigger Docker build/start.

---

## File Generation Order

Generate files in this exact order to avoid dependency issues:

### 1. Project root

| Template | Output path |
|---|---|
| `project/env_example.tera` | `<output_dir>/.env.example` |
| `project/agent.toml.tera` | `<output_dir>/agent.toml` |
| `project/system_prompt.md.tera` | `<output_dir>/system_prompt.md` |

### 2. Workspace Cargo.toml

| Template | Output path |
|---|---|
| `rust/workspace.cargo.toml.tera` | `<output_dir>/Cargo.toml` |

### 3. agent-core crate

Create directory: `<output_dir>/crates/agent-core/src/`

Write `Cargo.toml`:
```toml
[package]
name        = "agent-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
serde       = { workspace = true }
serde_json  = { workspace = true }
uuid        = { workspace = true }
chrono      = { workspace = true }
```

Write `src/lib.rs` from: `rust/agent_core.rs.tera`

### 4. agent-skills crate

Create directory: `<output_dir>/crates/agent-skills/src/`

Write `Cargo.toml`:
```toml
[package]
name    = "agent-skills"
version.workspace = true
edition.workspace = true

[dependencies]
agent-core  = { workspace = true }
anyhow      = { workspace = true }
walkdir     = { workspace = true }
tracing     = { workspace = true }
shellexpand = "3"
```

Write `src/lib.rs` from: `rust/agent_skills.rs.tera`

### 5. agent-mcp crate

Create directory: `<output_dir>/crates/agent-mcp/src/`

Write `Cargo.toml`:
```toml
[package]
name    = "agent-mcp"
version.workspace = true
edition.workspace = true

[dependencies]
agent-core  = { workspace = true }
anyhow      = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
reqwest     = { workspace = true }
tracing     = { workspace = true }
tokio       = { workspace = true }
```

Write `src/lib.rs` from: `rust/agent_mcp.rs.tera`

### 6. agent-server crate

Create directory: `<output_dir>/crates/agent-server/src/`

Write `Cargo.toml`:
```toml
[package]
name    = "agent-server"
version.workspace = true
edition.workspace = true

[dependencies]
agent-core    = { workspace = true }
agent-skills  = { workspace = true }
agent-mcp     = { workspace = true }
axum          = { workspace = true }
axum-extra    = { workspace = true }
tower         = { workspace = true }
tower-http    = { workspace = true }
tokio         = { workspace = true }
tokio-stream  = { workspace = true }
futures       = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
parking_lot   = "0.12"
reqwest       = { workspace = true }
uuid          = { workspace = true }
tracing       = { workspace = true }
async-stream  = "0.3"
```

Write `src/lib.rs` from: `rust/agent_server.rs.tera`

### 7. agent-cli crate (binary)

Create directory: `<output_dir>/crates/agent-cli/src/`

Write `Cargo.toml`:
```toml
[package]
name    = "agent-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "<agent_bin_name>"
path = "src/main.rs"

[dependencies]
agent-core    = { workspace = true }
agent-skills  = { workspace = true }
agent-mcp     = { workspace = true }
agent-server  = { workspace = true }
clap          = { workspace = true }
tokio         = { workspace = true }
anyhow        = { workspace = true }
toml          = { workspace = true }
serde_json    = { workspace = true }
serde         = { workspace = true }
tracing       = { workspace = true }
tracing-subscriber = { workspace = true }
parking_lot   = "0.12"
reqwest       = { workspace = true }
uuid          = { workspace = true }
shellexpand   = "3"
tikv-jemallocator = { workspace = true }
```

Write `src/main.rs` from: `rust/agent_cli.rs.tera`

Write `src/docker.rs` from: `docker/docker_commands.rs.tera`
(Add `mod docker;` and wire `DockerAction` into the `Command` enum in `main.rs`)

### 8. Frontend

Create directory: `<output_dir>/frontend/src/components/`
Create directory: `<output_dir>/frontend/src/lib/`

Write from templates:
- `frontend/package.json.tera` → `<output_dir>/frontend/package.json`
- `frontend/Chat.tsx.tera` → `<output_dir>/frontend/src/components/Chat.tsx`
- `frontend/ProviderConfig.tsx.tera` → `<output_dir>/frontend/src/components/ProviderConfig.tsx`
- `frontend/api.ts.tera` → `<output_dir>/frontend/src/lib/api.ts`

Generate `<output_dir>/frontend/src/main.tsx`, `vite.config.ts`, and `index.html` inline
(see SKILL.md for content).

### 9. Docker files (if `enable_docker = true`)

| Template | Output path |
|---|---|
| `docker/Dockerfile.tera` | `<output_dir>/Dockerfile` |
| `docker/docker-compose.yml.tera` | `<output_dir>/docker-compose.yml` |
| `docker/dockerignore.tera` | `<output_dir>/.dockerignore` |
| `docker/docker_detect.sh.tera` | `<output_dir>/docker-detect.sh` |

After writing `docker-detect.sh`, make it executable:
```bash
chmod +x <output_dir>/docker-detect.sh
```

Add a `SKILL_PACK_DIR` entry to `.env.example`:
```
# Path to Prometheus skill pack skills/ dir (mounted into Docker container)
SKILL_PACK_DIR=~/.prometheus/skill-pack/skills
```

### 10. .gitignore

```gitignore
target/
node_modules/
frontend/dist/
.env
.agent.pid
.agent.log
.agent/
```

### 11. Documentation

Generate `<output_dir>/README.md` including:
- Agent description and quick start
- Native (non-Docker) run instructions
- Docker run instructions (if `enable_docker = true`)
- Agent network wiring example
- CLI reference summary

Generate `<output_dir>/CLAUDE.md` with development guidelines.

---

## Post-Generation Validation

After writing all files:

1. **Rust check**: `cargo check --workspace --manifest-path <output_dir>/Cargo.toml`
2. **Frontend deps**: `npm install --prefix <output_dir>/frontend --silent`
3. **Docker validate** (if `enable_docker = true`):
   ```bash
   docker buildx ls 2>/dev/null || true   # not an error if missing
   ```

If `cargo check` fails, print the errors clearly and note which generated file needs editing.

## Producer-Model Guard (required before any adversarial review)

`cargo check` proves the workspace compiles; it says nothing about whether the
agent is any good. The adversarial review that judges that can only make its
judge≠producer guarantee if the producer's identity is known, so source the
shared resolver and call the guard **before** building a review packet:

```bash
# Portable across harnesses: repo-relative, then Claude Code, then Codex.
for _lib in \
  "$(cd "$(dirname "$0")" && pwd)/../../../../shared/scripts/lib/kbd-model-resolve.sh" \
  "${CLAUDE_PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh" \
  "${PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh"; do
  [ -f "$_lib" ] && { . "$_lib"; break; }
done

kbd_require_producer_model || exit 2   # exit 2, no packet, no findings file
```

A non-zero return is **fatal** — do not log it and continue. The generated
workspace persists on disk either way; what the guard withholds is the review and
the readiness declaration, not the work. Export the real value —
`export KBD_PRODUCER_MODEL="claude-opus-5"` — and never a `:-default`, which
would fabricate the identity rather than supply it.

---

## Post-Generation Docker Actions

These run AFTER validation, only if the user confirmed them in Specify:

### If `docker_build_now = true`

```bash
cd <output_dir>

# 1. Build the React frontend
npm --prefix frontend run build

# 2. Build the Docker image with --load (loads into local daemon)
docker build --load -t <image_tag> .

# Report result
echo "✅ Image <image_tag> built and loaded into Docker Desktop"
```

### If `compose_up_now = true` (only if `docker_build_now` succeeded or image already exists)

```bash
cd <output_dir>

# Copy .env.example to .env if .env doesn't exist yet
[ -f .env ] || cp .env.example .env

docker compose up --detach

echo "✅ Services started"
echo "   Chat:   http://localhost:<port>"
echo "   Logs:   <agent_name> docker logs -f"
echo "   Stop:   <agent_name> docker down"
```

If `compose_up_now = true` but API keys are missing from `.env`, print:
```
⚠️  Compose started but API keys not set in .env.
    Edit .env and add your ANTHROPIC_API_KEY, then:
    <agent_name> docker down && <agent_name> docker up --detach
```

---

## Adversarial Review (`--mode agent`)

Runs **after** `cargo check`, `npm install`, and any Docker actions above, and
**before** the Success Output banner below.

`cargo check` proves the workspace compiles. It says nothing about whether the
agent is wired to do the job that was asked for — whether the system prompt
matches the configured tools, whether a required MCP server is `enabled = false`,
whether the thing that builds is the thing that was requested. That is what this
step judges, using a model that did not generate the workspace.

The Producer-Model Guard above must already have passed.

```bash
ADV="${CLAUDE_PLUGIN_ROOT}/skills/process/adversarial-review"
REVIEW_DIR="<output_dir>/.review"
mkdir -p "$REVIEW_DIR"

# Record the build verdict where the packet builder will find it, so the judge
# sees the real result instead of "cargo check not run".
( cd "<output_dir>" && cargo check --workspace --message-format short 2>&1 | tail -40 ) \
  > "<output_dir>/.cargo-check.txt" || true

# Manifest-level packet: agent.toml, system_prompt.md, workspace members with
# per-crate purpose, mcp_servers, cargo check result, original intent.
# --intent is what the agent was ASKED to be; without it the judge can only
# check internal consistency, never fitness for purpose.
bash "$ADV/scripts/build-review-packet.sh" \
  --mode agent \
  --target "<output_dir>" \
  --intent "<path to the Specify-phase spec>" \
  --out "$REVIEW_DIR/packet.json" || exit 2

bash "$ADV/scripts/dispatch-judge.sh" \
  --mode agent \
  --packet "$REVIEW_DIR/packet.json" \
  --out "$REVIEW_DIR/findings.json"
```

Read `verdict` and `cross_model_check` from `findings.json`:

| Field | Meaning for this step |
|---|---|
| `verdict: BLOCK` | at least one CRITICAL finding — enter the retry loop below |
| `verdict: PASS` | no CRITICAL findings — print the Success Output banner |
| `cross_model_check: verified-distinct` | the judge provably differed from the producer |
| `cross_model_check: same-model-collision` | the judge WAS the producer — the review proves nothing; treat as unreviewed and say so |

A `PASS` carrying `same-model-collision` is **not** a passing review. Report the
workspace as unreviewed rather than declaring it ready.

## CRITICAL Retry Loop (max 2 rounds)

Identical to the skill creator's loop — the bound lives in one script so the two
creators cannot drift apart on how long they retry:

```bash
STATE="$(bash "$ADV/scripts/review-retry-loop.sh" state \
           --findings "$REVIEW_DIR/findings.json" --round "$ROUND")"

case "$STATE" in
  PROCEED)  # no CRITICAL findings — print the Success Output banner
            ;;
  RETRY)    # fix every CRITICAL finding, ROUND=$((ROUND+1)), re-review
            ;;
  CAPPED)   # do NOT print the banner; the workspace is not ready
            bash "$ADV/scripts/review-retry-loop.sh" unresolved \
              --findings "$REVIEW_DIR/findings.json" --round "$ROUND" \
              --out "<output_dir>/REVIEW-FINDINGS.md"
            ;;
esac
```

`state` exits `0` PROCEED / `3` RETRY / `4` CAPPED. A malformed `findings.json`
yields `CAPPED`, never `PROCEED` — a review that cannot be parsed is not a review
that passed.

### What blocking does and does not mean

The review blocks **the readiness declaration only**. The generated workspace
**always persists on disk**, whatever the verdict:

- Never delete, revert, or refuse to write the workspace because of findings.
- On `CAPPED`, write `REVIEW-FINDINGS.md` into the output directory and print
  the Blocked Output below instead of the Success banner.
- The operator inspects and repairs what was flagged. Discarding the work would
  destroy the very thing the findings describe.

## Blocked Output

When the retry loop returns `CAPPED`, print this **instead of** the Success
Output banner:

```
⚠️  Native agent '<agent_name>' generated in ./<agent_name>/ — NOT declared ready

    The adversarial review reported CRITICAL findings that survived 2 rounds.
    The workspace is on disk and intact; it has not been declared ready.

    Findings:  ./<agent_name>/REVIEW-FINDINGS.md
    Packet:    ./<agent_name>/.review/packet.json

    Review the findings, fix what they name, and re-run the review.
```

## Success Output

Print on completion **only when the retry loop returned `PROCEED`**:

```
✅ Native agent '<agent_name>' generated in ./<agent_name>/

─── Native run ─────────────────────────────────────────
  cd <agent_name>
  cp .env.example .env         # add your API keys
  cargo build --release -p agent-cli
  npm --prefix frontend run build
  ./<agent_name> start         # http://localhost:<port>

─── Docker run ─────────────────────────────────────────  [if enable_docker]
  cd <agent_name>
  cp .env.example .env         # add your API keys
  <agent_name> docker build    # builds + loads into Docker Desktop
  <agent_name> docker up -d    # starts all services

─── Docker Desktop ──────────────────────────────────────  [if docker_desktop]
  Image is available in Docker Desktop → Images → <image_tag>
  Start from there or with: <agent_name> docker up -d

─── A2A discovery ───────────────────────────────────────
  Agent card: http://localhost:<port>/.well-known/agent.json
  AG-UI run:  http://localhost:<port>/agui/run

─── Manage ──────────────────────────────────────────────
  <agent_name> status
  <agent_name> providers list
  <agent_name> mcp list
  <agent_name> skills reload
  <agent_name> docker detect   [if enable_docker]
```

---

## Rules

- Generate all files before running any validation
- Generate Docker files BEFORE post-generation Docker actions
- Use the exact `agent_bin_name` for the binary name (underscores, not hyphens)
- Never hardcode API keys — always use `key_env` pointing to env var names
- If Docker build fails, do not abort — print the error and continue to print next steps
- `docker_build_now` and `compose_up_now` are best-effort — failures are warnings, not errors
- **The generated workspace always persists.** A blocking adversarial review
  withholds the readiness *declaration*, never the work: never delete, revert, or
  refuse to write the workspace because of review findings. The operator cannot
  repair what was discarded, and the findings describe files that must still exist
  to be fixed.
- On a blocked review, write `REVIEW-FINDINGS.md` into the output directory and
  print the Blocked Output instead of the Success banner — never the Success
  banner with warnings appended, which reads as ready.
