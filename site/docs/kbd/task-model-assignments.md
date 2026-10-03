---
title: Task model assignments
sidebar_label: Task model assignments
description: Select a concrete model and a usable worker route for every KBD task.
---

# Task model assignments

KBD planning recommends a concrete model for **each task**, including integration and review work. It chooses demonstrated task suitability first, within explicit user selections, project policy and budget. Cost and latency break close ties. A change-level complexity label does not choose the model for all its tasks.

This behavior ships in mini **0.2.0** and full **1.11.0** (process plugin **1.7.0**, versioned full-pack kbd-plan **1.1.0**). It adds a planning contract and dispatch instructions; it does not introduce a worker runtime, service or task-state schema.

## Plan before dispatch

1. Draft tasks and resolve their backend identities.
2. Assess reasoning difficulty, uncertainty, scope, tools, context size, modalities and review independence for each task.
3. Discover the installed harness version, configured providers, exposed models, supported reasoning controls and actual worker-launch mechanism.
4. Compare task fit using dated local results or authoritative capability documentation. Label unknown or stale evidence; a model name is not evidence of quality.
5. Write one **Task model assignments** table in the active phase's plan.md and reconcile it against every emitted task before handoff.

Planning records prerequisites. It does not launch workers, change providers, start services or copy credentials into artifacts.

## The assignment key and fields

Each row is keyed by **full phase path + change ID + exact backend task ID**. For example, task 1 in changes api and docs has two distinct keys even if both display 1.1. Use the ID returned by the backend and canonical identity resolver; a displayed checkbox ordinal is not a replacement ID.

| Required field | What to record |
| --- | --- |
| Task requirements | Reasoning, uncertainty, scope, tools, context, modalities and independence |
| Provider/model | Concrete configured identity, not only small/medium/frontier |
| Reasoning effort | A supported setting, or explicitly unsupported/unknown |
| Rationale and evidence | Why it fits this task, evidence date/source and limitations |
| Harness and route | Installed harness/version and native or liter-llm inference route |
| Worker launch and handoff | Documented invocation, scope, working directory, tools, skills and result destination |
| Native alternative | Best viable native option if different; not permission to substitute |
| Availability | Configured, verified or unresolved, with evidence |
| Prerequisites | Missing model access, protocol support, gateway or tool-enabled launcher |

OpenSpec tasks.md and native KBD artifacts reference the matching rows in ordinary prose. Keep checkbox syntax and task titles unchanged. Table rows do not become extra tasks, and assignment keys do not replace canonical runtime IDs.

## Harness discovery

| Harness | Check before selecting a route |
| --- | --- |
| Codex | Exposed model choices, supported fresh-agent overrides and fork inheritance |
| Claude Code | Installed subagent model controls and the protocol required by its worker |
| OpenCode | Concrete provider/model identifiers and configured per-agent or per-invocation controls |
| DeepSeek Harness | Installed provider/agent controls; global settings do not prove per-member overrides |
| Kimi Code | Secondary-model pools and invocation controls; avoid ignored model frontmatter |

There is no permanent best-model ranking. Re-evaluate when tasks, harnesses, model access or capability evidence change.

## Native execution and liter-llm

Use the selected model natively when the harness supports it. Otherwise record two separate parts: **liter-llm supplies inference**, and an existing documented **tool-enabled worker** executes the task in the workspace. Check protocol compatibility as well as model availability.

Discover the installed tool schema or CLI documentation. Do not invent a launch_agent interface or a liter-llm complete command. An inference response alone does not prove filesystem or tool execution.

If access or a worker is missing, leave the route unresolved and record its native alternative separately. A different selection requires an explicit recorded rationale. Other eligible tasks may continue; silently substituting a cheaper or available model is not allowed.

## Execute, revise and review

Before dispatch, kbd-execute and kbd-apply recheck the exact scoped assignment and copy its reference and actual route into execution.md and the worker handoff. Workers return artifacts, results and blockers to the KBD driver. The driver owns begin/end task boundaries and canonical completion updates.

Legacy plans remain usable: perform explicit task selection at execution time before running the task. Changed, split, removed or renumbered tasks require reconciliation so each current task has exactly one current assignment.

Adversarial review checks the plan before handoff, and sycophancy screening is used when available. Assigning an integration or review task does not authorize early verification: final integration and review follow the complete production delivery boundary. Readiness, implementation, evidence, certification and publication remain distinct.

## Verification limits

Local release exercises covered mixed-complexity planning, native Codex tool reads, an OpenCode worker through liter-llm, unresolved routes, legacy plans, changed tasks and repeated task numbers. Synthetic model evidence tested selection behavior, not provider rankings. The gateway's underlying provider identity and cross-model reviewer independence were not independently verified. Claude Code, DeepSeek Harness and Kimi Code execution were documentation/capability-checked only; Windows runtime was not exercised.

See [OpenSpec lifecycle updates](/docs/kbd/openspec-lifecycle) for project refresh and recovery.
