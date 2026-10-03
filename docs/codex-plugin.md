# Codex Plugin & Marketplace

The Codex package is generated from `skill-system.json` and the collected skill inventory. It shares skill content with the Claude package while retaining its own native manifest. Do not hand-edit generated payloads or marketplace entries.

| Artifact | Source | Generator |
| --- | --- | --- |
| `dist/plugins/codex/prometheus-skill-pack/.codex-plugin/plugin.json` | Distribution contract and inventory | `scripts/generate-skill-system-distribution.js` |
| `.agents/plugins/marketplace.json` | Contract marketplace entries, imports and package paths | Same generator |
| Packaged `skills/` and `skill-index.json` | Collected skill source directories | Same generator |
| Packaged `.mcp.json` | Root MCP template, checked for machine paths and literal credentials | Same generator |

The current manifest exposes `skills: "./skills"`, `mcpServers: "./.mcp.json"` and the Codex `interface` metadata. It has no `hooks` or native-team `agents` field. The compatibility wrapper `scripts/build-codex-plugin.js` delegates to the distribution generator and explicitly rejects a manifest containing `hooks`.

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

The earlier change-cpd-006 experiment recorded plugin-hook behavior with codex-cli 0.144.1. That historical evidence does not describe the current generated package, whose manifest advertises skills and MCP and excludes `hooks`. Do not add a rejected field or claim hook firing from current plugin installation. Other generated hook artifacts retain their own installation and receipt contracts; they are not proof that the Codex plugin consumes them.

## Updating the distribution

Edit the skill sources and the relevant source inventory, domain roster or curated catalog. Release metadata and output paths belong to `skill-system.json`. Regenerate locally, inspect the payload diff and run the planned local validation gates before publication. Native installation or live service testing requires separate evidence; a clean generation check is not that evidence.

UAR can ingest the source `skills/<domain>/<name>/SKILL.md` tree through its configured built-in skills directory. The four agent-team procedures remain normal skills in that tree. Ingesting those skills does not register exported UAR agents or start an execution loop; service registration is a separate operation described in the [team reference](agent-teams.md).
