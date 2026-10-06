# forge-rs

Native context enrichment and reflection tooling for the full skill pack. `forge`
reads task files, resolves skill manifests and templates, checks the project's
constitution and writes context for an implementing agent. The generated context
is not generated application code or proof that the task was completed.

## Source layout

| Crate | Responsibility |
|---|---|
| `forge-core` | Domain types for skills, tasks, constitutions and iteration records |
| `forge-skills` | Skill discovery, resolution and template rendering |
| `forge-enricher` | Task input and enriched-context output |
| `forge-reflect` | Iteration records and drift calculations |
| `forge-mcp` | Local JSON-RPC HTTP adapter |
| `forge-cli` | The `forge` production entry point |

## First workflow

From a project with an approved task and the selected skills available:

```bash
forge --skills-root /path/to/pack/skills init
forge --skills-root /path/to/pack/skills enrich /path/to/task
forge reflect <iteration-id>
forge drift --language rust
```

Read the enrichment and implement the complete production specification before
validation. Reflection records acceptance information; it does not authorize a
skill mutation or release. `pk`/the configured knowledge endpoint supplies
optional context. An unavailable collaborator must be distinguished from a
successful knowledge write.

Global options (`--project-root`, `--skills-root`, `--pk-mcp-url`) appear before
the subcommand. `PK_MCP_URL` can supply the knowledge endpoint. Explicit skill
selection avoids relying on the default ancestor/home discovery. `EDITOR` chooses
the editor for `forge constitution <language>`.

Other source commands are `status`, `validate <file> --language <language>`,
`skill list`, `skill add`, `skill sync` and `package-librefang`. Their presence in
the CLI is not a guarantee that every registry/provider operation is available.
The current CLI declares no `forge template` command; `templates/meta/` retains
source templates, not an implemented template-management command family.

## MCP and templates

`forge mcp` defaults to loopback port 8943. `/mcp` accepts JSON-RPC HTTP requests;
configure the consumer for the actual adapter transport and bearer credentials.
The `--no-auth` option is permitted only for a loopback bind. An endpoint string
alone does not configure authentication or certify a native harness connection.
See [service operations](../../docs/guide/26-service-operations.md).

Skill `skill.toml` manifests select templates beneath their skill directories.
Project `.forge/skills/` overrides, constitutions, enrichment and reflection state
belong to that project. Keep authored state and recovery records when updating
the binary or the pack. Optional Companion controls are separate from this
standalone enrichment path.

## Maintain the tool

Read [the canonical repository rules](../../CLAUDE.md); they take precedence over
older command lists in this tool's [local guide](CLAUDE.md). Complete the whole
production phase before builds, tests or review. Use the smallest local
integration gate that exercises the `forge` entry point with real task files,
templates and its required collaborators. Record source identity, commands,
results and unavailable providers. Unit-only results and hosted CI are not
acceptance evidence.

Before Cargo work, ensure no other Cargo/rustc build runs on the machine. Keep a
separate target directory per worktree. Adding a language requires the domain
language/detection declarations, constitution and actual skill manifests; adding
a template requires its manifest entry as well as its source file.
