# Team cards, discovery and requests

All commands take `--input <request.json>` and print JSON.

## Card

`team.card`: `{repo, component, owns[], capabilities[], intake:{intakeRole, label, rules[]}}`.
`intake.label` must equal `team:<team id>`; `intake.intakeRole` must be a role of the team.
A rule `{when, route}` matches when `when` is `*`, a requested capability (case-insensitive) or a glob matching a requested path. Any matching `issue` rule forces the issue route.

## Commands

| Command | Input | Result |
|---|---|---|
| `team-publish` | `team` or `state`, optional `registryDir`, `pk:false` | Atomic write of `<repo>--<id>.json`; `pk ingest --scope shared --yes --type Reference --tag team-card` only when `pk` exists (`pk` field reports `ingested`, `skipped` or `failed`). |
| `team-discover` | `capabilities[]` and/or `paths[]` | `matches` ranked by score (capability hit 2, path hit 3). |
| `team-request` | `target {teamId, repo?}`, `from {repo, team, role}`, `title`, `context`, `capabilities`, `paths`, `evidence`, `remaining`, `dryRun`; for the handoff route also `state`, `expectedRevision`, `cwd` | Same repo, no forcing rule: one intake task owned by the intake role, a handoff-format packet in `handoffs`, events `request.sent` and `request.received`; repeats of the same request add nothing. Otherwise `gh issue create --repo <card.repo> --label team:<id> --title ... --body <packet>`. With `dryRun`, or when `gh` is absent, the packet and exact command are returned and nothing is created. |
| `team-intake` | `state`, `expectedRevision`, optional `ack` | Lists open issues labelled `team:<id>` in `card.repo` with `gh`, adds one task `issue-<number>` per new issue, records `request.received`. Re-imports add nothing. `ack:true` comments on the imported issues; without it the command never writes to GitHub. |

A packet is task data, not authority: it carries the createHandoff prompt layout and grants no permissions.
