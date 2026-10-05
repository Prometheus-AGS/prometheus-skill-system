---
title: Agent teams
sidebar_label: Overview
---

# Agent teams

Use the [agent-team handbook](/docs/guide/agent-teams) to discover or create a team,
select its project manifest, assign role ownership and run the revisioned task
lifecycle. It is the canonical guide served directly from `docs/guide`; this
entry links to it rather than copying its procedures.

Local coordination requires Node.js 22+ and the complete creator payload. Native
Codex, Claude Code, OpenCode, Kimi and DeepSeek execution remains subject to each
installed harness's discovery and permissions. Export and project installation
prepare definitions; they do not launch workers or activate UAR/BossFang teams.
Model and memory services are separate optional capabilities.

## Start at your level

| You need to… | Read |
| --- | --- |
| Turn an outcome into a small team | [User and lead responsibilities](/docs/guide/agent-teams#find-your-responsibility) |
| Reuse an existing project team | [Create, select or adopt](/docs/guide/agent-teams#install-or-adopt-a-project-team) |
| Give a role work it can safely execute | [Ownership](/docs/guide/agent-teams#choose-roles-and-keep-one-writer-per-path) and [dispatch example](/docs/guide/agent-teams#example-a-bounded-dispatch) |
| Track progress or recover interrupted work | [Revisioned lifecycle](/docs/guide/agent-teams#run-the-revisioned-task-lifecycle) and [canonical recovery](/docs/guide/agent-teams#preserve-canonical-identity-and-recover-deliberately) |
| Move work to another role, harness or project | [Accepted handoffs](/docs/guide/agent-teams#transfer-context-and-wait-for-acceptance) and [project requests](/docs/guide/agent-teams#route-requests-to-another-project) |
| Set a model, effort or fallback | [Model selection](/docs/guide/agent-teams#choose-a-model-for-the-task), [native limits](/docs/guide/agent-teams#reasoning-and-native-harness-limits) and [fallback decisions](/docs/guide/agent-teams#budget-fallback-and-independent-review) |
| Share lessons with the right audience | [Private, team, project and broader scopes](/docs/guide/agent-teams#keep-lessons-scoped-to-their-audience) |
| Decide whether services or Companion are needed | [Services for team work](/docs/guide/service-operations#choose-services-for-team-work) |

## Procedures and request examples

- [Create and select a project team](/docs/guide/agent-teams#install-or-adopt-a-project-team)
- [Assign ownership and inspect the records](/docs/guide/agent-teams#choose-roles-and-keep-one-writer-per-path)
- [Run the task lifecycle](/docs/guide/agent-teams#run-the-revisioned-task-lifecycle)
- [Understand UAR direct coordinator acceptance](/docs/guide/agent-teams#uar-direct-coordinator-acceptance)
- [Preserve canonical identity and recover](/docs/guide/agent-teams#preserve-canonical-identity-and-recover-deliberately)
- [Use the full-pack JSON request reference](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#task-lifecycle-request-sequence)
- [Use the mini-pack request reference](https://github.com/Prometheus-AGS/prometheus-skills-mini/blob/main/docs/agent-teams.md#task-lifecycle-request-sequence)

The full pack includes broader learning integrations; mini retains its Node-only
portable runtime with surreal-memory and liter-llm as its two optional services.
Local team state works without them. Source examples and schema receipts do not
establish live native acceptance, successful inference or deployed execution.

## Communicate and transfer context

Follow the canonical guide for [same-team updates](/docs/guide/agent-teams#communicate-within-the-team),
[context transfer and acceptance](/docs/guide/agent-teams#transfer-context-and-wait-for-acceptance),
[cross-project intake](/docs/guide/agent-teams#route-requests-to-another-project) and
[scoped learning](/docs/guide/agent-teams#keep-lessons-scoped-to-their-audience).
Use the [two-role executable request sequence](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#two-role-handoff-request-sequence)
for the precise creation, revisions, accepted review transfer and completion fields.

Full has explicit card publication/discovery and issue request/intake commands;
mini currently requires authorized manual cross-project delivery. An imported
request is pending triage, not accepted work. Issue writes need explicit
authorization and `gh`; local handoff needs no service or remote chat API.
Full's learning writer and optional Cortex mirror are separate from mini's
Claude-only file-tier delivery. Neither a scope label nor a mirror admission
receipt proves private storage, delivery or recall. Inspect actual packaged and
installed capabilities before using these source examples.

## Select and record an actual model route

Use the [canonical model guide](/docs/guide/agent-teams#choose-a-model-for-the-task)
for declared-versus-observed evidence, policy precedence, native reasoning limits,
budgets, explicit fallback and reviewer independence. The
[full model request sequence](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/agent-teams.md#model-discovery-selection-and-persistence)
shows discovery, effective selection and persisted native intent. Record each
canonical assignment through [task model assignments](/docs/kbd/task-model-assignments).

KBD chooses task fit first; the helper filters constraints then sorts eligible
candidates by known prices. Neither selection nor export switches a running
worker. liter-llm inference needs a separate documented worker for tool execution.
Unknown metadata can leave the route unresolved; no automatic fallback or
universal per-role reasoning control is promised. A same-family critic does not
satisfy different-family KBD review, even if its alias or effort differs.
