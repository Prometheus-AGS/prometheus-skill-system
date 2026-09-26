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

## From an outcome to a proposal

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

Targets are UAR, Codex, Claude Code, Copilot, Kimi Code, MiniMax, OpenCode, DeepSeek Harness and BossFang. The adapter uses each tool's native format. Claude and Kimi have alternative agent plugin/marketplace artifacts; Codex uses standalone native agent TOML without an invented plugin `agents` field. MiniMax agent files belong under its active user-data directory. Kimi does not apply per-role model frontmatter, and DeepSeek's experimental team composition does not create a roster or select per-member models. UAR and BossFang exports contain service registration payloads; BossFang Hand activation is a separate action that may start schedules.

Native role overrides, team options and opaque files preserve settings beyond the common manifest, with source/version provenance. Preservation does not certify a setting against an installed tool. The [native contract reference](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/agent-team-creator/references/native-harnesses.md) links official sources and explains which options become proposed config files and which remain sidecar data for deliberate application.

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

## Models, ownership and continuity

Model selection combines team, role, skill and task policy. Strength tiers are explicit annotations; they are not inferred from a model's name. Unknown prices cannot satisfy a cost ceiling, and unavailable capability metadata remains unknown. Discovery is not a successful inference test. Review the selected exact model ID before updating policy and exporting it.

Tasks have an owner, dependencies, revisions and completion evidence. Handoff creation captures fresh context, remaining work and Git state while the source retains ownership. The destination must accept a matching, current packet before ownership changes. Session IDs, credentials and permissions are not portable authority. Local locks protect state updates but do not provide a distributed lease or interrupt a native process.

KBD-linked tasks keep canonical project/run/phase/change/task identity. Their completion uses the real typed KBD command and receipt, following the existing QA and review lifecycle. Team state does not replace canonical progress or mark a parent phase complete. See [process skills](09-process-skills.md) for the surrounding lifecycle.

Memory is optional. Publication failures leave a durable local outbox; uncertain remote outcomes require reconciliation before retry. The runtime supports the verified surreal-memory REST contract or an explicit HTTP mapping, not automatic discovery of arbitrary MCP tools. Team records do not directly write Karpathy projections or the `pk` knowledge bundle.

Maintainers author TypeScript 7 `.mts` sources and ship compiled `.mjs` alongside the skills. Native format sources and local validation receipts have separate meanings: neither a source link nor compilation claims live service acceptance, installed CLI compatibility or Windows certification.

Guided creation resolves ownership before producing a ready team. For example, after reviewing proposed roles, add `"ownership": {"implementer": ["src/checkout/**"], "reviewer": ["reviews/checkout.md"]}` for a two-role checkout task. Use paths actually appropriate to the project and include every proposed role. Missing ownership returns `ready: false`, `proposedRoles`, and focused questions without a `team` value; it does not grant access to the whole repository.
