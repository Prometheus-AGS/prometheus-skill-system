# Parallel Agent Routing

How tasks in this pack are partitioned across `agent_role` values. Five roles, designed to maximize parallelism and minimize cross-stream contention.

## Why five roles, not one

A single agent role would serialize work that can run in parallel. The roles below are partitioned along **file-modification boundaries** so two sessions in different roles rarely touch the same files. Cross-role contention is the leading cause of merge conflicts in agent-driven workflows; this routing minimizes it.

## Roles

### `skill-pack-maintainer`

**What they own.** The structure of `prometheus-skill-pack`. The two CLAUDE.md files. The `.claude-plugin/plugin.json` and `hooks/hooks.json` (modulo the hooks-engineer's per-script ownership). Skill discovery, naming, and description matrices. Slash commands that ship with the pack. The skill productization workflow (lifting project-specific patterns into pack skills).

**Don't own.** The contents of individual hook scripts, the `prometheus-knowledge` Rust workspace, the SSR test pipeline.

**Tasks routed here.** SP-001, SP-015, SP-016, SP-017, BDD-004, XC-001, XC-004, XC-005.

### `hooks-engineer`

**What they own.** The shell scripts under `shared/scripts/`. The hooks.json wiring (in coordination with skill-pack-maintainer). The `pk-focus-on-prompt.sh`, `forge-reflect-on-stop.sh`, and the Stop chain. Cedar policies that enforce skill mutations. Observability infrastructure (`~/.prometheus/hooks.log`). Anything that runs inside Claude Code's hook lifecycle.

**Don't own.** Rust source code, TypeScript scripts in user projects, BDD test internals.

**Tasks routed here.** SP-002, SP-003, SP-006, SP-007, SP-009, SP-011, SP-012, SP-013, SP-014, SP-018, SP-021, XC-002.

### `rust-codegraph`

**What they own.** Anything Rust. The `prometheus-knowledge` crates (pk-core, pk-store, pk-mcp, pk-librarian, pk-cli). New crates such as `pk-codegraph`. Surreal schemas and migrations. The librarian event model. The memory dual-store separation.

**Don't own.** Hook shell scripts, TypeScript test infrastructure.

**Tasks routed here.** SP-005, SP-008, SP-010, SP-019, SP-020, BDD-008, BDD-009.

### `bdd-engineer`

**What they own.** Everything under `/Users/gqadonis/Projects/sansaba/ssr-frontend/tests/` and `/Users/gqadonis/Projects/sansaba/ssr-frontend/scripts/`. The cucumber.js profiles. The video-proof runner, IPFS upload, manifest, validation gates, docs generator. Step definitions and feature files. The feedback engine integration with tests.

**Don't own.** Component code in `src/`, the document-generation-agent project, Rust crates.

**Tasks routed here.** BDD-001, BDD-002, BDD-003, BDD-005, BDD-007, BDD-010, BDD-011, BDD-012, BDD-014, BDD-015.

### `docs-writer`

**What they own.** Markdown-only changes. CLAUDE.md rules added/modified (without touching code). User-story → feature-file contracts (the docs side, not the parser). The XC-003 scratchpad pattern documentation.

**Don't own.** Code that backs the docs (those are routed to whichever role owns the underlying code).

**Tasks routed here.** BDD-006, BDD-013, XC-003.

## Concurrency matrix

|                              | skill-pack-maintainer | hooks-engineer | rust-codegraph | bdd-engineer | docs-writer |
|------------------------------|:--:|:--:|:--:|:--:|:--:|
| **skill-pack-maintainer**    | ✗  | ⚠  | ✓  | ✓  | ⚠  |
| **hooks-engineer**           | ⚠  | ✗  | ⚠  | ✓  | ✓  |
| **rust-codegraph**           | ✓  | ⚠  | ✗  | ✓  | ✓  |
| **bdd-engineer**             | ✓  | ✓  | ✓  | ⚠  | ⚠  |
| **docs-writer**              | ⚠  | ✓  | ✓  | ⚠  | ✗  |

Legend: ✓ safe to run concurrently · ⚠ check that specific tasks don't overlap files · ✗ same role; serialize unless tasks are clearly disjoint.

## Recommended starting allocation

If you have **one Claude Code session** to spend right now:

- Pick `hooks-engineer` and start with SP-013 (sycophancy in reflector). Highest-leverage P0 in the entire pack.

If you have **two parallel sessions**:

- Session A (hooks-engineer): SP-013.
- Session B (rust-codegraph): BDD-008 (pk-codegraph extraction). It's a P0, takes 1–2 weeks of work, and unblocks four downstream tasks. The earlier it starts, the better.

If you have **three parallel sessions**:

- A (hooks-engineer): SP-013.
- B (rust-codegraph): BDD-008.
- C (bdd-engineer): BDD-001 (manifest dual-key cleanup). Half a day of work, immediately reduces noise in every video run that follows.

If you have **four parallel sessions**:

- A, B, C as above.
- D (skill-pack-maintainer): SP-016 (skill description collision detection). Pulls in a real bug — at 64 skills, near-miss descriptions are statistically guaranteed.

If you have **five parallel sessions**:

- A–D as above.
- E (docs-writer): BDD-006 (immutable-tests CLAUDE.md rule). It's 0.5 days, and it locks down the boundary that BDD-005 and BDD-007 rely on. Critically, this is the doc that explains why "auto-update tests when code changes" is being declined as written. Land it early so other agents don't accidentally try to satisfy the original ask.

## Anti-patterns

- **Do not let one session do two roles' work.** A `hooks-engineer` session that wanders into Rust crate territory should stop, commit, and hand off to a `rust-codegraph` session. The role boundary is the file-modification boundary.
- **Do not let one session do two categories' work in the same role.** Even within `hooks-engineer`, SP-013 and SP-006 should be different sessions. Both touch `shared/scripts/` and would conflict.
- **Do not skip BDD-006 if you are touching tests.** It is the ground rule. If you do not internalize it, you will produce work that the next reviewer rejects.
