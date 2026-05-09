---
id: SP-008
title: Karpathy KB per-project scoping (confidentiality)
status: ready
priority: P0
estimated_effort: 1-2d
agent_role: rust-codegraph
depends_on: []
unblocks: [SP-019, XC-005]
related: [SP-020]
created_from_conversation_turn: 3-4
---

# SP-008 — Karpathy KB per-project scoping

## Problem

The `prometheus-knowledge` Karpathy wiki defaults to `~/.prometheus/knowledge/` — a global directory shared by every project on the user's machine. A session in one project (say a hypothetical Brius healthcare project) writes its learnings into the same KB that a session in another project (say SSR) reads from. **Patient data, customer data, internal-only information, and project-confidential reasoning all co-mingle in one unscoped store.**

This is a confidentiality problem, not a feature request.

## Evidence

1. Read `pk-store/src/markdown.rs` (or equivalent) — note that the storage path is constructed from a single env var or default with no project key.
2. Read `pk-cli` — note that `pk ingest` writes to whatever location the store resolves to, with no project-aware partitioning.
3. Confirm: `PK_KB_DIR` exists as a configuration knob, but nothing automatically sets it per-project.

## Why it matters

This is P0 because:

1. **Active risk.** As soon as more than one project uses the Karpathy loop, cross-contamination begins. There may already be cross-contamination depending on which projects have been active.
2. **Compliance shape.** If healthcare or financial projects use this stack, the global KB violates per-project data segregation requirements (HIPAA, SOX, internal data classification).
3. **Foundational dependency.** SP-019 (LibrarianEvent persistence) and XC-005 (project-scoped overlay) both assume per-project scoping. They cannot land safely without this.

## Proposed fix

Three layered changes.

**1. Convention.** The KB directory becomes `<project_root>/.prometheus/knowledge/` by default, scoped per repository. The `~/.prometheus/knowledge/` path is preserved but reserved for *cross-project shared knowledge* (patterns, libraries, framework references) — explicitly opted into by `pk ingest --scope=shared`.

**2. Resolution.** The `pk` CLI walks up from `cwd` looking for the nearest `.git`, `package.json`, or `Cargo.toml` to find project root. Sets `PK_KB_DIR` to `<project_root>/.prometheus/knowledge/` if not already set.

**3. Migration.** A one-time `pk migrate-to-per-project` command:
- Inventories `~/.prometheus/knowledge/` entries.
- For each entry, attempts to associate it with a known project root by inspecting the entry's metadata (the librarian records source-project for ingested content).
- Moves entries to project-scoped directories.
- Leaves entries that genuinely apply across projects in `~/.prometheus/knowledge/` under `shared/`.

## Trade-offs and risks

- **Migration is lossy if entries lack source-project metadata.** Many existing entries may not record their origin. Mitigation: dry-run mode prints the categorization; user approves before move.
- **Cross-project queries become harder.** A query that wanted to find "patterns I've solved before across all projects" now requires explicit `--include-shared` plus optional fanout. Mitigation: explicit-by-default is correct for confidentiality; cross-project lookup is a deliberate operation.
- **Existing automation may break.** Any script that hardcodes `~/.prometheus/knowledge/` will silently miss the per-project content. Mitigation: grep the skill-pack and prometheus-knowledge for `~/.prometheus/knowledge` and `$HOME/.prometheus/knowledge` strings; update or document each one.

## Acceptance criteria

- [ ] `pk` CLI default KB resolution returns `<project_root>/.prometheus/knowledge/` when run inside a project.
- [ ] `pk` CLI default returns `~/.prometheus/knowledge/` only when run outside any project root, with an info message noting the context.
- [ ] `pk ingest --scope=shared` is the *only* path that writes to `~/.prometheus/knowledge/`.
- [ ] `pk migrate-to-per-project --dry-run` produces a categorization report.
- [ ] After migration, `pk ingest` from the SSR repo writes to `/Users/gqadonis/Projects/sansaba/ssr-frontend/.prometheus/knowledge/` (not `~/.prometheus/knowledge/`).
- [ ] `.prometheus/` is added to per-project `.gitignore` template (or per-project rules) so KB content does not accidentally commit. The exception: if the team explicitly wants to share KB across the team via git, they remove the gitignore entry deliberately.

## Implementation steps

1. Add project-root resolution to `pk-cli/src/lib.rs` (walk up looking for `.git`, `Cargo.toml`, `package.json`, in that priority).
2. Modify `pk-store` initialization to consume the resolved path.
3. Add `--scope=shared` flag to `pk ingest`.
4. Implement `pk migrate-to-per-project` with `--dry-run` default.
5. Update `prometheus-knowledge/README.md` documenting the new layout.
6. Audit skill-pack and prometheus-knowledge for hardcoded paths; update.
7. Add a default `.gitignore` line covering `.prometheus/` to project templates.

## Dependencies

None.

## Open questions

- Should the project-root marker include `.kbd-orchestrator/` since some projects have that but not the standard markers? Yes — add it to the marker list.
- Should `shared/` content be additionally gated (e.g. `pk ingest --scope=shared` requires confirmation)? Probably yes, given the easy-to-fat-finger nature of this. Default `pk ingest --scope=shared` to interactive confirmation; bypass with `--yes`.
- Is there a viable per-team KB tier (between project and shared)? Out of scope for this task; may emerge as a future need.
