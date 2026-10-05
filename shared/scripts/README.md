# Shared scripts

This directory contains the full pack's shared lifecycle, learning and workflow
helpers. The files and `lib/` modules here are canonical source; packaged copies
are generated from them.

## Main groups

| Source | Purpose |
|---|---|
| `sessionstart-learning.sh`, `subagentstart-learning.sh`, `subagentstop-learning.sh` | Harness lifecycle learning entry points |
| `lib/learning_write.py`, `lib/learning_recall.py`, `lib/learning_route.py` | Scoped lesson writes, recall and routing |
| `lib/project_id.py`, `lib/agent_identity.py`, `lib/path-scope.sh` | Project, author and ownership context |
| `lib/canonical-url.sh`, `lib/slug.sh` | Shared research URL and slug normalization |
| `memory-outbox-flush.sh`, `mem0-compress.sh`, `pk-lint.sh` | Explicit memory and knowledge maintenance |
| `codex-memories-config.sh` | The pack's Codex native-memory configuration policy |
| `scheduled/` | Periodic maintenance templates; separate from interactive hooks |

Each entry point declares its own inputs, dependencies and exit behavior. Bash,
Python, jq and native tools are workflow-specific prerequisites; the presence of
a shared file does not make that workflow Node-only or portable to every host.
The earlier `validators/`, `generators/`, `formatters/`, `parsers/` and `common/`
directory examples were a proposed layout, not the contents of this directory.

## Packaging and state

Copy the whole required helper family into the plugin payload. Callers resolve
the shipped plugin root through their declared environment or bounded layout
resolver; they must not require the original developer checkout. A standalone
copy of one skill may also need shared helpers. Read that skill's prerequisites.

Learning state and queues belong to the configured project and learning roots.
Do not put credentials in lesson content or provenance. Missing optional services
must preserve an explicit durable outcome rather than imply publication.
See [memory tiers](../../docs/guide/memory-tiers.md) and
[team memory](../../skills/process/agent-team-creator/references/models-memory.md).

## Maintenance

Update callers and their packaging contract with helper changes. Complete the
whole production phase before generating copies or running local integration
gates. Acceptance exercises the installed entry point and its real collaborators;
a helper-only result does not establish delivery. Preserve other skills' use of
shared normalization, scope and receipt formats.
