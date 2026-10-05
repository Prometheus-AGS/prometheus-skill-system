# Agent teams

This page is the executable request reference. Follow the [canonical team handbook](https://prometheus-ags.github.io/prometheus-skill-system/docs/guide/agent-teams) for the workflow, ownership and native boundaries. Full commands use `skills/process/agent-team-creator`; mini commands use `skills/agent-team-creator`. Installed use resolves the actual creator directory. Examples document source contracts, not completed native execution.

The four team skills help choose roles, assign bounded work, select models and transfer unfinished tasks between coding tools. Start with `agent-team-creator` and describe the outcome in plain language. A small isolated change usually needs one implementer. Add a specialist for a distinct deliverable or an independent reviewer when a separate check is valuable. Parallel roles need separate file ownership before they edit.

| Skill | Responsibility |
| --- | --- |
| `agent-team-creator` | Recommend an editable team; validate its manifest; stage native artifacts. |
| `agent-team-manage` | Record tasks, dependencies, owners, status and evidence with revision checks. |
| `agent-team-models` | Discover configured models and explain selection against explicit policy. |
| `agent-team-handoff` | Capture fresh task context and transfer ownership after destination acceptance. |

All four use the creator's self-contained runtime. Installed execution requires **Node.js 22 or newer**. The shipped `scripts/*.mjs` need no TypeScript installation, package manager, shell script, executable bit or repository-root module. Both packages ship local task/handoff commands; optional memory adapters and card/intake commands currently differ. Copy all four sibling skills when installing the family manually.

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
      "owns": ["src/settings/", "notes/settings-*.md"],
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
| UAR | Legacy per-agent `AgentArtifact` staging plus draft.2 canonical package and private-binding administration; durable team activation remains unsupported. |
| BossFang | Agent TOML, standalone registration bodies, workflow and alternative multi-agent Hand; registration, activation and workflow execution remain separate actions. |

Role `native[target]` objects override native agent fields. Team `native[target]` requires `source` and `version`, and can contain `options` and `files`. Unknown fields and opaque file contents are preserved, not certified. For Codex, Claude and OpenCode, options become proposed native project configuration. UAR options supply artifact defaults; BossFang options override the Hand; DeepSeek options configure its experimental team service. Kimi, MiniMax and Copilot team options are retained in `native-options.json` for deliberate application through the installed tool's supported configuration interface. Opaque files cannot replace generated files; use a role override or a distinct alternate artifact.

Claude and Kimi have verified agent plugin/marketplace exports. Install either their project agent files or the plugin alternative to avoid duplicate definitions. Other targets disclose unverified agent-marketplace mappings rather than inventing them. This is separate from distributing the four **skills** through the pack's existing Claude/Codex plugins and harness surfaces. Skill installation alone does not create a team.

UAR defaults require explicit review of tool policy and bundles before registration; skill preference is not a deny policy. BossFang's native `skills=[]` means all unless `skills_disabled=true`; an empty portable role skill list exports the disabled form. Supply an approved service URL and credential reference for registration, retain returned IDs, and account for partial success. Hand activation can start autonomous schedules. Neither export nor discovery establishes authorization to mutate a service.

## UAR draft.2 workspace and package boundary

`export --target uar` still stages legacy per-agent AgentArtifact payloads for existing consumers. It does not create a canonical team. New collaboration definitions use the provider-owned draft.2 profile and a file-backed workspace: `workspace.json`, `manifest.source.json`, separate `agents/*.json`, `teams/*.json`, and `workflows/*.json`. The bundled example names `agents/coordinator.json`, `agents/child.json`, `teams/root.json`, `teams/subteam.json`, and `workflows/review.json` explicitly.

```text
node skills/process/agent-team-creator/scripts/cli.mjs uar-workspace-init --input skills/process/agent-team-creator/assets/uar-intake.json
node skills/process/agent-team-creator/scripts/cli.mjs uar-workspace-status --input workspace-status.json
node skills/process/agent-team-creator/scripts/cli.mjs uar-workspace-update --input update-one-document.json
node skills/process/agent-team-creator/scripts/cli.mjs uar-package-build --input workspace-build.json
```

Status returns counts, one next question, and a bounded diagnostic page. Build requires one top-level TeamDefinition and kind-correct, acyclic, exact-version and exact-digest references. Existing inline and draft.1 packages remain readable through explicit migration with field dispositions; required unsupported semantics refuse build or preflight.

Compiled packages are portable immutable catalog data. DeploymentBinding is separate private installed state and may contain only opaque credential, storage, and RepresentationGrant references. Package installation confers no credential, consent, grant, installed authority, or activation. UAR validates current private state during binding or execution. The accepted draft.2 checkpoint publishes a document contract; it does not claim a durable team runtime, so `uar-activate` continues to refuse.

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


## Task lifecycle request sequence

Use the initialized state from the earlier example. These revisions assume a
fresh state at `0` and no intervening mutations. Read `status` before each real
operation and replace the illustrated revisions with the returned values.
Save each object below in the named JSON file. Run the first command below, then repeat it with `start-task.json`, `block-task.json`, `resume-task.json` and your chosen completion or cancellation request:

```text
node skills/process/agent-team-creator/scripts/cli.mjs task --input add-task.json
```

Save `add-task.json`:

```json
{
  "state": ".agent-teams/settings-accessibility.json", "expectedRevision": 0,
  "task": {
    "action": "add", "id": "keyboard-settings", "title": "Complete keyboard navigation in settings",
    "owner": "implementer", "harness": "codex", "dependsOn": [],
    "evidence": [], "remaining": ["Update settings and collect keyboard integration evidence"]
  }
}
```

Save `start-task.json` after assignment:

```json
{
  "state": ".agent-teams/settings-accessibility.json", "expectedRevision": 1,
  "task": {"action": "start", "id": "keyboard-settings", "owner": "implementer", "expectedTaskRevision": 0}
}
```

If work cannot continue, save `block-task.json`:

```json
{
  "state": ".agent-teams/settings-accessibility.json", "expectedRevision": 2,
  "task": {
    "action": "block", "id": "keyboard-settings", "owner": "implementer", "expectedTaskRevision": 1,
    "reason": "Need the supported setup environment", "evidence": ["notes/settings-blocker.md"]
  }
}
```

After resolving that blocker, save `resume-task.json`:

```json
{
  "state": ".agent-teams/settings-accessibility.json", "expectedRevision": 3,
  "task": {
    "action": "start", "id": "keyboard-settings", "owner": "implementer", "expectedTaskRevision": 2,
    "remaining": ["Finish the production change and record the final integration gate"]
  }
}
```

After the complete production boundary and actual required evidence, save
`complete-task.json`. References are illustrative; replace them with real evidence.
The runtime checks supplied evidence and remaining-work fields, not whether the
referenced test or review really ran.

```json
{
  "state": ".agent-teams/settings-accessibility.json", "expectedRevision": 4,
  "task": {
    "action": "complete", "id": "keyboard-settings", "owner": "implementer", "expectedTaskRevision": 3,
    "evidence": ["notes/settings-integration.md"], "remaining": []
  }
}
```

As an alternative at the same pre-completion snapshot, save `cancel-task.json`:

```json
{
  "state": ".agent-teams/settings-accessibility.json", "expectedRevision": 4,
  "task": {
    "action": "cancel", "id": "keyboard-settings", "owner": "implementer", "expectedTaskRevision": 3,
    "reason": "The owner withdrew this change"
  }
}
```

`complete` and `cancelled` are terminal. Create a new task for follow-up work.
Cancellation changes the ledger; stop native work separately when authorized.
Evidence merges with previous evidence; an explicit `remaining` array replaces
previous work, except that `block` always retains its reason.

To assign a different existing role, use `reassign` with the current owner and
both revisions plus `toOwner` and optional `toHarness`. Reassignment is immediate
administrative intervention; it does not require acceptance or carry a fresh
context packet. A running task becomes pending. Use a separate review task with
its own task dependency after all phase production is complete. There is no
universal `assign`, `update` or `task-accept` command: these operations are
`task` actions, and unsupported action names fail.

Local assignment/start is not a destination handshake. Use `handoff-create` and
`handoff-accept` for a context-bearing transfer. UAR draft.2 direct coordinator
acceptance is the provider's `taskAcceptance` schema policy, not a new local CLI
verb. Its empty `allowedWorkflows` form is valid only for
`mode: "coordinator-within-binding"`; operator mode requires workflow references.
Read the [canonical handbook's acceptance boundary](https://prometheus-ags.github.io/prometheus-skill-system/docs/guide/agent-teams#uar-direct-coordinator-acceptance)
and the vendored schema receipt before claiming provider execution. `uar-activate`
continues to refuse activation.

### Canonical completion and recovery

For linked work, preserve `kbd.projectId`, `runId`, `phaseId`, `changeId` and
`taskId` from canonical status. Local IDs and titles do not select canonical
work. Local `complete` refuses a KBD-linked task. `complete-kbd` uses the same
state/task revision fields, `cwd`, and `task.kbdCli` naming the actual executable;
the canonical task must be `in_progress` or already complete. See the
[complete KBD request](../skills/process/agent-team-creator/references/task-handoff.md#complete-a-task-linked-to-canonical-kbd).

A committed local event records the canonical command ID, identities, revision,
response hash and evidence. It does not bypass canonical claims or permission,
QA, review or phase gates. A crash between canonical commit and local persistence
requires status reads and reconciliation, not invented completion or rollback
of canonical history.

A lock conflict means another writer may still own `<state>.lock`. Inspect its
PID, timestamp and token and confirm no live writer before manually removing an
abandoned lock. Re-read current state and any uncertain remote/canonical result.
Use one reliable local filesystem; locks are advisory coordination, not an
identity check, sandbox, distributed lease or process-cancellation mechanism.

## Model discovery selection and persistence

These requests document source contracts. Use the actual installed creator path
and Node.js 22+. A local declared-catalog comparison needs no service; live
listing needs a compatible, authorized endpoint. Neither is successful inference
or tool-enabled execution. For KBD, choose task fit first within policy; the
helper's cheapest-eligible ordering does not replace that judgment.

Save `model-discovery.json` for an actually configured gateway. The URL/port are
illustrative and credentials remain in an environment reference:

```json
{
  "kind": "openai", "baseUrl": "http://127.0.0.1:8000",
  "auth": {"env": "GATEWAY_API_KEY"}, "timeoutMs": 10000
}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs models-discover --input model-discovery.json
```

The gateway route is `/v1/models`; UAR/BossFang instead need their explicit
`discoveryUrl` and service-native response shape. Save/review returned metadata
with an editor. `available: true` means configured or declared eligibility, not
that the selected model answered an inference or used workspace tools. Map a
gateway alias to exact catalog provider/model IDs explicitly; name similarity
or a provider prefix is not mapping evidence. Consult the
[discovery contract](../skills/process/agent-team-creator/references/models-memory.md#discovery-api) for request details.

Save `model-selection.json`. This complete normalized catalog is an **operator
declaration for illustration**, not live capability, pricing or route evidence.
The example ID must be exposed by the actual harness before use.

```json
{
  "state": ".agent-teams/model-demo.json",
  "team": {
    "schemaVersion": 1, "id": "model-demo", "outcome": "Clarify setup documentation",
    "scope": "project", "harness": "codex",
    "modelPolicy": {"tier": "medium", "capabilities": ["function_calling"]},
    "skillPolicies": {"documentation": {"capabilities": ["structured_output"]}},
    "roles": [{
      "id": "implementer", "description": "Owns setup documentation",
      "prompt": "Edit assigned documentation and report evidence and remaining work.",
      "skills": ["documentation"], "owns": ["docs/setup.md"],
      "inputs": ["Setup requirements"], "outputs": ["Updated setup"], "dependsOn": [],
      "modelPolicy": {"model": "gpt-6.1-sol"}
    }]
  },
  "roleId": "implementer", "skills": ["documentation"],
  "taskPolicy": {"capabilities": ["reasoning"]},
  "catalog": {
    "schemaVersion": 1,
    "models": [{
      "id": "gpt-6.1-sol", "available": true, "tier": "medium",
      "capabilities": {"function_calling": true, "structured_output": true, "reasoning": true},
      "pricing": {"inputPerMillion": null, "outputPerMillion": null},
      "provenance": {"availabilityBasis": "operator-declared"}, "freshness": {"stale": null}
    }]
  }
}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs models-select --input model-selection.json
```

Inspect `selected`, `policy`, `appliedLayers`, `rejected` and `warnings`. This
example can select the declared ID with unknown pricing because it has no price
ceiling. Capabilities accumulate across all layers; scalar model/tier/rate
ceilings override in team → role → explicitly ordered skills → task order.
Supplying `skills: []` means no skill-policy layer, even if the role lists skills.
A missing/false capability or unknown/disabled availability rejects a candidate.

For a deliberate unknown-price failure, add `"maxOutputPerMillion": 8` to
`taskPolicy`. The candidate must be rejected because output price is unknown;
`selected` is null. Do not weaken an actual budget to make an example pass.
Prices/freshness must come from current authorized evidence when used for work.
`low`/`medium`/`hard` are operator annotations, not effort controls or benchmarks.

Selection is read-only: it does not save its model, change a task or launch an
agent. To persist a reviewed concrete result, use `team-update` with current
state revision and the entire replacement manifest. For example, `init` can use
the same JSON file's `state` and `team`; it ignores
selection-only top-level fields and refuses an existing state. `models-select`
uses the supplied team/catalog and does not read the `state` path. Save
`model-status.json` as `{"state":".agent-teams/model-demo.json"}`, inspect status,
then save `model-update.json` below. The model and native effort are illustrative;
apply only if exposed by the installed Codex version and intended by the owner.

```json
{
  "state": ".agent-teams/model-demo.json", "expectedRevision": 0,
  "team": {
    "schemaVersion": 1, "id": "model-demo", "outcome": "Clarify setup documentation",
    "scope": "project", "harness": "codex",
    "modelPolicy": {"tier": "medium", "capabilities": ["function_calling"]},
    "skillPolicies": {"documentation": {"capabilities": ["structured_output"]}},
    "roles": [{
      "id": "implementer", "description": "Owns setup documentation",
      "prompt": "Edit assigned documentation and report evidence and remaining work.",
      "skills": ["documentation"], "owns": ["docs/setup.md"],
      "inputs": ["Setup requirements"], "outputs": ["Updated setup"], "dependsOn": [],
      "modelPolicy": {"model": "gpt-6.1-sol"},
      "native": {"codex": {"model_reasoning_effort": "high"}}
    }]
  }
}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs init --input model-selection.json
node skills/process/agent-team-creator/scripts/cli.mjs status --input model-status.json
node skills/process/agent-team-creator/scripts/cli.mjs team-update --input model-update.json
```

The replacement keeps historical role IDs and team identity. Inspect returned
state; `team-update` does not regenerate native files. Export to a new staging
directory and review it through the existing export/install procedure. Native
export uses role's explicit model, otherwise team's; it does not rerun skill/task
selection. A role native override can replace the exported model. Read the
actual artifact and worker result rather than relying on portable intent.

A task-specific policy may be recorded under `task.modelPolicy` on `add` and
passed as `taskPolicy` to selection, then applied through a supported native
invocation. There is no generic policy-update action or universal live switch.
Record changed assignments and actual route before dispatch. `modelPolicy`
rejects `reasoningEffort`, `contextWindow`, total-budget and fallback keys;
native settings and plan evidence hold those requirements separately.

Follow the [canonical native reasoning limits](https://prometheus-ags.github.io/prometheus-skill-system/docs/guide/agent-teams#reasoning-and-native-harness-limits)
and [budget/fallback policy](https://prometheus-ags.github.io/prometheus-skill-system/docs/guide/agent-teams#budget-fallback-and-independent-review).
Kimi ignores model frontmatter; DeepSeek export has no per-member model route.
Missing controls leave the route unresolved. liter-llm inference is separate
from an existing tool-enabled worker. No automatic fallback is implemented;
record an explicit authorized alternative, preserving evidence of failed attempts.
A same-family critic does not satisfy this repository's distinct-family KBD QA.

## Two-role handoff request sequence

This standalone example uses a fresh `.agent-teams/setup-review.json` ledger.
Both native harnesses must be available to execute the work; local records alone
need only Node.js 22+. These revisions assume no intervening mutation. Read
`status` before each actual operation and use returned values. Complete all
production in the active phase before local integration and independent review.
Evidence paths below are illustrative: create the actual authorized artifacts
and record results before claiming completion.

Save `review-init.json`:

```json
{
  "state": ".agent-teams/setup-review.json",
  "team": {
    "schemaVersion": 1, "id": "setup-review",
    "outcome": "Clarify setup and independently review its evidence",
    "scope": "project", "harness": "codex",
    "card": {
      "repo": "example/setup-project", "component": "Setup documentation",
      "owns": ["docs/**"], "capabilities": ["documentation"],
      "intake": {"intakeRole": "implementer", "label": "team:setup-review", "rules": []}
    },
    "roles": [
      {
        "id": "implementer", "description": "Owns the complete setup change",
        "prompt": "Edit assigned setup files and record actual local integration evidence.",
        "skills": [], "owns": ["docs/setup.md", "evidence/setup.md"],
        "inputs": ["Setup requirements"], "outputs": ["Setup patch", "Integration record"],
        "dependsOn": []
      },
      {
        "id": "reviewer", "description": "Independently reviews the complete change",
        "prompt": "Read the patch and evidence; write findings only in reviews/setup.md.",
        "skills": [], "owns": ["reviews/setup.md"],
        "inputs": ["Complete patch", "Integration record"], "outputs": ["Review record"],
        "dependsOn": ["implementer"]
      }
    ]
  }
}
```

Run each command when its corresponding step is ready:

```text
node skills/process/agent-team-creator/scripts/cli.mjs init --input review-init.json
node skills/process/agent-team-creator/scripts/cli.mjs task --input implementation-add.json
node skills/process/agent-team-creator/scripts/cli.mjs task --input implementation-start.json
node skills/process/agent-team-creator/scripts/cli.mjs task --input implementation-complete.json
node skills/process/agent-team-creator/scripts/cli.mjs task --input review-add.json
node skills/process/agent-team-creator/scripts/cli.mjs handoff-create --input review-handoff.json
node skills/process/agent-team-creator/scripts/cli.mjs handoff-accept --input review-accept.json
node skills/process/agent-team-creator/scripts/cli.mjs task --input review-start.json
node skills/process/agent-team-creator/scripts/cli.mjs task --input review-complete.json
```

Save `implementation-add.json`:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 0,
  "task": {"action": "add", "id": "setup-patch", "title": "Clarify setup",
    "owner": "implementer", "harness": "codex", "dependsOn": [],
    "remaining": ["Complete production and record actual local integration"]}
}
```

Save `implementation-start.json`, then dispatch the authorized native implementer
with its assigned paths and prohibitions:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 1,
  "task": {"action": "start", "id": "setup-patch", "owner": "implementer", "expectedTaskRevision": 0}
}
```

After coherent phase production and the actual local integration boundary,
save `implementation-complete.json`:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 2,
  "task": {"action": "complete", "id": "setup-patch", "owner": "implementer",
    "expectedTaskRevision": 1, "evidence": ["docs/setup.md", "evidence/setup.md"], "remaining": []}
}
```

Create the separate dependent review task with `review-add.json`. The implementer
initially owns this dispatch record; it may not edit the reviewer's findings path.

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 3,
  "task": {"action": "add", "id": "setup-review", "title": "Independently review setup",
    "owner": "implementer", "harness": "codex", "dependsOn": ["setup-patch"],
    "remaining": ["Independently review the patch and integration record"]}
}
```

Save `review-handoff.json`. Replace `cwd` with the actual authorized checkout:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 4, "cwd": "/path/to/project",
  "handoff": {
    "taskId": "setup-review", "owner": "implementer", "expectedTaskRevision": 0,
    "toOwner": "reviewer", "toHarness": "claude",
    "context": "Production is complete. Independently inspect docs/setup.md and evidence/setup.md. Write only reviews/setup.md; do not edit implementation files.",
    "evidence": ["docs/setup.md", "evidence/setup.md"],
    "remaining": ["Independently review the patch and integration record"],
    "memoryRefs": []
  }
}
```

Creation returns state revision `5`, a generated `handoffs[].id`, and the saved
fresh-context `prompt`; review task revision stays `0`, owned by `implementer`.
Deliver that packet through an authorized native/file channel. In the destination,
read current instructions and inspect Git/evidence availability. Replace the
packet ID below with the returned value; acceptance is a separate deliberate call.

Save `review-accept.json`:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 5,
  "id": "REPLACE-WITH-RETURNED-HANDOFF-ID",
  "destination": {"owner": "reviewer", "harness": "claude"}
}
```

Acceptance returns state revision `6` and review task revision `1`, still pending,
now owned by `reviewer` on Claude. The packet gains `acceptedAt` and an acceptance
event; it does not launch Claude or complete the review. Save `review-start.json`:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 6,
  "task": {"action": "start", "id": "setup-review", "owner": "reviewer", "expectedTaskRevision": 1}
}
```

After independent review actually produces `reviews/setup.md` and all findings
are resolved through authorized production work and required final gates, save
`review-complete.json`. If work remains, block or report it instead.

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 7,
  "task": {"action": "complete", "id": "setup-review", "owner": "reviewer",
    "expectedTaskRevision": 2, "evidence": ["reviews/setup.md"], "remaining": []}
}
```

The final state is revision `8`; implementation task revision is `2` and review
task revision is `3`, both complete. These are expected request results, not
verification evidence. Inspect actual returned records. For KBD-linked work,
attach canonical identity when adding the tasks and use `complete-kbd` with real
receipts in place of ordinary completion; this local example does not create a
canonical run or claim a phase complete.

A later source-task change makes a pending packet stale. Read state and create a
new packet instead of rewriting it. Repeating acceptance is idempotent only while
the accepted task remains at the transferred revision and owner/harness, and the
caller supplies current state revision. After `review-start`, the old accepted
receipt is stale. Administrative `reassign` is immediate and lacks this handshake.
Git snapshots do not copy untracked changes, files, memory content or credentials.

## Cross-project card and request example

Full ships these commands; mini does not. The two-role example includes a card
with repository `example/setup-project`. Replace that illustrative repository
with the actual authorized repository before publication or remote requests.
Cards expose role descriptions, ownership and capabilities, so review them for
private information. Publication does not certify that a team is available.

After the example's final state, save `publish-card.json`:

```json
{"state": ".agent-teams/setup-review.json", "registryDir": "./team-registry", "pk": false}
```

Save `discover-card.json`:

```json
{"registryDir": "./team-registry", "capabilities": ["documentation"], "paths": ["docs/setup.md"]}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs team-publish --input publish-card.json
node skills/process/agent-team-creator/scripts/cli.mjs team-discover --input discover-card.json
```

Publish returns the atomic card file and `pk` outcome. `pk: false` suppresses
optional shared ingestion. Discovery ranks capability hits (2) and path hits (3);
inspect target repository and intake owner, rather than treating ranking as an
execution or permission grant. Specify `target.repo` when team IDs are ambiguous.

Save `cross-project-request.json` to preview an issue request:

```json
{
  "registryDir": "./team-registry",
  "target": {"teamId": "setup-review", "repo": "example/setup-project"},
  "from": {"repo": "example/web-project", "team": "web-team", "role": "implementer"},
  "title": "Clarify the supported setup contract",
  "context": "The web project needs an owner-reviewed setup contract. Please triage in the target project before accepting work.",
  "capabilities": ["documentation"], "paths": ["docs/setup.md"],
  "evidence": ["Source issue URL or reachable authorized artifact"],
  "remaining": ["Destination owner triage and explicit acceptance"], "dryRun": true
}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs team-request --input cross-project-request.json
```

Different repository strings choose the issue route. The preview returns a packet
and quoted `gh issue create` command; it creates neither an issue nor a worker.
Only after explicit authorization for the destination repository's label/issue
writes should a caller change `dryRun` to false. The runtime attempts label
creation and then issue creation using existing `gh` authentication. Missing `gh`
returns the manual command. There is no resident dispatcher or automatic issue
workflow. Native file/message delivery is a separate authorized option.

For the same repository with no rule forcing `issue`, `team-request` instead
requires the target `state`, current `expectedRevision` and `cwd`. It creates one
pending intake task owned by the intake role, a handoff-format record and
`request.sent`/`request.received` events. **`dryRun` does not suppress this local
route.** Its same-owner context record is not ordinary source-to-destination
ownership transfer or confirmation that the intake role agreed to execute.

An authorized destination may import open labelled issues. Save `intake.json`
with the actual current revision; `8` below assumes the completed local example
and no later state mutation:

```json
{"state": ".agent-teams/setup-review.json", "expectedRevision": 8, "ack": false}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs team-intake --input intake.json
```

This explicitly reads up to 200 open issues labelled `team:setup-review` from the
card repository using `gh`, adds pending `issue-<number>` tasks for triage, and
records repository/issue URL events. Re-imports skip existing IDs. `ack: true`
adds comments to newly imported issues; it requires authorization and is not
acceptance. A failed comment does not roll back a committed local import.

Retain source/target repository, project/team/role, request task and packet IDs,
issue URL and actual owner acknowledgement. The same-repository request ID hashes
source identity/title/context; repeated input reuses it, but changing only paths,
evidence or remaining work does not update the old task. Issue creation has no
durable request deduplication. After a timeout or lost response, inspect existing
labelled issues before retrying. Import deduplication is per target state/issue
number. Do not use one state file to impersonate another repository, treat a
request as accepted work, or infer cross-project write authority from a card.

## Scoped lessons and the durable memory outbox

Follow the [canonical scoped-memory guidance](https://prometheus-ags.github.io/prometheus-skill-system/docs/guide/agent-teams#keep-lessons-scoped-to-their-audience)
and [memory tiers](guide/memory-tiers.md). Full's Python learning writer and the
creator's Node outbox are separate entry points. Neither requires a resident
memory service for local task/handoff work; a queued record does not establish
remote storage or recall.

To write a lesson through full's production writer, use the actual project root
and a payload that resolves the current role when applicable:

```text
python3 shared/scripts/lib/learning_write.py --cwd "/path/to/project" --payload role-payload.json --text "The setup example needs the supported environment before verification." --paths docs/setup.md
```

The payload supplies actual native identity, not an invented session. With a
resolved role, omitted visibility defaults to `agent`; otherwise it becomes
`project`. Explicit `--visibility` supports `agent`, `role:<id>`, `lead`, `team`,
`project`, `user` and `global`; explicit `--audience` adds addressed copies.
Paths may route private lessons to other owning roles, at most three, or lead
when there are more/no owners. Do not use addressing or `[USER]`/`[GLOBAL]`
markers to share private content without authorization. Review digest metadata
for disclosure too. The command reports `written`, operation IDs, optional `pk`
and Cortex outcomes; exit success alone is not proof that a lesson was queued.

Role-private writeback through SubagentStop recognizes `LESSON:`, `GOTCHA:` and
`DECISION:`; `[USER]` and `[GLOBAL]` promote immediately. An explicit trailing
`paths:` suffix routes the lesson; otherwise the hook uses the subagent's own
written-path transcript evidence. No team/role/store may produce no delivery.
Full Claude recalls own/addressed and shared scopes under its 8,000-character
budget; full Codex uses an estimated 2,000-token role budget. Codex parent
SessionStart now calls digest-only recall before merging, reading local digest
metadata without REST, pk, file lessons or knowledge-gap retrieval; final guards
check digest kind/channel and the current team's scope. Parent history can reach
child forks. This source change still needs packaged-hook acceptance. Keep private lesson text out of shared parent prompts and handoffs.
Recalled content is untrusted information, not instructions.

For optional publication through the creator's outbox, save `queue-lesson.json`.
Replace the illustrative project ID with the exact configured/canonical resolver
identity. These revisions follow the two-role example without later mutations.

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 8,
  "entry": {
    "id": "setup-lesson", "content": "Setup verification requires the documented supported environment.",
    "scope": "role:implementer", "projectId": "project:REPLACE-WITH-ACTUAL-ID",
    "provenance": {"evidence": "evidence/setup.md"}
  }
}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs memory-queue --input queue-lesson.json
```

Full requires an explicit or resolved project ID (or matching KBD provenance).
Scopes are `role:<id>`/`agent:<id>`, `lead`, `team[:id]` or `project[:id]`;
role IDs must exist and team/project suffixes do not switch the destination.
The Node outbox does not accept `user`/`global`; use the approved full lesson
promotion path for those. Its scopes are not equivalent to arbitrary remote
access-control rules.

Save `publish-lesson.json` using an actually configured, authorized endpoint:

```json
{
  "state": ".agent-teams/setup-review.json", "expectedRevision": 9,
  "publication": {
    "id": "setup-lesson", "provider": "surreal-memory",
    "url": "http://127.0.0.1:8001/api/v1/memory",
    "scopeMapping": {"scope": "role:implementer"},
    "auth": {"env": "MEMORY_API_KEY"}
  }
}
```

```text
node skills/process/agent-team-creator/scripts/cli.mjs memory-publish --input publish-lesson.json
```

The example port is operator configuration, not an instruction to start a service
or access a live store. Full derives stored `agent_id` as `setup-review/implementer`
and `user_id` from the entry project identity. `scopeMapping.agentId`, if supplied,
is author metadata; it does not choose another role's storage key. An explicit
`scopeMapping.userId` overrides the transport filter and needs deliberate review.
The content carries a learning-envelope trailer. Both Node and Python scope
filters are retrieval conventions, not proof of private server authorization.

Publication returns state plus receipt, retaining remote ID/status/uncertainty.
Missing services leave local queue work usable. Identical queue input reuses its
ID; changed content under that ID fails. Published entries stay in the outbox and
repeat publication returns the saved receipt. Uncertain outcomes require remote
reconciliation before explicit `retryUncertain: true`; crashes can still separate
remote commit from local persistence. Target/mapping changes cannot silently
redirect a recorded retry. No exactly-once guarantee is claimed.

Cortex is optional and not part of lookup priority (surreal-memory, then pk, then
scoped file fallback). Full's writer admits detached feeder/server pairs under a
shared capacity limit and finite deadline. `cortex_mirror.status: accepted` means
input accepted by the feeder, not memory stored or recalled. Absence, disabled
state or saturation can skip the mirror while primary queue success remains.
See [bounded Cortex outcomes](guide/memory-tiers.md#optional-cortex-mirror).

Never place credentials, private session tokens or unauthorized personal data in
content, provenance, paths or portable prompts. A reference conveys no retrieval
permission. The creator does not complete canonical KBD work, emit Karpathy
boundaries or rewrite `pk` bundles when it publishes a lesson.

## Maintainer boundary

Maintainers edit `skills/process/agent-team-creator/runtime/src/*.mts`. The local runtime package pins TypeScript 7.0.2 and uses NodeNext imports ending in `.mjs`; its build emits the shipped `scripts/*.mjs`. After coherent implementation, compile locally with `npm run build --prefix skills/process/agent-team-creator/runtime`, regenerate each pack's admitted changes at the final production boundary, then run the planned integration and documentation gates. Full-only cards and current memory differences remain explicit; do not copy the entire runtime to infer parity. End users run the shipped JavaScript. Source inspection and a local compile do not certify Windows, an installed native CLI, live authentication or service execution; consult the current validation receipts for the evidence actually collected.

Guided creation resolves ownership before producing a ready team. For example, after reviewing proposed roles, add `"ownership": {"implementer": ["src/checkout/**"], "reviewer": ["reviews/checkout.md"]}` for a two-role checkout task. Use paths actually appropriate to the project and include every proposed role. Missing ownership returns `ready: false`, `proposedRoles`, and focused questions without a `team` value; it does not grant access to the whole repository.
