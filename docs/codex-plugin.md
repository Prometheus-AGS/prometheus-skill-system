# Codex Plugin & Marketplace

The Codex package is generated from `skill-system.json` and the collected skill inventory. It shares skill content with the Claude package while retaining its own native manifest. Do not hand-edit generated payloads or marketplace entries.

| Artifact | Source | Generator |
| --- | --- | --- |
| `dist/plugins/codex/prometheus-skill-pack/.codex-plugin/plugin.json` | Distribution contract and inventory | `scripts/generate-skill-system-distribution.js` |
| `.agents/plugins/marketplace.json` | Contract marketplace entries, imports and package paths | Same generator |
| Packaged `skills/` and `skill-index.json` | Collected skill source directories | Same generator |
| Packaged `.mcp.json` | Root MCP template, checked for machine paths and literal credentials | Same generator |

The current manifest exposes `skills: "./skills"`, `mcpServers: "./.mcp.json"` and the Codex `interface` metadata. It has no `hooks` or native-team `agents` field: hooks ship as `hooks/hooks.json` at the package root, which Codex discovers by convention (see Hooks below). The compatibility wrapper `scripts/build-codex-plugin.js` delegates to the distribution generator and explicitly rejects a manifest containing `hooks`.

## Local generation and installation

```sh
npm run build:codex
npm run validate:codex
```

These commands generate and check the shared distribution locally. `validate:codex` checks drift without writing. It does not certify an installed Codex version, authenticate MCP servers or run agents. Hosted test workflows are not a validation path.

Every target in `skill-system.json` declares `sourceTreeLifecycle`. Required repository trees must exist and be populated; install-only destinations may be absent until installation. The generated marketplace points to packaged payloads and adjacent plugin sources. Curated user-skill catalog selection in `config/codex-catalog.txt` is separate from the full packaged inventory.

The repository recorded these installation verbs with codex-cli 0.144.1; inspect the installed CLI's help before using them with a different release:

```sh
codex plugin marketplace add .
codex plugin add prometheus-skill-pack@prometheus-skill-pack
codex plugin list
codex mcp list
```

Start a new native session as required by the installed tool. Installation and MCP discovery do not establish successful authentication or execution. Provision credentials through environment references or user-local native configuration; never commit secret values. The packaged MCP template must remain portable and free of machine-specific paths.

Ordinary `prometheus setup --full` uses the signed local KBD runtime. Optional cross-machine replication is managed separately by `prometheus-companion`; it is not a prerequisite for these skill procedures or typed KBD mutations.

## Selecting the Codex home

The installer uses a nonempty inherited `CODEX_HOME` first. Otherwise, it uses `.codex` under the selected `--home` directory, or `HOME` when `--home` is omitted. An empty `CODEX_HOME` is unset. The selected path is made absolute and normalized before subprocesses change working directory; normalization does not require the directory to exist or resolve symlinks. `--home` changes the fallback and the other platform homes, but does not override an explicit `CODEX_HOME`. No additional Codex-root option is needed.

Native plugin inspection, memory policy, MCP configuration and copied skills all use that root. Verification and rollback check its actual skill directories and generation markers; uninstall removes only managed entries there. A generation marker must belong to a verified generation in the selected plugin store and name one of its skills. Unknown, malformed or unverified markers are preserved with a diagnostic. Supported legacy Codex `source=skills/...` markers and MiniMax platform/name metadata remain recognized. Unowned files and the unused fallback Codex directory are preserved.

Signed receipts in the plugin store certify the portable payload and logical target `.codex/skills`; they do not encode an absolute user home. Verification also checks the actual projection under the effective Codex root, so a valid store receipt alone cannot certify a missing custom-root copy. Generation cleanup scans selected copy targets for references and refuses to retire a referenced generation. Doctor uses the same root selection for memory policy and native Codex-cache inspection and supplies it explicitly in verified helper suggestions.

These lifecycle paths have source implementation only. Final scratch acceptance must set `HOME`, `CODEX_HOME` and `PROMETHEUS_PLUGIN_ROOT` explicitly, including empty-root and path-with-spaces cases; an inherited real `CODEX_HOME` can escape scratch isolation. Standalone goal setup and slash-command registration are separate paths and were not changed by this installer work.

## Codex memory policy and startup consolidation

The policy has three settings:

```toml
[features]
memories = false

[memories]
generate_memories = false
use_memories = false
```

For codex-cli 0.158.0, release commit `064c6b8c737f5b41d171fdda80bd9ef10ad06eb3`, the [startup entry point](https://github.com/openai/codex/blob/064c6b8c737f5b41d171fdda80bd9ef10ad06eb3/codex-rs/memories/write/src/start.rs) skips ephemeral sessions, non-root agents and sessions with the memory feature disabled. It does not check `generate_memories` or `use_memories` before starting the pipeline. According to the [configuration types](https://github.com/openai/codex/blob/064c6b8c737f5b41d171fdda80bd9ef10ad06eb3/codex-rs/config/src/types.rs), `generate_memories=false` changes the eligibility of newly created threads, while `use_memories=false` skips memory usage instructions in developer prompts. Neither alone prevents startup consolidation.

Startup runs extraction and then consolidation. The [consolidation phase](https://github.com/openai/codex/blob/064c6b8c737f5b41d171fdda80bd9ef10ad06eb3/codex-rs/memories/write/src/phase2.rs) can claim a global job and use retained extraction outputs from earlier eligible threads even when the current thread disables generation. Its internal agent disables generation and use while writing artifacts. The feature gate is therefore the setting that blocks new startup pipelines in this release. These source findings do not certify a desktop engine or prove that a machine change has been applied.

Selected Codex installs call `shared/scripts/codex-memories-config.sh --create` through the actual installer dispatch, including a missing config. The helper requires stdlib `tomllib`, preserves unrelated TOML and comments, validates the candidate before replacement, and keeps collision-safe backups. Invalid TOML or unsupported target inline-table syntax fails explicitly without replacing the original. Repeated application leaves an already compliant config unchanged. Uninstall does not rewrite the memory policy.

Apply mode archives only `$CODEX_HOME/memories/memory_summary.md` (v1) and `$CODEX_HOME/memories_v2/memory_summary.md` (v2), whose directories are defined by [MemoryVersion](https://github.com/openai/codex/blob/064c6b8c737f5b41d171fdda80bd9ef10ad06eb3/codex-rs/protocol/src/memory_version.rs). Archive names include the version and timestamp with collision suffixes under `memories-archive/`. `MEMORY.md`, `raw_memories.md`, extension files and database state remain intact.

`--check` writes nothing and reports each setting and v1/v2 summary presence separately. `prometheus doctor --check codex.memories` parses the complete config and passes only when all three settings are false and both known summaries are absent. Invalid tables, values or TOML are unhealthy. These checks inspect persisted user configuration; selected profiles, project/agent configuration or CLI overrides can change effective settings. Sessions and jobs already running may retain cloned configuration. An owner-approved machine application must include effective-setting inspection and session restart/drain; this repository implementation has not performed that operation. Controlled scratch runtime evidence remains part of the final local integration gate.

When repair is needed, doctor resolves `PROMETHEUS_PLUGIN_ROOT`, falling back to the selected home's `.prometheus/plugins/prometheus-skill-pack`. It verifies the installed generation, signed target receipts and selected projections using the existing installer verifier embedded in the CLI, without executing code from an unverified `current` link or requiring a source checkout. This read-only path uses the existing capability record only when its schema, store root, boolean fields and installer source identity match; it never probes or refreshes the cache. The helper must be a regular executable file owned by the verified manifest and contained in that generation.

Doctor then offers a shell-quoted absolute command with explicit effective `CODEX_HOME`, so spaces and quote characters in paths cannot change the command. The command is an owner-approved apply suggestion; diagnostic checking never runs it. If Node.js, the matching capability record, generation, receipt or owned helper is unavailable or invalid, doctor offers installation remediation without a helper command. Repair the installation before applying memory policy; a repository-relative fallback is not emitted.

## Agent-team skills and native agent exports

The process plugin source roster includes `agent-team-creator`,
`agent-team-manage`, `agent-team-models` and `agent-team-handoff`. The distribution
generator collects these skill directories into plugin payloads; the curated
Codex catalog is a separate source selection. Update source rosters and catalog
configuration, then regenerate through the existing local distribution commands.
Do not edit generated manifests, marketplace entries or copied runtime modules.

Installing these procedures makes the team workflow available. It does not
create native agents. The creator's compiled `scripts/cli.mjs` runs with Node.js
22+ and no TypeScript installation. Its `export` command stages
`.codex/agents/<name>.toml` with native `name`, `description` and
`developer_instructions`, plus any supplied role overrides. Team native options
become a proposed `.codex/config.toml`; the exporter never merges that proposal
into the live project. Existing output directories and filename collisions fail.

Review the staged files and their source/version receipts before installation.
The native agent format is [source-verified](https://learn.chatgpt.com/docs/agent-configuration/subagents);
this does not certify the installed Codex version or a live invocation. Codex
plugin skill support does not establish an `agents` plugin manifest field, so
the exporter uses standalone native agent files. Claude and Kimi have separately
verified agent-plugin layouts; their artifacts are not Codex manifests.

The [agent-team reference](agent-teams.md) covers JSON requests, model constraints,
task ownership, accepted handoffs and optional memory. KBD-linked tasks complete
through canonical KBD commands and recorded receipts, not by editing progress
projections or treating a local team status as canonical completion.

## Hook evidence is version-scoped

## Hooks

The package ships `hooks/hooks.json`, rendered from `shared/harnesses/hook-contract.json` as `hooks/codex-hooks.json`, together with the runtime closure every hook needs: `scripts/hook-entry.mjs`, `scripts/lib`, `shared/` and the signed-skill runtime. Codex finds a plugin's `hooks/hooks.json` by convention, so the manifest declares no `hooks` key.

Verified on codex-cli 0.158.0 (2026-10-03):

- **Codex runs only the `command` string and ignores `args`.** Every Codex entry is one string: `node ${CLAUDE_PLUGIN_ROOT}/scripts/hook-entry.mjs --bundle <id> --hook <id> --harness codex`. Codex substitutes `${CLAUDE_PLUGIN_ROOT}` and exports both `CLAUDE_PLUGIN_ROOT` and `PLUGIN_ROOT`.
- **`timeout` is in seconds in both Codex and Claude Code** (Codex default 600). The generator enforces 1–600 seconds. Starting the entry point takes about one second, so contract hooks use at least 10 seconds. An earlier note that Codex reads milliseconds was a misdiagnosis, corrected by change-tlm-004.
- **Hook stdout that opens with `{` is parsed as a structured response.** Hooks must not begin their output with raw JSON unless they mean it. `subagentstart-learning` is the one that does: it prints `{"hookSpecificOutput":{"hookEventName":"SubagentStart","additionalContext":…}}`, which Codex injects into the **child** thread as a developer message (it never reaches the parent thread).

### Immutable hook generations

The dispatcher generator emits `export PYTHONDONTWRITEBYTECODE=1` before any command. Both compiled and shell runtime paths converge on this dispatcher, so Python hooks and their descendants inherit suppression even when the caller sets the variable to `0`. Direct invocation of `sessionstart-learning.sh`, `subagentstart-learning.sh` or `subagentstop-learning.sh` applies the same export. The standalone `learning_write.py` sets `sys.dont_write_bytecode = True` before local imports and sets the environment variable to `1` for new interpreters, including its enqueue helper and detached Cortex feeder. Command results and hook payload formats are preserved.

Generation verification still checks the complete payload, including content and executable modes; bytecode directories have no exemption. Generated dispatchers and packaged copies remain generator-owned. Change `change-ldd-12-integration-rollout` owns generated-output reconciliation and the final local integration gate for this change. That gate must exercise packaged compiled and shell entry points, direct wrappers and writer/feeder imports with suppression absent or `0`, demonstrate learning activity in scratch state, and compare the entire generation before and after execution. Source edits alone do not certify an installed generation. Keep installed caches and existing live bytecode untouched during implementation.

### Enabling hooks: `[features].hooks` and the one-time trust prompt

Verified on codex-cli 0.158.0 (2026-10-04, phase team-aware-learning-memory-impl, change B5):

- **The feature flag is `[features].hooks`.** The older name `[features].codex_hooks` is deprecated; in the Codex SubagentStart spike a config that set only `codex_hooks` (run without the trust bypass) never fired a hook, while `hooks = true` plus the bypass did. Use:

  ```toml
  [features]
  hooks = true
  multi_agent = true   # needed for subagents, and so for SubagentStart/SubagentStop
  ```

- **Plugin hooks are non-managed, so Codex asks once before running them.** The first interactive `codex` session after the plugin is installed shows a hook-trust prompt; accept it once and the plugin's hooks run from then on. A project-layer config (`.codex/`) is honoured only for a trusted project (`[projects."<path>"] trust_level = "trusted"`).
- **Headless automation** (`codex exec`) never shows the prompt; vetted automation passes `--dangerously-bypass-hook-trust` instead. The B5 alpha gate (`shared/scripts/tests/test-subagent-delivery.sh`) uses that flag only inside a fully scratch `CODEX_HOME`.
- **SubagentStart and SubagentStop fire through plugin hooks** with the bare TOML agent name as `agent_type` (`-` in a role id becomes `_`, e.g. role `api-dev` → agent `api_dev`), plus `agent_id`. `agent_identity.py` maps the name back to the role.

To check firing without touching the machine's real hook runtime, isolate all three of `CODEX_HOME`, `PROMETHEUS_PLUGIN_ROOT` and `HOME`. Then run `codex plugin marketplace add <repo>`, `codex plugin add prometheus-skill-pack@prometheus-skill-pack`, and `codex exec --skip-git-repo-check --dangerously-bypass-hook-trust "ok"`, and count the `hook: SessionStart Completed` lines on stderr.

From PR #54 (2026-08-10) until this fix, the generated package shipped no hooks and no Prometheus hook fired in Codex. The change-cpd-006 evidence for codex-cli 0.144.1 predates both the generated package and the exec-form hooks.

## Updating the distribution

Edit the skill sources and the relevant source inventory, domain roster or curated catalog. Release metadata and output paths belong to `skill-system.json`. Regenerate locally, inspect the payload diff and run the planned local validation gates before publication. Native installation or live service testing requires separate evidence; a clean generation check is not that evidence.

UAR can ingest the source `skills/<domain>/<name>/SKILL.md` tree through its configured built-in skills directory. The four agent-team procedures remain normal skills in that tree. Ingesting those skills does not register exported UAR agents or start an execution loop; service registration is a separate operation described in the [team reference](agent-teams.md).
