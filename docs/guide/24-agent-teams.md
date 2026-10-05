---
title: Agent teams
description: Choose a small team, assign ownership, select models and hand work between native coding tools.
---

# Agent teams

Start with the outcome you want, then choose the smallest team that can deliver it. For a focused change, one implementer can use specialist skills as needed. Add another role when it has a distinct deliverable, separate files to own, or an independent review to perform. More agents also add coordination and model cost.

The four procedures are available as ordinary skills through the process plugin and the pack's skill distribution:

| Procedure | Use it to |
| --- | --- |
| `agent-team-creator` | Describe an outcome, stage native definitions, then install or adopt the project team. |
| `agent-team-manage` | Assign and track revisioned tasks, dependencies and evidence. |
| `agent-team-models` | Compare configured models against explicit capabilities, tiers and price ceilings. |
| `agent-team-handoff` | Prepare fresh context and transfer ownership after destination acceptance. |

## Find your responsibility

Use the same selected project team across these levels. A person or agent may
perform several responsibilities, but each task still needs an explicit owner.

| Level | What you decide or record | Start here |
| --- | --- | --- |
| User or project owner | Desired outcome, scope, budget, permitted destinations and requested review | [Propose a team](#from-an-outcome-to-a-proposal) |
| Team lead | Selected manifest, bounded assignments, dependencies, active writers and completion evidence | [Install or adopt](#install-or-adopt-a-project-team), then [task lifecycle](#run-the-revisioned-task-lifecycle) |
| Role owner | Assigned paths, current task identity, progress, blockers and evidence | [Ownership](#choose-roles-and-keep-one-writer-per-path) and [team updates](#communicate-within-the-team) |
| Project maintainer | Discovery records, local ledger and canonical KBD links | [Records](#choose-roles-and-keep-one-writer-per-path) and [identity recovery](#preserve-canonical-identity-and-recover-deliberately) |
| Cross-project coordinator | Authorized request, destination intake, correlation IDs and acknowledgement | [Project requests](#route-requests-to-another-project) |
| Harness operator | Discovered native roles, available dispatch tools, permissions and actual model route | [Native discovery](#check-native-discovery-before-dispatch) and [model selection](#choose-a-model-for-the-task) |
| Service operator | Optional endpoints, credentials, data ownership and recovery | [Services and ownership](/docs/guide/service-operations#choose-services-for-team-work) |

For a first local team, follow proposal, project installation, path ownership and
task lifecycle in that order. Read the handoff section when ownership or harness
changes. Add model discovery, shared memory or connected services only when the
work needs those capabilities. The [team overview](/docs/agent-teams/overview)
provides shorter routes through this handbook.

## From an outcome to a proposal

Before creating state, read the project's instructions and existing
`.agent-team/project-routing.json` and `.agent-team/<id>/team.json` files. Reuse
the selected team when it fits. You need Node.js 22+ and the complete creator
payload; Git is optional for snapshots. A native coding tool is needed to execute
work. Model endpoints, shared memory and UAR/BossFang services are separate,
optional dependencies. Full includes broader learning integrations; mini keeps
its Node-only runtime and two optional services, surreal-memory and liter-llm.
Neither pack starts a service or agent merely by creating a local team.

Ask `agent-team-creator` to help with a concrete result, such as making a settings page keyboard accessible. It asks about scope, deliverables, budget, review and the coding tool you will use. A simple change defaults to one implementer; independent review is an explicit choice. Review suggested skills against what is installed, and assign paths in every editing role's `owns` list before parallel work. Inputs, outputs and dependencies make the expected handoff explicit.

Experts can supply a team manifest directly. The shared runtime accepts JSON request files:

```sh
node skills/process/agent-team-creator/scripts/cli.mjs guide --input guide-request.json
```

From an installed skill, use its actual `agent-team-creator/scripts/cli.mjs` path. Node.js 22+ is required; the shipped JavaScript needs no TypeScript installation or runtime packages. For example, `guide-request.json` can contain:

```json
{
  "id": "settings-accessibility",
  "outcome": "Make settings usable with a keyboard",
  "complexity": "simple",
  "areas": ["code"],
  "deliverables": ["Accessible settings page", "Verification record"],
  "budget": "balanced",
  "review": false,
  "harness": "codex",
  "scope": "project"
}
```

The result includes an editable team, reasons and alternatives. An empty request returns the intake questions. Save the reviewed manifest through `init`; use `status` to retrieve current revisions before later mutations. The [repository request reference](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md) provides complete initialization, export, task and handoff examples.

## Export is a review boundary

An `export` request names the team or state file, target and a new output directory. The runtime stages native definitions and provenance receipts. It refuses existing output directories and path or filename collisions. Review selected files before installing them; export does not install plugins, register service agents or start execution. Native permissions and invocation rules remain authoritative.

Targets are UAR, Codex, Claude Code, Copilot, Kimi Code, MiniMax, OpenCode, DeepSeek Harness and BossFang. The adapter uses each tool's native format. Claude and Kimi have alternative agent plugin/marketplace artifacts; Codex uses standalone native agent TOML without an invented plugin `agents` field. MiniMax agent files belong under its active user-data directory. Kimi does not apply per-role model frontmatter, and DeepSeek's experimental team composition does not create a roster or select per-member models. UAR export remains legacy per-agent AgentArtifact staging; draft.2 packages and private bindings use the separate file-backed flow below. BossFang Hand activation is a separate action that may start schedules.

Native role overrides, team options and opaque files preserve settings beyond the common manifest, with source/version provenance. Preservation does not certify a setting against an installed tool. The [native contract reference](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/agent-team-creator/references/native-harnesses.md) links official sources and explains which options become proposed config files and which remain sidecar data for deliberate application.

## UAR draft.2 file-backed teams

Legacy `export --target uar` still creates per-agent AgentArtifact staging files. Canonical teams use the provider-owned draft.2 profile with `workspace.json`, `manifest.source.json`, and separate source documents such as `agents/coordinator.json`, `agents/child.json`, `teams/root.json`, `teams/subteam.json`, and `workflows/review.json`.

```text
node skills/process/agent-team-creator/scripts/cli.mjs uar-workspace-init --input skills/process/agent-team-creator/assets/uar-intake.json
node skills/process/agent-team-creator/scripts/cli.mjs uar-workspace-status --input workspace-status.json
node skills/process/agent-team-creator/scripts/cli.mjs uar-workspace-update --input update-one-document.json
node skills/process/agent-team-creator/scripts/cli.mjs uar-package-build --input workspace-build.json
```

Workspace status stays bounded: fixed counts, one next question, and paged field diagnostics. Validation requires one root TeamDefinition, kind-correct references, valid workflow roles, acyclic dependencies, and exact versions and digests. Draft.1 and inline callers migrate explicitly and retain field-level loss reports.

The compiled package is portable immutable catalog data. DeploymentBinding is private installed state and carries opaque host references; package installation does not confer credentials, RepresentationGrants, consent, authority, or activation. The draft.2 checkpoint does not claim durable team execution, and `uar-activate` refuses rather than reporting an unobserved runtime result.

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

See [UI/UX routing](25-ui-ux-routing.md) for the selective workflow.

## Choose roles and keep one writer per path

A role has an ID, prompt, installed skills, `owns`, inputs, outputs and role
dependencies. Use project-relative paths or globs for write ownership. Assign a
reviewer its own findings path, such as `reviews/settings.md`; it may read the
implementation without editing the implementer's files. An empty ownership list
does not authorize unrestricted edits. The expert schema permits empty arrays,
but guided creation asks you to resolve ownership before returning a ready team.

The portable team manifest, local task state and project routing record each use
schema version `1`. They serve different purposes:

| Record | Purpose |
| --- | --- |
| `.agent-team/<id>/team.json` | Project discovery manifest: roles, ownership and native bindings |
| `.agent-team/project-routing.json` | Selected manifest and installed native definition paths |
| `.agent-teams/<id>.json` | Mutable task ledger chosen by your `state` request |
| Export directory | Proposed native artifacts and provenance receipts |

Import a reviewed portable manifest by placing it under `team` in an `init` or
`install-project` JSON request. There is no universal native-agent import command.
Keep native fields under `native.<target>` with source/version provenance rather
than treating a native file as a portable team manifest. The separate UAR draft.2
profile uses immutable agent, team and workflow definitions; it is not schema-v1
task state. Read the [manifest schema](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/agent-team-creator/schemas/team.schema.json)
and [project installation requests](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/agent-team-creator/references/project-installation.md)
before replacing a definition.

Role dependencies describe the team. A task must separately name task IDs in
`dependsOn`; the runtime does not create those edges from role dependencies.
Assign separate implementation and review tasks. Keep review dormant until all
production work in the active phase is complete, then use the project's required
local integration and independent review boundary.

## Run the revisioned task lifecycle

Save requests as JSON and call the creator's packaged runtime. The
[repository request reference](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#task-lifecycle-request-sequence)
contains a complete add/start/block/resume/complete sequence, with cancellation
and reassignment alternatives. The four skills share this runtime; installing
four skills does not create four agents.

| Step | Command or action | What it records |
| --- | --- | --- |
| Inspect | `status` | Current state revision, tasks, owners and task revisions |
| Assign | `task` with `action: "add"` | A pending task owned by an existing role |
| Start | `task` with `action: "start"` | Running state after dependencies complete |
| Update | `block`, `start` or `reassign` | Evidence and explicit remaining work on a supported transition |
| Cancel | `task` with `action: "cancel"` | Terminal cancellation and a reason |
| Complete | `task` with `action: "complete"` | Running task, evidence and no remaining work |

There is no generic `update` action. Every mutation after initialization supplies
the current `expectedRevision`; existing-task operations also supply current
`owner` and `expectedTaskRevision`. Read `status` before each mutation and
reconcile conflicts instead of incrementing a guessed revision. A successful
task change increments state and task revisions once. `add` starts a task at
revision `0`; completed and cancelled tasks remain terminal.

An owner string is a claim made by a cooperating caller, not authentication or a
distributed lease. Assignment and `start` do not spawn a process. Cancellation
does not interrupt it. Dispatch the native worker separately with the exact
project directory, assigned paths, prohibited paths, deliverable and current
task identity. Stop the old worker before reassignment permits another writer.
Use sequential role execution and disclose it when native delegation is absent.

### Example: a bounded dispatch

Suppose the lead assigns `settings-copy` to `implementer`, with ownership of
`src/settings/copy.ts`. The native dispatch should carry a brief such as:

```text
Project: /absolute/path/to/project
Team: the selected project team
Task: settings-copy; owner: implementer
Revisions: copy the current state and task revisions from status
Own: src/settings/copy.ts
Do not edit: src/settings/layout.tsx or another role's assigned files
Deliver: the agreed settings text and the changed-file references
Report: evidence, blockers and remaining work; preserve other contributors' edits
```

This is a dispatch brief, not CLI input. Add the real canonical task identity
when linked to KBD, the selected model route and any task-specific constraints.
The lead records `start` after dependencies permit it and invokes an authorized
native worker separately. The role reports findings; the lead reconciles them
with the ledger before recording completion. If work is blocked, preserve the
reason and remaining work. A later `start` resumes a blocked task; a terminal
task needs a new follow-up task.

If review is required, give the reviewer a separate dependent task and its own
findings path. Dispatch it at the project's completed-phase review boundary.
If the same unfinished task must move to another role or harness, use the
acceptance flow below before the new owner edits.

Local `add` and `reassign` are administrative assignments; neither requires a
destination acceptance packet. A context-bearing `handoff-create` retains source
ownership until `handoff-accept`. Direct UAR coordinator acceptance is a third,
provider-schema concept, described below. These are distinct boundaries.

## UAR direct coordinator acceptance

The vendored draft.2 [TeamDefinition schema](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/d8896d743cd945d40f918ff8ca397909f6c1fe22/docs/agents/collaboration/v0.1.0-draft.2/schemas/team-definition.schema.json)
uses the field `taskAcceptance`, not a local `task-accept` command. This fragment
belongs inside a complete draft.2 TeamDefinition:

```json
{
  "taskAcceptance": {
    "mode": "coordinator-within-binding",
    "allowedWorkflows": []
  }
}
```

Coordinator mode permits an empty workflow list, allowing a definition to
represent direct coordinator tasks within its binding. Operator mode requires
at least one immutable workflow reference. An empty list is not a wildcard
workflow grant. Routing requires `eligibilityFirst: true`, strategy
`operator-role-capacity-cost-stable-id`, and `explain: true`.

The consumer receipt pins provider commit
`d8896d743cd945d40f918ff8ca397909f6c1fe22` as a source-contract checkpoint. This is
schema support, not durable execution or live acceptance. It does not replace
local `handoff-accept`, grant credentials, activate a team or establish that a
resident UAR implements the policy. The shipped `uar-activate` path still refuses
activation. Check the provider contract and deployed capabilities before use.

## Preserve canonical identity and recover deliberately

For KBD-linked tasks, retain the exact `projectId`, `runId`, `phaseId`, `changeId`
and `taskId`. A displayed title, task index, inferred next command or local team
ID is not a canonical task selector. The canonical task must already be
`in_progress`, or already complete when reconciling a confirmed result, before
the team's `complete-kbd` adapter records local completion.
Ordinary local `complete` rejects KBD-linked tasks.

The adapter executes the explicit `kbdCli`, reads canonical status, verifies all
five identities, and records the committed receipt before local completion.
Local and canonical state are separate stores: a timeout or crash may leave KBD
complete while the local task remains running. Read both stores and reconcile
the confirmed result before retrying. Preserve the project’s claim, stage,
permission, evidence and phase gates; team assignment grants none of them.

Local mutations use an exclusive `<state>.lock` and atomic replacement on one
filesystem. They do not support shared multi-machine writers or automatic stale
lock takeover. If a writer crashes, inspect its recorded PID, time and token,
confirm it has stopped, then remove only the abandoned lock. Re-read state and
uncertain external outcomes before retrying. Read the
[task and recovery contract](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/agent-team-creator/references/task-handoff.md)
for exact KBD requests and recovery steps.

## Check native discovery before dispatch

Portable definitions preserve native options; they do not override the installed
harness's permissions. Current [Codex role configuration source](https://github.com/openai/codex/blob/main/codex-rs/core/src/agent/role.rs)
applies a role configuration layer when a role is spawned. Review generated TOML
against the installed Codex version before invoking it; do not invent a plugin
`agents` field or infer execution from files on disk.

Claude Code [project subagents](https://code.claude.com/docs/en/sub-agents) use
Markdown with frontmatter in `.claude/agents/`. Native tools and permission modes
remain authoritative. [Experimental agent teams](https://code.claude.com/docs/en/agent-teams)
are a separate feature; exporting subagent files does not enable or create one.
[OpenCode custom agents](https://github.com/anomalyco/opencode/blob/dev/packages/web/src/content/docs/agents.mdx)
use `.opencode/agents/*.md`; native `permission` rules govern tools and which
subagents a primary agent may invoke. [Kimi custom agents](https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/agents.md)
use native Markdown definitions, including project `.kimi-code/agents/` discovery;
exported model frontmatter does not establish per-role model selection. DeepSeek's
[experimental agent-team service](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/experimental/agent-team/README.md)
uses runtime member creation, rather than a portable manifest that launches its
roster. Read the linked native contract before enabling it.

Record the actual native tool/version and discovered role. Report missing
discovery or unsupported settings explicitly; an exported file is not proof of
an available worker.

## Communicate within the team

Use native messages for coordination and the revisioned ledger for ownership.
A useful update names project/team/task, current owner and revision, what changed,
evidence, remaining work and the next requested action. A message saying “please
review” is not a task start, accepted handoff or completion receipt. Save durable
artifact references in task evidence; do not rely on a conversation that another
harness cannot read. The creator does not supply a universal chat or dispatch API.

Use an authorized local file or the native harness's available communication
mechanism. Follow the destination tool's messaging permissions; do not send
Slack/email or another project's messages without explicit authorization. Keep
one writer per path while replies are pending. If delegation is unavailable,
disclose sequential execution and preserve the same ownership boundaries.

## Transfer context and wait for acceptance

`handoff-create` captures a task's current owner/revision, destination role/harness,
context, evidence, remaining work, memory references and a fresh-context prompt.
It records Git root, HEAD, branch and dirty state read-only. Unknown Git fields
stay unknown; dirty state is not a patch, and the packet copies no files. Make
artifacts reachable through an authorized checkout or file transfer separately.

The source keeps ownership until the exact destination calls `handoff-accept`
with the generated packet ID and current state revision. Acceptance commits the
receipt and ownership change together. A running source task becomes pending;
the destination starts it separately after dependencies complete. Native source
work must stop before the destination edits. Acceptance does not stop a process,
resume a portable session, grant permissions or prove completion.

A state revision conflict requires a fresh `status` read. If the task owner,
harness or revision changed after packet creation, the packet is stale: inspect
both records and create a new packet from current context. Do not patch historical
packets or replay an accepted receipt after later task changes. Duplicate
acceptance is a no-op only while ownership/harness/task revision still match the
accepted snapshot, with the current state revision supplied. Cancelled and
completed tasks remain terminal. Administrative `reassign` changes ownership
immediately and carries none of this acceptance handshake.

The [two-role request sequence](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#two-role-handoff-request-sequence)
creates an implementer and reviewer, completes coherent production and its local
integration evidence, then transfers a separate dependent review task from Codex
to Claude Code. It shows every revision through acceptance, start and completion.
Mini has the [same local handoff sequence with mini paths](https://github.com/Prometheus-AGS/prometheus-skills-mini/blob/main/docs/agent-teams.md#two-role-handoff-request-sequence).
These are request examples, not recorded successful native runs.

For UAR, include exact workspace/package/definition identities and digests,
private binding revision and migration receipt references. Context transfer does
not mutate immutable definitions, install bindings or carry credentials,
RepresentationGrants or consent. The draft.2 `taskAcceptance` schema remains
separate from this local handoff receipt and unsupported durable activation.

## Route requests to another project

First identify the actual repository, project ID, selected team and intake role.
A similarly named team or shared directory is not the same repository. Ask the
owner for authorization before messaging or writing another project; a card,
issue, request packet or ownership glob provides discovery, not permission.

Full currently ships `team-publish`, `team-discover`, `team-request` and
`team-intake`. A reviewed `team.card` records repository, component, owned paths,
capabilities, intake role, label and routing rules. Cards publish to a local
registry (optionally `pk`); discovery ranks capability/path matches, not verified
availability. Repository equality is a literal card/request string comparison,
not a remote identity proof. Inspect stale or ambiguous cards before selecting a
target. The [full request examples](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#cross-project-card-and-request-example)
show explicit publication, discovery and an issue-route preview.

| Route | Shipped behavior and boundary |
| --- | --- |
| Full, same repository, no forcing rule | Adds a pending intake task and handoff-format context record to the target team's local state. It does not transfer an existing source task or obtain destination acceptance. |
| Full, another repository or matching `issue` rule | Explicit `team-request` invokes `gh` to create the team label and issue. Requires authorized GitHub access. It creates no remote worker or accepted task. |
| Full issue intake | Explicit `team-intake` imports up to 200 open labelled issues into pending `issue-<number>` tasks for triage. `ack: true` comments only on newly imported issues; import/ack is not acceptance. |
| Mini | Does not ship card/discovery/request/intake commands. Deliver reviewed context manually through an authorized native/file/issue channel; the destination creates its own local task and canonical identity. |

For the issue route, `dryRun: true` returns the packet/quoted command without a
GitHub write; missing `gh` also returns a manual command. **`dryRun` does not
suppress the same-repository local route.** A live request must be explicitly
authorized, including label/issue writes. This is invoked issue creation and
intake, not a background GitHub automation or cross-project executor.

Save source repository/team/role, target repository/team/intake role, task/packet
IDs, issue URL, local events and owner acknowledgement as correlation evidence.
Same-repository requests derive `req-<hash>` from source identity/title/context;
repeats reuse that task, so changes to other fields do not update it. Issue
creation has no durable request-deduplication receipt. If it times out or the
result is lost, inspect labelled issues before retrying. Intake deduplicates
within its target state by issue number; a failed acknowledgement needs separate
reconciliation. A request becoming a pending intake task still needs owner
triage, canonical claims and an explicit start. Local handoff is never automatic
cross-repository ownership transfer.

## Keep lessons scoped to their audience

Use the [memory tiers guide](memory-tiers.md) for operational details and the
[team-aware memory design](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/design/team-aware-learning-memory.md)
for identity, addressing and promotion. The table describes full's current
learning writer and recall scopes; these are routing/filter conventions, not a
server authorization guarantee.

| Visibility | Intended audience |
| --- | --- |
| `agent` | Authoring team role; default when that role resolves |
| `role:<id>` | Explicitly addressed role in the same team |
| `lead` | Team lead scope |
| `team` | Shared team scope and digest |
| `project` | Shared project scope; default when no role resolves |
| `user` | User-wide promoted knowledge across projects |
| `global` | Explicitly promoted knowledge usable across projects |

Full resolves project/team/role from the project resolver, selected manifest and
native role/path evidence. Path ownership can address a role-private lesson to
other owners: at most three recipients; more matches or no matching owner route
to the lead. Paths owned only by the author keep the lesson private. A team digest
contains author, paths and content hash, without lesson text. Review filenames
and metadata before sharing too; even a digest can disclose project information.

Full's SubagentStart recalls the role's own/addressed lessons plus allowed shared
scopes, within 8,000 characters on Claude or an estimated 2,000-token Codex budget.
Claude's team SessionStart receives lead scope and team digest. Codex's current
source calls digest-only recall before merging: local digest candidates only,
without REST, pk, file lessons or knowledge-gap retrieval. A final guard checks
digest kind/channel and the selected team's digest scope. Parent history may be
forked into child roles, so this prevents shared lesson text from replacing its
digest during merge. Packaged-hook acceptance remains a separate final gate.
Do not paste a private lesson into the parent or a shared handoff to bypass that
boundary. Delivery is fenced untrusted information, not instructions; absent
teams/roles/stores can yield no context. Inspect actual delivery when required.

Lookup priority is surreal-memory MCP, then `pk`/knowledge tools, then the scoped
learning log and small project index. Query the current project/role rather than
reading an entire private store. Cortex is not a lookup fallback. Keep role
lessons private by default; use explicit authorized addressing and approved
promotion. Detectors propose broader promotion for human confirmation; the lead
or reflector may promote within the team. `[USER]` and `[GLOBAL]` final-message
markers promote immediately in full's hook, so use them only when intended.

Full writes through `shared/scripts/lib/learning_write.py`; SubagentStop recognizes
`LESSON:`, `GOTCHA:`, `DECISION:`, `[USER]` and `[GLOBAL]`. Queue success and log
records are durable local evidence, not proof of remote recall. Optional `--pk`
starts separate ingestion. The writer's optional Cortex mirror is bounded by
feeder slots and a finite deadline; saturation/absence can skip it. Its
`accepted` status means feeder input accepted, not server storage. See the memory
tiers guide for exact outcomes, configuration and scratch-only verification.

Mini keeps Node-only delivery: its Claude SubagentStart reads the role's local
`MEMORY.md` and digest file under an 8,000-character fence. It does not ship full's
Python writer, scoped store recall, Codex digest SessionStart or Cortex feeder.
Its corrected Node outbox source uses canonical `/api/v1/memory`, derived
team/project scope keys and a learning-envelope trailer. It needs an explicit or
selected project identity, keeps missing-identity entries queued without network
I/O, and preserves recorded legacy attempt fingerprints for deliberate
reconciliation. This does not add full's hook recall or privacy guarantees.
Compiled payload regeneration and production integration remain deferred.
Neither pack requires a memory service for local handoffs.

Keep secrets, personal data and portable session credentials out of packets,
lesson content, provenance and digest paths. Scope labels and source receipts do
not prove access control. A memory reference is a pointer; it transfers neither
private lesson content nor permission to retrieve it.

## Choose a model for the task

KBD planning chooses demonstrated task fit first, within user selections,
project policy and budget; cost and latency break close ties. The team's
`models-select` helper instead filters declared constraints and then orders
eligible candidates by lowest known input-plus-output price, with exact ID as
its tie-break. Use that helper to explain constraints, not to replace the KBD
quality-first decision or infer a permanent model ranking.

Record reasoning difficulty, uncertainty, tools, context/output size, vision or
other modalities and review independence before choosing. `low`, `medium` and
`hard` are declared tiers; they are neither benchmark scores nor native reasoning
effort. A tier matches exactly. Document unsupported or unknown requirements;
a role title, model name or catalog listing does not prove suitability.

| Evidence | What it establishes |
| --- | --- |
| Catalog or operator declaration | Candidate identity and declared metadata |
| Configured discovery response | Endpoint-listed eligibility at that time |
| Native schema/config inspection | An exposed control, not an applied session |
| Successful inference | That request reached a working inference route; retain actual identity when available |
| Tool-enabled worker result | Actual workspace/tool execution under the recorded route |
| Independent review/integration | Only the specific boundary and evidence actually exercised |

Both packs use `models-discover` and `models-select` through the creator runtime.
Local declared-catalog comparison needs Node.js 22+ only. Live discovery needs an
explicit compatible gateway or service endpoint and authorized credentials by
environment reference. Neither command starts a model service, runs inference
or launches a worker. liter-llm supplies inference; an existing documented
worker with tools supplies execution. An alias that reaches a generic proxy is
not proof that the requested provider/model answered.

## Inspect effective policy before binding

Portable policy allows only `model`, `tier`, `capabilities`,
`maxInputPerMillion` and `maxOutputPerMillion`. It resolves in this order:

1. Team `modelPolicy`.
2. Selected role `modelPolicy`.
3. `skillPolicies` for the explicitly supplied skills, in their supplied order.
4. Explicit `taskPolicy` passed to selection.

Later scalar values replace earlier values; capabilities accumulate and every
required capability must be explicitly true. Empty task capabilities do not
clear team requirements. Missing capability/availability evidence fails that
constraint. Skill selection does not read arbitrary skill frontmatter or
silently use every role skill; the caller supplies the ordered list.

There is no portable `reasoningEffort`, `contextWindow`, total-dollar budget or
`fallback` field. Keep those requirements and approved fallback decisions in the
plan/native configuration. Unknown keys fail validation. Context sufficiency
must be checked against the actual model/version and effective native limits;
adding a `reasoning` capability does not set effort or guarantee context size.

Inspect `selected`, effective `policy`, `appliedLayers`, `rejected` and `warnings`.
The helper returns no match when constraints cannot be established. It does not
mutate state or configure an agent. The
[full executable selection and persistence example](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#model-discovery-selection-and-persistence)
and [mini equivalent](https://github.com/Prometheus-AGS/prometheus-skills-mini/blob/main/docs/agent-teams.md#model-discovery-selection-and-persistence)
show declared metadata, layered policy, an unknown-price failure and a reviewed
state update. Example declarations are not measurements or native acceptance.

Native export uses the explicit role model, otherwise the team model. It does
not execute the skill/task selector for each export. Persist the reviewed
concrete result in a role/team policy, or carry task-specific policy to the
actual invocation and assignment record. A task may retain `modelPolicy` when
added, but that does not switch its process. Role native overrides can replace
exported model fields; inspect the proposed artifact and actual worker result.

## Reasoning and native harness limits

Read current primary documentation and inspect the installed version/tool schema
before applying a selected model. These controls are native and have different
names and inheritance rules:

| Harness | Model and reasoning boundary |
| --- | --- |
| Codex | Role config supports `model` and `model_reasoning_effort` when exposed by the installed version. Fresh-agent override availability comes from the current tool schema; full-history forks may inherit and reject overrides. Only exposed supported IDs/efforts are usable. |
| Claude Code | Subagent `model` accepts supported aliases, full IDs or `inherit`; invocation/provider/organization settings can affect the result. Reasoning/effort controls are version/model dependent. Inspect the actual running model, not an alias alone. |
| OpenCode | Configured `providerID/modelID` identifies the native model. Per-agent model and model-specific options/variants are distinct controls; an inherited variant is not guaranteed after selecting a different agent model. |
| Kimi Code | Role model frontmatter is ignored. Use verified invocation/global model-pool controls separately; do not invent per-role effort/model switches. |
| DeepSeek Harness | Current persona/experimental team export does not select models per member. Global provider/model settings do not prove distinct member routing, remote workers or worktree isolation. |

Sources: [Codex role overrides](https://github.com/openai/codex/blob/main/codex-rs/core/src/agent/role.rs),
[Claude subagents](https://code.claude.com/docs/en/sub-agents),
[OpenCode models](https://github.com/anomalyco/opencode/blob/dev/packages/web/src/content/docs/models.mdx)
and [agent configuration](https://github.com/anomalyco/opencode/blob/dev/packages/web/src/content/docs/agents.mdx),
[Kimi agent fields](https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/agents.md),
[DeepSeek experimental team contract](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/experimental/agent-team/README.md).
Upstream docs establish supported contract shapes; they do not certify the
installed native tools. Report unresolved routes explicitly.

For a Codex role, `native.codex.model_reasoning_effort` can preserve a reviewed
native setting alongside its explicit model. That opaque preservation is not a
universal effort field or proof the model accepts it. Claude's alias inheritance
and OpenCode's provider variants similarly need actual version-specific review.
Moving a task across harnesses requires a fresh model/permission/context check,
not reuse of an incompatible native field.

## Budget, fallback and independent review

Prices are USD per million text tokens in the helper. liter-llm catalog prices
start per token and are multiplied by 1,000,000; the adapter uses the maximum
base/context-tier rate, leaving a side unknown if any relevant tier rate is
unknown. Missing prices cannot satisfy a ceiling. Stale or unknown freshness
produces warnings, not an automatic refreshed rate or guaranteed bill.

Rate ceilings do not cap total spend. Include expected input/output volume,
reasoning usage, tool retries, review, concurrency and provider extras when
estimating a budget. The helper's price sum assumes no workload token mix;
it is a deterministic comparison, not the cost of this task. Context, cache,
audio/image, subscription rules and future charges need their own evidence.

If a route fails or no model qualifies, record the reason and leave it unresolved.
The runtime has no automatic fallback chain. Re-select only through an explicit
recorded decision that respects the user's constraints; a listed native
alternative is not permission to substitute silently. Preserve previous
attempt/route evidence, reconsider tools/context and prices, and record the
actual model after the new worker responds. Other eligible work may continue.

Independent review needs a separate context and the required model independence.
This repository's KBD rule requires a critic outside the producer's family;
a different alias, reasoning setting or same-family model is not a substitute.
Record producer and critic identity plus the serving route. An alias comparison
or `verified-distinct` name result alone does not prove underlying family
independence through a proxy. If the eligible reviewer is unavailable, leave the
review pending until a valid route or explicitly authorized exception exists.
All integration/review work waits for complete phase production.

## Persist canonical task assignments and actual routes

The KBD assignment key is full phase path + change ID + exact backend task ID.
Retain canonical project/run identity too. `Task model assignments` is authored
plan prose, not a new task schema or launcher. Reconcile split/changed tasks and
record provider/model, supported effort, dated rationale, native/worker route,
working directory/ownership, actual availability, alternative and prerequisites.
The driver owns canonical start/end; worker results return as evidence.

For cross-project requests, source model preferences are context, not destination
policy. The target coordinator triages scope, selects against its own configured
models/budget/tools and records its own canonical task assignment. A team card,
issue or UAR direct-task schema policy does not bind a remote model or authorize
its inference. Revalidate on acceptance or cross-harness transfer; keep actual
source and destination routes distinct in receipts.

See [task model assignments](https://prometheus-ags.github.io/prometheus-skill-system/docs/kbd/task-model-assignments)
for the scoped table and execution record. Planning, configured access, source
implementation, regenerated payloads, actual execution and final acceptance are
separate evidence states. Neither this handbook nor an export claims a live run.
