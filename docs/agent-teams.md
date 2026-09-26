# Agent teams

The four team skills help choose roles, assign bounded work, select models and transfer unfinished tasks between coding tools. Start with `agent-team-creator` and describe the outcome in plain language. A small isolated change usually needs one implementer. Add a specialist for a distinct deliverable or an independent reviewer when a separate check is valuable. Parallel roles need separate file ownership before they edit.

| Skill | Responsibility |
| --- | --- |
| `agent-team-creator` | Recommend an editable team; validate its manifest; stage native artifacts. |
| `agent-team-manage` | Record tasks, dependencies, owners, status and evidence with revision checks. |
| `agent-team-models` | Discover configured models and explain selection against explicit policy. |
| `agent-team-handoff` | Capture fresh task context and transfer ownership after destination acceptance. |

All four use the creator's self-contained runtime. Installed execution requires **Node.js 22 or newer**. The shipped `scripts/*.mjs` need no TypeScript installation, package manager, shell script, executable bit or repository-root module. The full and mini packages carry the same runtime. Copy all four sibling skills when installing the family manually.

## Start with an outcome

From this repository, every command has the same form:

```sh
node skills/process/agent-team-creator/scripts/cli.mjs guide --input guide-request.json
```

For an installed skill, replace the script path with its actual `agent-team-creator/scripts/cli.mjs` location. Requests are JSON files, so native settings and paths containing spaces do not need command-line encoding. Commands print JSON; failures print a JSON error and return a nonzero exit code.

Save this as `guide-request.json`:

```json
{
  "id": "settings-accessibility",
  "outcome": "Make the settings page usable with a keyboard",
  "complexity": "simple",
  "areas": ["code"],
  "deliverables": ["Accessible settings page", "Recorded keyboard verification"],
  "budget": "balanced",
  "review": false,
  "harness": "codex",
  "scope": "project"
}
```

An empty request `{}` returns the intake questions. `review` is a JSON boolean; `budget` is `economy`, `balanced` or `quality`; `complexity` is `simple` or `complex`. The first complete intake returns proposed roles, reasons, a single-agent alternative and questions for unresolved file ownership. Supply an `ownership` map from each proposed role ID to its nonempty project-relative output paths or globs; only `ready: true` returns `team`. Suggested skill names are discovery hints: inspect installed skills and replace unavailable suggestions. Inspect the project and reuse known scope before answering ownership questions. Assign reviewers a separate findings output path; their read scope can be broader. Edit each role's prompt, inputs, outputs, dependencies and model policy before saving it.

Experts may provide the manifest directly. This complete `init-request.json` creates one local team:

```json
{
  "state": ".agent-teams/settings-accessibility.json",
  "team": {
    "schemaVersion": 1,
    "id": "settings-accessibility",
    "outcome": "Make the settings page usable with a keyboard",
    "scope": "project",
    "harness": "codex",
    "roles": [{
      "id": "implementer",
      "description": "Implement and verify the settings page changes",
      "prompt": "Stay within assigned files and report verification evidence and remaining work.",
      "skills": [],
      "owns": ["src/settings/"],
      "inputs": ["Keyboard acceptance criteria"],
      "outputs": ["Accessible settings page", "Verification record"],
      "dependsOn": []
    }]
  }
}
```

```sh
node skills/process/agent-team-creator/scripts/cli.mjs validate --input init-request.json
node skills/process/agent-team-creator/scripts/cli.mjs init --input init-request.json
```

`init` refuses to replace an existing state. `status` reads it using a request containing `state`. Use `team-update` with `state`, the current `expectedRevision` and the replacement `team` to revise an initialized definition. Team identity cannot change, and roles referenced by task history cannot be removed. These local records coordinate work; they do not enforce native filesystem permissions.

## Stage a native export

Save this as `export-request.json` and run the command below:

```json
{
  "state": ".agent-teams/settings-accessibility.json",
  "target": "codex",
  "out": ".agent-team-exports/settings-accessibility-codex"
}
```

```sh
node skills/process/agent-team-creator/scripts/cli.mjs export --input export-request.json
```

Export writes into a new staging directory. It refuses an existing output directory and unsafe paths, reserved generated filenames, case-insensitive collisions and file/directory overlaps. Review the result and install selected native files deliberately. **Export does not install a plugin, register an agent or start execution.** The native harness retains its own invocation, permissions, session and model rules.

The nine targets are `uar`, `codex`, `claude`, `copilot`, `kimi`, `minimax`, `opencode`, `deepseek` and `bossfang`. BossFang is a deployment scope/target; the manifest's `harness` separately names the execution harness. Native formats and verified sources are recorded in the [native contract reference](../skills/process/agent-team-creator/references/native-harnesses.md).

| Target | Native output and relevant limit |
| --- | --- |
| Codex | `.codex/agents/*.toml` and optional proposed `.codex/config.toml`; no invented plugin `agents` field. |
| Claude Code | Project agent Markdown and an alternative native plugin/marketplace; these definitions do not automatically create an experimental agent team. |
| Copilot | `.github/agents/*.agent.md`; Fleet execution remains a separate native feature. |
| Kimi Code | `.kimi-code/agents/*.md` and alternative plugin/v2 marketplace; per-role model frontmatter is ignored. |
| MiniMax | `agents/<name>/agent.md` for the active user-data directory, usually `~/.minimax`; `mcode exec` has no verified custom-agent selector. |
| OpenCode | `.opencode/agents/*.md` and optional `opencode.json`, using the deployed singular `agent`/`permission` schema. |
| DeepSeek Harness | Cordis persona profiles and a separate experimental team composition; the lead creates members at runtime, with no static per-member model setting. |
| UAR | Complete per-agent `AgentArtifact` request bodies for `/api/agents`; no invented persistent team API. |
| BossFang | Agent TOML, standalone registration bodies, workflow and alternative multi-agent Hand; registration, activation and workflow execution remain separate actions. |

Role `native[target]` objects override native agent fields. Team `native[target]` requires `source` and `version`, and can contain `options` and `files`. Unknown fields and opaque file contents are preserved, not certified. For Codex, Claude and OpenCode, options become proposed native project configuration. UAR options supply artifact defaults; BossFang options override the Hand; DeepSeek options configure its experimental team service. Kimi, MiniMax and Copilot team options are retained in `native-options.json` for deliberate application through the installed tool's supported configuration interface. Opaque files cannot replace generated files; use a role override or a distinct alternate artifact.

Claude and Kimi have verified agent plugin/marketplace exports. Install either their project agent files or the plugin alternative to avoid duplicate definitions. Other targets disclose unverified agent-marketplace mappings rather than inventing them. This is separate from distributing the four **skills** through the pack's existing Claude/Codex plugins and harness surfaces. Skill installation alone does not create a team.

UAR defaults require explicit review of tool policy and bundles before registration; skill preference is not a deny policy. BossFang's native `skills=[]` means all unless `skills_disabled=true`; an empty portable role skill list exports the disabled form. Supply an approved service URL and credential reference for registration, retain returned IDs, and account for partial success. Hand activation can start autonomous schedules. Neither export nor discovery establishes authorization to mutate a service.

## Install or adopt a project team

After reviewing the export, finish normal project-team creation with `install-project`. For a new team, save `{"project":"/path/to/project","team":<manifest>}` as `install-request.json`, then run:

```text
node skills/process/agent-team-creator/scripts/cli.mjs install-project --input install-request.json --dry-run
node skills/process/agent-team-creator/scripts/cli.mjs install-project --input install-request.json
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project" --check
```

For an existing team, use `--project "/path/to/project"` without an input manifest. A recorded `.agent-team/project-routing.json` selection wins; otherwise a sole `.agent-team/<id>/team.json` is adopted. Multiple candidates require `--team <id>`; stale selections fail rather than silently switching. Intentional manifest replacement requires `updateTeam: true` in the request. Creator check exits 2 for drift and 1 for errors.

Installation writes managed discovery pointers to both instruction entrypoints, the active routing record and missing native definitions. Existing native files, role IDs, ownership, model policies, permissions and concurrency remain intact; differing configuration is reported for deliberate merge. Recovery records retain prior instruction bytes. Export stages proposals; installation establishes discovery. Neither starts execution or activates UAR/BossFang registration.

All code tasks use the selected team’s relevant roles. UI roles conditionally use `prometheus-ui-ux`; UI review uses `prometheus-ui-review` after the whole implementation phase, without taste or user-only skill preloads. Backend work loads no UI guidance. Use native delegation only if available, otherwise disclose sequential role execution; builder self-review is not independent review. Zed receives the pointer in its effective existing instruction file. External ACP agents keep native configuration, and Zed parallel threads are not a delegation API.

See [UI/UX routing](guide/25-ui-ux-routing.md) for the selective workflow.


## Choose models from evidence

`models-discover` accepts `kind: "openai"` with a configured `baseUrl`, or `kind: "uar"`/`"bossfang"` with an explicit `discoveryUrl`. Authentication uses `auth.env` referencing a local environment variable; do not embed tokens in JSON. The OpenAI-compatible route is `/v1/models`, UAR uses `/api/uar/providers/{id}/models`, and BossFang uses `/api/models`. Native discovery describes configured availability, not successful inference.

`models-select` takes `team`, `roleId`, a `skills` array, optional `taskPolicy`, and `catalog` containing the discovery response. It applies scalar overrides in team → role → supplied skills → task order; required capabilities accumulate. A policy may include an exact `model`, declared `tier` (`low`, `medium`, `hard`), `capabilities`, `maxInputPerMillion` and `maxOutputPerMillion`. Prices are USD per million tokens. Tiers are operator annotations, not inferred from names. Exact tiers must match; a `hard` label is not automatically a substitute for `medium`.

Available models can be joined to liter-llm catalog schema 1 through explicit `aliases`; discovery `tiers` maps native IDs to declared tiers. Missing capabilities or prices remain unknown, and unknown prices cannot satisfy a ceiling. Selection explains rejections and can return no model. Stale price data is disclosed. Review the result and save the explicit model ID into the appropriate policy before native export; selection does not rewrite team state or configure a harness. Kimi and DeepSeek report their unsupported per-role model behavior.

## Manage tasks and accepted handoffs

Local state holds revisioned tasks, handoff packets, events and a memory outbox. Mutations require the latest `expectedRevision`; task changes also require the current owner and `expectedTaskRevision`. Read `status` after each successful operation instead of guessing revision numbers. A lock and atomic file replacement protect the local file; they are not a distributed ownership lease. A held lock is never automatically stolen.

For a newly initialized state at revision 0, a `task` request can be:

```json
{
  "state": ".agent-teams/settings-accessibility.json",
  "expectedRevision": 0,
  "task": {
    "action": "add",
    "id": "keyboard-navigation",
    "title": "Repair keyboard navigation",
    "owner": "implementer",
    "dependsOn": []
  }
}
```

Supported actions are `add`, `start`, `block`, `cancel`, `reassign` and `complete`. Start requires completed dependencies. Completion requires a running task, evidence and no remaining work. Reassigning running work returns it to pending. Block and cancel require a reason. A local status change does not launch or interrupt the native agent process.

`handoff-create` takes `state`, `expectedRevision`, `cwd` and a `handoff` object with `taskId`, `owner`, `expectedTaskRevision`, `toOwner`, `toHarness`, `context`, `evidence`, `remaining` and `memoryRefs`. The destination owner must already be a team role. The resulting packet records the source task revision, a fresh prompt, Git HEAD/branch/dirty status and unresolved work. Creation keeps ownership with the source. `handoff-accept` takes `state`, the current `expectedRevision`, packet `id` and `destination: {owner,harness}`. Only a matching, current destination can accept; stale transfers fail. Source session IDs, credentials and permissions do not transfer.

For KBD work, attach the exact `projectId`, `runId`, `phaseId`, `changeId` and `taskId` under the task's `kbd` field. Ordinary task completion refuses these tasks. `complete-kbd` additionally takes `cwd` and `task.kbdCli`; it verifies canonical identity/status and records a successful typed KBD transition or reconciles an existing canonical completion. Follow KBD's task boundaries, QA and review requirements. Never edit KBD progress projections or treat a team receipt as completion of the parent phase. A cross-store interruption needs reconciliation, not rollback of canonical history.

## Optional memory and local development

`memory-queue` persists an entry with `content`, explicit `scope` and `provenance`; `memory-publish` later attempts a configured publication. Unavailable memory leaves the entry queued. The verified surreal-memory REST route is `/api/v1/memory/` with explicit identity/scope mapping; another HTTP provider needs a source/version-backed field mapping. This runtime does not discover arbitrary MCP tools automatically. Remote outcomes can be uncertain, and publication is not exactly once: reconcile before explicitly retrying an uncertain attempt. A scope label is not server authorization.

Team memory records remain distinct from canonical Karpathy task boundaries and the `pk` knowledge bundle. The team runtime does not directly rewrite either. No shared-memory service is required for local team definition, task management or export.

Maintainers edit `skills/process/agent-team-creator/runtime/src/*.mts`. The local runtime package pins TypeScript 7.0.2 and uses NodeNext imports ending in `.mjs`; its build emits the shipped `scripts/*.mjs`. After coherent implementation, compile locally with `npm run build --prefix skills/process/agent-team-creator/runtime`, preserve full/mini source and compiled parity, then run the planned integration and documentation gates. End users run the shipped JavaScript. Source inspection and a local compile do not certify Windows, an installed native CLI, live authentication or service execution; consult the current validation receipts for the evidence actually collected.

Guided creation resolves ownership before producing a ready team. For example, after reviewing proposed roles, add `"ownership": {"implementer": ["src/checkout/**"], "reviewer": ["reviews/checkout.md"]}` for a two-role checkout task. Use paths actually appropriate to the project and include every proposed role. Missing ownership returns `ready: false`, `proposedRoles`, and focused questions without a `team` value; it does not grant access to the whole repository.
