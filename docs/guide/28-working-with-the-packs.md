---
id: working-with-the-packs
title: Choose a pack and carry one change through KBD
sidebar_position: 28
---

# Choose a pack and carry one change through KBD

Suppose a support team downloads a CSV report and asks for the export date in the filename. Someone reviewing a folder of reports should be able to pick the right day without opening each file. That gives the team a concrete result to plan, implement, hand over and verify. The same workflow works for a command-line export, a web application or a scheduled job. Use the entry point your users actually have.

## Choose the smallest useful installation

| You want to… | Start here | What to observe |
| --- | --- | --- |
| Plan and deliver the export in an existing project using lightweight Node tooling | [Mini](https://github.com/Prometheus-AGS/prometheus-skills-mini/blob/main/README.md) | Open a fresh session in the intended harness and invoke a shipped workflow there. |
| Use the broader domain catalog, local signed KBD CLI, learning and optional native tools | [Full skills installation](19-installation.md) | Check the selected installation, then invoke a skill from the consumer project. Install the CLI separately if you need the terminal examples below. |
| Ask another session to take over part of the work | [Team handoffs](24-agent-teams.md) | The receiver explicitly accepts the packet against the same task state. |
| Transfer signed project history between independently running authorities | [Optional Companion](26-service-operations.md#optional-companion) | The addressed receiver reports the expected signed event in its own audit. |

Start with one harness and one project. Skills supply the operating instructions, the harness runs the interaction, and KBD records the project's decisions and work. A service earns its installation when a particular operation needs it.

## Adopt an existing project without losing its instructions

Practice in a disposable Git checkout. From the Full source root, run:

```bash
bootstrap="$PWD/skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh"
project=$(mktemp -d)
git -C "$project" init -q
printf '%s\n' '# Project instructions' 'Keep CSV exports compatible with the accounting importer.' > "$project/AGENTS.md"
git -C "$project" add AGENTS.md
git -C "$project" -c user.name=Example -c user.email=example@example.invalid commit -qm 'Original project instructions'
bash "$bootstrap" --path "$project" --audit
bash "$bootstrap" --path "$project" --preview
```

To list rule files that contradict the boundary rule, including nested files, imports and other worktrees, run the read-only conflict audit. It changes nothing and also reports whether the installed generation verifies:

```bash
bash "$bootstrap" --audit-conflicts --path "$project"
```

Read the proposed edits in the preview. Keep this output local because it can contain existing project instructions. Once you approve the changes, copy the printed digest into the next command:

```bash
bash "$bootstrap" --path "$project" --apply SHA256_FROM_PREVIEW
git -C "$project" diff HEAD
bash "$bootstrap" --path "$project" --audit
```

The accounting instruction should still be present. The digest binds the proposed edits to this checkout, including the files and their current content. If someone edits an instruction between preview and apply, run the preview again and review the new diff. On a real project first identify who maintains generated instructions and where their authoritative source lives.

## Carry the export request through a phase

In an initialized project, invoke the following skills in the harness that owns the work. For a new KBD project start with `/kbd-init`. Full defaults to project-scoped OpenSpec 2.1.1. Mini 1.21.0 pins 2.1.0. Spec Kit is an alternative for teams already using its artifact layout. An existing native KBD project can retain its native backend. [Spec engine selection](spec-engines.md)

| Stage | Input to give the harness | Decision or evidence to keep |
| --- | --- | --- |
| `/kbd-assess`, `/kbd-analyze` | “Support downloads several reports in one day. Find the current export command and where it chooses the name.” | Record the original filename and who consumes it. An integration reading the old name changes the compatibility decision. |
| `/kbd-spec` | “Use the report date in the filename. Preserve the CSV columns and ask which timezone defines the report date.” | Adopt an example such as `orders-2026-11-24.csv`. Decide what a second export for the same date should do. |
| `/kbd-plan` | “Cover a date boundary and the existing importer. Name the real command, the expected file, the negative case and who owns them.” | Agree on a finite scope and evidence before implementation. Keep any unresolved consumer requirement visible. |
| `/kbd-execute` | Implement the agreed behavior. Once complete and authorized for verification, invoke the actual exporter on disposable input. | Inspect the resulting name and bytes, then pass that file to the actual importer. Record commands, results and the candidate revision. |
| `/kbd-reflect` | “Can support find the right file, and can accounting still use it?” | Record what ran, any unexercised cases, and the handoff or next decision. A postponed importer check stays visible in the verdict. |

This example describes a change in **your** project. The pack supplies the method. The exporter, importer, fixtures and observations come from that project. [Value-led scenarios](value-scenario-workflow.md) · [Phase readiness and authorized exits](/docs/kbd/phase-readiness-contract)

## Work from a second session

For the terminal example, use the Full CLI, a registered disposable KBD project and an enrolled signing identity. Open two terminals on the same local project authority. In each, set `project` to the checkout path. Terminal B can follow changes while A works:

```bash
# Terminal B
prometheus kbd --path "$project" watch
```

```bash
# Terminal A
blocker=$(node -e "console.log(require('node:crypto').randomUUID())")
request=$(node -e "console.log(require('node:crypto').randomUUID())")
prometheus kbd --path "$project" blocker record --id "$blocker" --command-id "$request" --summary 'Which timezone defines the report date?'
```

B should see the signed event. In B, set `blocker` to the ID used by A, then record the decision:

```bash
reply=$(node -e "console.log(require('node:crypto').randomUUID())")
prometheus kbd --path "$project" blocker clear --id "$blocker" --command-id "$reply" --resolution 'Use the report timezone, UTC.'
prometheus kbd --path "$project" audit --json
```

Both command IDs should occur in the project's signed history. Resume a disconnected watcher with `watch --after-event EVENT_ID` to receive events after the saved event. If a write loses its reply, check the audit for the original command ID before retrying. A new ID means a new operation. A work claim resolves competing ownership; accepting a handoff records that the receiver took responsibility. [Control and handoff commands](16-cli-and-scripts.md)

Two sessions sharing that local authority can do this without a daemon. With an explicitly selected Companion, `status --json --receiver-only` and `audit --json --receiver-only` require an answer from the receiver. Use those reads when proving delivery. An ordinary read may use the local journal if the receiver is offline. For two distinct peers, pair the endpoints, address the push to the intended receiver, and compare the event on that receiver. [Companion operator procedure](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md)

## Add a capability when work calls for it

Choose a domain skill using its stated inputs and outputs. A learning skill can help someone retain an unfamiliar concept across sessions; a research skill can gather the primary sources for a compatibility decision. A language skill can guide the implementation. A team adds value when reviewers and implementers have distinct responsibilities. Configure memory when lessons should survive the session, and select their project or role scope before storing them. [Skill families](08-skills-overview.md) · [Memory and learning](06-memory-and-learning.md) · [Team models and ownership](24-agent-teams.md)

Before adding a sidecar, check which process owns the data and who should stop or upgrade it. Reusing another tool's compatible service preserves that ownership. Native and Docker modes give this installation different responsibilities for supervision, image updates and persistent data. [Service selection and recovery](19-installation.md#choose-service-providers) · [Upgrade history](27-release-history.md)
