# UI/UX routing command reference

Use Node.js 22+ and the shipped helpers. Paths below are relative to the full pack checkout; installed users substitute the actual skill directory. See the [user guide](../guide/25-ui-ux-routing.md) for design authority, routing choices and evidence limits.

## Portable commands

```text
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs install --project "/path/to/project" --dry-run
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs install --project "/path/to/project"
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs install --project "/path/to/project" --check
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs route --input ui-request.json
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs phase-boundary --input ui-request.json
```

Both routing commands accept the same request:

```json
{"project":"/path/to/project","ui":true,"affected":["apps/web/src/Settings.tsx"],"operation":"refine","surface":"app","model":"gpt-6","focus":"layout"}
```

| Field | Contract |
| --- | --- |
| `project` | Required existing directory; relative paths use the invocation directory. |
| `ui` | Boolean, default true; false loads no UI skills. |
| `affected` | Project-contained paths; omitted/empty resolves the root. Future files use their ancestors. |
| `operation` | `new`, `redesign`, `refine` (default), `review`. |
| `surface` | Marketing/landing/pricing/campaign → Persuade; docs/reading/articles/changelogs → Read; showcase/portfolio → Experience; otherwise Operate. |
| `model` | Actual identifier; GPT-family selection applies only to new work. |
| `focus` | Layout, typography, colors, copy, accessibility or motion selects one craft skill; otherwise better-interface. |
| `overlay` | Explicit high-end-visual-design, minimalist-ui or industrial-brutalist-ui; ignored for refine/review. |
| `role` | Reviewer/verifier/auditor identifiers force review. |
| `stack` | Pro Max query hint; does not override manifest detection. |

Do not supply manifests or evidence fields. The [runtime API](../../skills/ui-ux/prometheus-ui-ux/references/runtime-api.md) defines all aliases and result fields. Phase-boundary returns an evidence checklist with `executed: false`, not an acceptance receipt.

| Operation | Exit behavior |
| --- | --- |
| UI install `--check` | 0 current; 1 drift; 2 usage/preflight failure. |
| Route / phase-boundary | 0 request processed; 2 invalid input, command or path. |
| Creator install-project `--check` | 0 current; 2 drift; 1 error. |

## Full bootstrap and existing injector

These broader full-pack commands require their existing shell dependencies. Preview first. The v4 bootstrap additionally requires Python 3; that prerequisite does not apply to the portable Node UI helper.

```text
bash skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh --path "/path/to/project" --dry-run
bash skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh --path "/path/to/project" --layout v4 --dry-run
```

Remove `--dry-run` only when applying the broader context installation within the authorized scope. Both layouts preflight the same canonical UI installer. Existing generated projects must retain routes in their source `rules/src/routing.md`. Do not use `--force` merely to add UI guidance.

For the existing injector, set `KBD_ORCHESTRATOR_ROOT` to the actual installed orchestrator directory (the default is `$HOME/.claude/skills/kbd-process-orchestrator`). From the checkout, an example for a Bash-capable host is:

```text
KBD_ORCHESTRATOR_ROOT="$PWD/skills/process/kbd-process-orchestrator" bash skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/kbd-inject-agent-rules.sh --pack uiux-routing --path "/path/to/project" --target both --dry-run
```

Remove `--dry-run` to apply. The `uiux-routing` pack delegates to the Node installer; it does not add a second protocol implementation. Direct Node installation is the portable route.

## Install project-team discovery

```text
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project" --dry-run
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project"
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project" --check
```

Use `--team <id>` for ambiguity or an intentional selection change. To install a new manifest, `--input install-request.json` accepts `{"project":"/path/to/project","team":<manifest>}`. Intentional replacement of a differing manifest also requires `updateTeam: true`. Export remains proposal-only. The [project installation contract](../../skills/process/agent-team-creator/references/project-installation.md) defines preservation, recovery, native targets and Zed precedence; the [team request reference](../agent-teams.md) covers initial creation and handoff.

## Focused Pro Max retrieval

```text
node skills/ui-ux/ui-ux-pro-max/scripts/search.mjs "operational dashboard accessible forms" --domain ux --json
node skills/ui-ux/ui-ux-pro-max/scripts/search.mjs "keyboard navigation React 19" --stack react --json
```

Use the actual framework version and a focused query. Full design-system generation and persistence are for establishing authorized direction; see the [Pro Max contract](../../skills/ui-ux/ui-ux-pro-max/SKILL.md). Existing recommendations remain advisory. A full-only preinstalled Impeccable engine must be explicitly selected with `IMPECCABLE_BIN`; no download or mini native-engine parity is implied.
