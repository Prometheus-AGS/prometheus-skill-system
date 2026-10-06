# Prometheus Skill Pack

Prometheus supplies reusable skills, project workflows and agent-team tools for
AI-assisted development. Use it to give a coding harness a repeatable way to plan
work, assign ownership, implement a specification and retain useful lessons.

Start with the skills profile. Add services when a workflow needs durable memory,
model routing, native research or signed execution evidence.

[Product guide](docs/guide/README.md) ·
[Documentation site](https://prometheus-ags.github.io/prometheus-skill-system/) ·
[Skill catalog](SKILLS.md) · [Contribution rules](CONTRIBUTING.md)

## Choose full or mini

| | Full skill pack | [Prometheus Skills Mini](https://github.com/Prometheus-AGS/prometheus-skills-mini) |
|---|---|---|
| Intended use | Broad language and domain guidance, process orchestration, learning and optional native tools | Portable process, team, research and UI guidance with Node entry points |
| Skill layout | Categorized source under `skills/`; distribution flattens skill names | Flat `skills/<name>/` source |
| Host requirements | Git, Node and Bash for installation; individual workflows can also require Python, jq or Rust | Node 22+ and Git for the core; optional services have their own requirements |
| Windows path | Skills profile through Git Bash or WSL; full native service installation is rejected | Native Node core; host and harness acceptance is specific to the documented path |
| Optional infrastructure | Native memory, knowledge, gateway, research and execution components | File state and optional HTTP services; no dependency on the full pack's Python learning pipeline |

These are separate products with shared contracts. A copied skill, generated
plugin or exported team definition does not prove that every harness can execute
it. Read the selected skill's prerequisites and the
[platform guide](docs/guide/17-platform-support.md) before choosing a host path.
Counts and release identities come from the selected checkout's
[skill-system.json](skill-system.json), [package.json](package.json), the
[release matrix](config/release-version-matrix.json) and pinned Git dependencies.
Mini also has an owner-controlled `versions.toml`; full has no root file by that
name. This entry point does not select a new release.

## Install the skills

Use a stable checkout you intend to retain. Registered plugin sources must remain
available after installation; a disposable task worktree is unsuitable as a
permanent plugin source. Node 22+ covers the team runtime as well as the root
package's Node requirement.

```bash
git clone https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
npm ci
./install.sh --profile skills --targets detected --dry-run
./install.sh --profile skills --targets detected
```

The installer shows the selected clients and mutations before confirmation. If
no client is detected, pass a comma-separated list of target IDs from
`skill-system.json`, such as `--targets claude,codex`. Required imports initialize
at their declared commits. The skills profile installs an immutable generation
and selected skill targets; it does not build native binaries or start services.

For an explicitly reviewed unattended install, add `--non-interactive --yes`.
Installing the Codex target also applies the pack's Codex memory policy and
archives known native memory summaries; inspect the displayed plan first.
`CODEX_HOME`, when set, selects Codex's configuration root; `--home` supplies the
fallback home for the other targets.

After installation:

```bash
./install.sh --verify --targets detected --non-interactive
```

This checks the selected installation surfaces. It does not certify a provider
connection, a native harness session or an optional service workflow.
See [installation details](docs/guide/19-installation.md) and
[plugin distribution](site/docs/plugin-distribution/immutable-generations.md).

## Do the first useful piece of work

Open an existing project in your harness and ask it to use the installed skill by
name. For example:

> Use `kbd-process-orchestrator` to plan a small feature in this project. Record
> the specification, production entry point, acceptance criteria and file
> ownership. Implement the complete production change before running its local
> integration gate.

KBD retains change and task identity across sessions. Model and tool availability
determine which steps can run; unavailable providers and missing prerequisites
must stay visible in task state. Read [the KBD overview](site/docs/kbd/overview.md)
and [task model assignments](site/docs/kbd/task-model-assignments.md).

For an improvement cycle, name `iterative-evolver`. For a sourced investigation,
name `deep-research` and read its
[stage contracts](skills/research/deep-research/references/stage-contracts.md).
For UI or product copy, name `prometheus-ui-ux`; it preserves the project's design
authority and selects focused craft guidance.

## Work with teams and models

Use `agent-team-creator` to create or import a team, assign role and path
ownership, and stage native definitions. `agent-team-manage` tracks tasks and
claims; `agent-team-handoff` records destination acceptance before ownership
transfers. An export is a proposal artifact. Installation, registration and
execution have separate boundaries.

Adopt an existing selected team before dispatching code work. A sole team can be
adopted; multiple teams require an explicit selection. Preserve native permissions
and use the harness's actual delegation capabilities. When delegation is
unavailable, disclose the sequential fallback.

`agent-team-models` records choices and their evidence. Use concrete model IDs
available on your route, with reasoning effort only where supported. Policy
resolves from team to role to skill to task. Catalog entries and price estimates
are not proof of live inference or comparative quality. Independent review runs
after the complete production phase, through the required reviewer route.

Read [the team handbook](docs/guide/24-agent-teams.md),
[request contracts](docs/agent-teams.md), and
[models and memory](skills/process/agent-team-creator/references/models-memory.md).

## Add services deliberately

| Component | Role and boundary |
|---|---|
| `surreal-memory-server` | Optional durable memory service; separate repository pinned under `tools/` |
| `prometheus-knowledge` | Knowledge CLI and learning worker; separate repository pinned under `tools/` |
| `liter-llm` | Optional model gateway; credentials and route availability belong to your installation |
| `forge-rs` | Native context enrichment and template tooling in this repository |
| `prometheus-research` | Native background research entry point; separate from the prompt-only workflow |
| `prometheus-exec` | Optional signed execution receipts; does not restrict the harness's ordinary tools |
| Prometheus Companion | Separate optional extension for connected control-plane and peer-sync features |

The packs operate without Companion. Companion consumes the pack's extension
contract; the pack does not install or start it as a prerequisite. Companion's
publication and installation are separate from a pack release. See
[the extension contract](docs/integration-contract.md) and
[the relocation decision](docs/decisions/sovereign-sync-relocated-to-companion.md).

On macOS or Linux, preview the full profile before opting into native builds,
MCP configuration and user-service installation. The service installer requires
Bash 4+; ensure `bash` on PATH is a modern installation, rather than macOS's
system Bash 3.2:

```bash
./install.sh --profile full --targets detected --dry-run
./install.sh --profile full --targets detected
```

Service components have independent versions and storage configuration. A
successful installer exit can include warnings and does not certify every
requested service. Credentials, durable state, backups and runtime certification
need their own operations procedure. See [service operations](docs/guide/26-service-operations.md),
[deployment modes](docs/deployment-modes.md) and [execution operations](site/docs/execution/installation-doctor-and-recovery.md).

## Update, recover or remove an installation

From a clean, retained source checkout:

```bash
npm run update
```

The updater fast-forwards the checkout, initializes declared imports, checks
distribution drift, installs and verifies a generation, refreshes native plugin
surfaces, and advances the installation receipt after success. Resolve a dirty
or diverged checkout before updating. Back up project state and service data
before changing their release identities.

Generation rollback requires current and previous generations; the previous
release must satisfy the minimum active version. It restores skill activation,
not a service database migration:

```bash
node scripts/install-plugin-generation.js --rollback --targets claude,codex
```

To preview and remove only receipt-owned selected skill surfaces:

```bash
./install.sh --uninstall --targets claude,codex --dry-run
./install.sh --uninstall --targets claude,codex
```

This retains unrelated skills, service data and separately installed native
plugin registrations. Unregister those plugins through their harness. For the
full service profile, `bash scripts/install-mcp-services.sh --unload` stops
managed services and retains their unit files and data. Read
[updating and recovery](docs/guide/20-updating.md) before removing persisted state.

## Documentation and maintenance

- [Guide index](docs/guide/README.md): workflows, skills, installation and contribution pages.
- [Memory tiers](docs/guide/memory-tiers.md): scoped lessons, durable queue and recall boundaries.
- [UI/UX routing](docs/guide/25-ui-ux-routing.md): context authority and completed-phase evidence.
- [Research client](docs/deep-research/README.md): native UI and server prerequisites.
- [Readiness evidence](docs/production-readiness-report.md): source, artifacts, runtime and deployment boundaries.

Support is specific to a release, host and production entry point. Source
convergence and generated bytes do not establish an installed release. Published
release manifests and applicable local evidence determine compatibility; keep
dependency pins and recovery receipts together. Design documents and old release
receipts are history, not installation promises.

## Contribute

Read [AGENTS.md](AGENTS.md), [CLAUDE.md](CLAUDE.md) and
[CONTRIBUTING.md](CONTRIBUTING.md) before editing. Preserve upstream licenses,
attribution, provider schemas and owner-controlled pins.

Complete the coherent production implementation before authoring, modifying or
running tests. Then run the smallest relevant full integration gate through the
real production entry point and collaborators. Validation, builds and independent
review run locally at the final phase boundary. Hosted CI and unit-only results
are not acceptance evidence. Record commands, source identity, results and gaps
before proposing a release or push.

Update canonical documentation alongside the behavior it describes. Generated
distribution, indexes and site mirrors are refreshed from their sources at the
final boundary. Protected BDD changes require the signed approval protocol. Only
the owner approves release pins, protected version changes and tags.

Licensed under [MIT](LICENSE). Imported skills and vendored guidance retain their
own license and provenance records.
