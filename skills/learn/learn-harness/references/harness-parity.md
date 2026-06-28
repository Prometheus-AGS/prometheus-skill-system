# Harness Parity Reference

Detailed cross-harness parity for the Prometheus Skill Pack.
Parent skill: `skills/learn/learn-harness/SKILL.md`

---

## 1. Complete Cross-Harness Capability Table

| Capability | Claude Code | OpenCode | Codex | Kimi Code | Zed |
|---|---|---|---|---|---|
| **Skills (agentskills.io format)** | ✓ | ✓ | ✓ | ✓ | ✓ |
| **Skill invocation (/name)** | ✓ | ✓ | ✓ | ✓ | partial¹ |
| **MCP servers (general)** | ✓ | ✓ | ✗ | partial² | ✗ |
| **AskUserQuestion — Tier 1** | ✓ | ✗ | ✗ | ✗ | ✗ |
| **File-pair UI — Tier 1 alt** | ✗ | ✓ | ✓ | ✓ | ✗ |
| **PostToolUse hooks** | ✓ | ✗ | ✗ | ✗ | ✗ |
| **Stop hooks** | ✓ | ✓ | ✗ | ✗ | ✗ |
| **Subagents** | ✓ | ✓ | ✓ | partial³ | ✗ |
| **surreal-memory MCP** | ✓ | ✓ | ✗ | partial² | ✗ |
| **sycophancy-correction MCP** | ✓ | partial⁴ | ✗ | partial² | ✗ |
| **Hooks config format** | hooks.json | config.json Stop only | n/a | n/a | n/a |
| **Plugin/marketplace install** | ✓ | ✓ | ✗ | ✗ | ✗ |
| **Structured subagent output** | ✓ | ✓ | partial | ✗ | ✗ |

### Notes

1. **Zed partial invocation** — Zed's slash command integration varies across
   versions. Skills may need to be triggered by describing the skill name in
   prose rather than a literal `/skill-name` command.

2. **Kimi partial MCP** — Kimi Code supports MCP servers via
   `~/.kimi-code/config.toml`, but compatibility and reliability vary across
   Kimi versions. Some MCP tool calls may fail silently. Validate by running
   `bash shared/scripts/detect-toolchain.sh` and checking the `mcp` section.

3. **Kimi partial subagents** — Kimi Code supports subagent delegation but
   does not propagate full tool sets to subagents in all configurations.
   Complex subagent chains (e.g. feynman-loop → learn-grade) may require
   manual step-by-step invocation.

4. **OpenCode partial sycophancy-correction** — The MCP server loads, but the
   binary must be built separately (`cargo build --release` in
   `skills/imported/sycophancy-correction/`). The Stop hook that gates
   reflection artifacts requires the binary to be present.

---

## 2. Learn Domain Skills Parity

Which learn-* skills work at which tier on each harness.

| Skill | Claude Code | OpenCode | Codex | Kimi Code | Zed |
|---|---|---|---|---|---|
| **learn-plan** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-survey** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **feynman-loop** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-grade** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-practice** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-retain** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-certify** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-kb** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-goal** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **ui-surface** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |
| **learn-harness** | Tier 1 | Tier 1 alt | Tier 0 | Tier 1 alt | Tier 0 |

### Tier definitions

| Tier | Name | Description |
|---|---|---|
| **Tier 0** | Text only | All output is plain markdown in the chat thread. No interactive UI. |
| **Tier 1** | AskUserQuestion | Native interactive prompts via the harness AskUserQuestion tool. |
| **Tier 1 alt** | File-pair | Interactive UI via `__ui_intent__.json` / `__ui_response__.json` file exchange. |

All skills degrade gracefully to Tier 0 when the harness does not support a
higher tier. No skill crashes on an unknown or unsupported harness.

---

## 3. Installation Paths per Harness

The `install-skills-flat.sh` script handles all of the paths below
automatically. Manual paths are listed here for reference.

| Harness | Skill directory | Config file |
|---|---|---|
| **Claude Code** | `~/.claude/skills/` | `~/.claude/settings.json`, `.mcp.json` |
| **OpenCode** | `~/.opencode/skills/` | `~/.opencode/config.json` |
| **Codex** | `~/.codex/skills/` | n/a (no config; skills are self-contained) |
| **Kimi Code** | `~/.kimi-code/skills/` | `~/.kimi-code/config.toml` |
| **Zed** | `~/.zed/skills/` | Zed AI settings (version-dependent) |

### Install command

```bash
# Install to all detected platforms
bash scripts/install-skills-flat.sh

# Uninstall from all platforms
bash scripts/install-skills-flat.sh --uninstall
```

The installer copies (not symlinks) skills for platforms that do not support
symlinks (MiniMax, some Kimi versions). Claude Code and OpenCode receive
symlinks where supported.

---

## 4. MCP Server Setup per Harness

### Claude Code

Add to `.mcp.json` at repo root or `~/.claude/mcp.json` for user scope:

```json
{
  "mcpServers": {
    "surreal-memory": {
      "command": "node",
      "args": ["path/to/surreal-memory/dist/index.js"]
    },
    "sycophancy-correction": {
      "command": "/path/to/sycophancy-correction/target/release/sycophancy-correction",
      "args": ["--mcp"]
    }
  }
}
```

### OpenCode

Add to `~/.opencode/config.json`:

```json
{
  "mcp": {
    "servers": {
      "surreal-memory": {
        "command": ["node", "path/to/surreal-memory/dist/index.js"]
      }
    }
  }
}
```

sycophancy-correction is available but only the Stop hook fires in OpenCode —
PostToolUse is not supported.

### Kimi Code

Add to `~/.kimi-code/config.toml`:

```toml
[[mcp.servers]]
name = "surreal-memory"
command = ["node", "path/to/surreal-memory/dist/index.js"]
```

Verify with `bash shared/scripts/detect-toolchain.sh --json | jq '.mcp'`.

### Codex

No MCP support. Skills must be fully self-contained. Do not reference MCP
tools in Codex-targeted skill paths.

### Zed

No MCP support. Zed's extension model does not expose MCP server integration
as of the Zed versions tested with this skill pack.

---

## 5. Known Limitations and Workarounds

### Claude Code

No known learn-domain limitations. This is the full-capability reference
harness.

### OpenCode

**PostToolUse hooks absent** — The reflector sycophancy gate (PostToolUse on
SubagentStop) does not fire. Workaround: run
`bash shared/scripts/sycophancy-check-reflection.sh` manually after a
reflection artifact is produced, or accept that gate is inactive on OpenCode.

**AskUserQuestion absent** — All Tier 1 UI falls back to the file-pair
convention automatically. No manual workaround needed.

### Codex

**No MCP** — surreal-memory is unavailable. Learner state cannot be persisted
to the knowledge graph automatically. Workaround: use the file-based memory
path (`~/.claude/projects/.../memory/MEMORY.md`) or accept session-only
memory.

**No hooks** — sycophancy gate and all PostToolUse automation are inactive.
No workaround; gate is advisory, not blocking.

**No structured subagent output** — Codex subagents may not return JSON
reliably. Workaround: skills that parse subagent output should add a Tier 0
prose fallback.

### Kimi Code

**Partial MCP reliability** — Test MCP connectivity before running a long
Feynman session:
```bash
bash shared/scripts/detect-toolchain.sh --json | jq '.mcp'
```
If `surreal-memory` shows `"status": "error"`, restart the MCP server before
proceeding.

**Subagent tool propagation** — If a subagent call fails with a tool-not-found
error, break the chain into sequential manual invocations:
1. `/feynman-loop --concept-id X --goal-id Y --depth 0`
2. Wait for completion, then: `/learn-grade --artifact <path>`

### Zed

**Tier 0 only** — All interactive UI is unavailable. Every learn skill falls
back to printing questions and collecting answers as plain text in the chat
thread. The Feynman loop works but requires the user to respond to each step
in the chat rather than via a rendered form.

**No memory persistence** — Without surreal-memory or a file-based hook, no
learner state is saved between Zed sessions. Workaround: export the learner
model to a file at the end of each session:
```bash
bash shared/scripts/export-learner-model.sh --goal-id <id> --out ./learner-export.json
```
Then import at the start of the next session.

**Slash command availability** — `/skill-name` invocation depends on Zed's
current AI integration. If slash commands are unavailable, paste the skill
name in prose: "Please run learn-harness with --map-only."
