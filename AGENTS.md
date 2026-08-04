# AGENTS.md

Behavioral rules for all AI agents working in this repository. These rules apply to Claude Code, subagents, orchestrators, and any AI assistant operating on this codebase.

---

## Progress Signaling (MANDATORY)

Every agent must emit a progress signal at the start and completion of each phase and each task. These signals make long conversations scannable and let any participant orient immediately.

### Exact format

```
Starting phase 2 out of 6:  SP-014 fallback SubagentStop matcher verification
Starting task 1 out of 4:   Locate all fallback hook scripts
Completed task 1 out of 4:  Locate all fallback hook scripts
Completed phase 2 out of 6: SP-014 fallback SubagentStop matcher verification
```

### Rules

- Signal **before any work begins** — not mid-task.
- Signal **immediately after completion** — before starting the next item.
- Read `progress.json` or the active plan to get accurate totals. Never guess.
- Use the **canonical name** from `plan.md` or `progress.json`.
- Signals are plain text in the response — no tool call required.
- Applies in every session regardless of context length or role.

---

## Local-Only Validation (MANDATORY)

All builds, tests, linting, formatting, type checks, documentation checks, API
contract checks, doctor runs, health checks, diagnosis, and release certification
must run on the local development machine.

- **Never use GitHub Actions or any hosted CI/CD runner for testing or validation.**
- Do not start, rerun, watch, poll, debug, or cite a GitHub Actions test workflow.
- Do not use GitHub Actions as a development loop, failure reproducer, parity gate,
  or source of release evidence.
- Push only after the applicable local gates pass, and record the exact local
  commands and results.
- GitHub may be used for source hosting, review, and an explicitly authorized
  deployment such as GitHub Pages. A deployment workflow must not substitute for
  local testing.
- The only hosted automation exceptions are deterministic `docs:sync` updates
  after a commit reaches `main` and GitHub Pages packaging/deployment. Neither
  workflow may run tests, lint, doctors, contract checks, or certification.
- If a legacy test workflow starts automatically, cancel it when authorized and
  continue locally. Its result is not validation evidence.

## Agent Tool Freedom and Certification Integrity

- Bash, Python, Edit, Write, and other mutation tools remain unrestricted.
- Do not add PreToolUse mutation guards, Bash matchers, shell parsers, command
  allow-lists, or Python restrictions.
- Protected BDD test integrity is evaluated only at final local certification
  from committed Git state with `scripts/verify-protected-tests.mjs`.
- Intentional protected-test changes require an SSH-signed canonical approval
  manifest under the `prometheus-test-change` namespace.

---

## Memory — Mandatory Protocol

Memory lookup and write is **not optional**. Every agent session must follow this protocol.

### Tool Priority Chain

Use the first available tool:

1. **surreal-memory MCP** — preferred. Detected by presence of any of: `create_entity`, `add_memory`, `search_memories`, `semantic_search`, `hybrid_search_memories`
2. **Cortex MCP** — fallback. Detected by presence of `cortex_recall`, `cortex_remember`
3. **File-based memory** — always available at:
   `~/.claude/projects/-Users-gqadonis-Projects-prometheus-prometheus-skill-pack/memory/`

If surreal-memory is in `.mcp.json` but absent from the session tool list, use the next available option. Never skip memory.

### Session Start — Always Run These Lookups

```
# surreal-memory
semantic_search("prometheus-skill-pack recent work")
search_memories(query="<current task topic>", user_id="prometheus-skill-pack")
search_memories(query="<current task topic>", user_id="global")  # global lessons

# Cortex
cortex_recall("recent work prometheus-skill-pack")
cortex_recall("<current task topic>")

# File
Read ~/.claude/projects/-Users-gqadonis-Projects-prometheus-prometheus-skill-pack/memory/MEMORY.md
# Then read any files listed there that are relevant to the current task
```

### After Every Feature or Bug Fix — Always Write Memories

Immediately after completing any non-trivial work, save memories using this distinction:

#### Global memories
For patterns, gotchas, and lessons that apply to **any** project using the same stack (Rust, clap, agentskills, subagent orchestration, etc.).

```
# surreal-memory
add_memory(content="...", user_id="global", metadata={"type":"global","topic":"rust|clap|subagent|..."})

# Cortex
cortex_remember(content="...", global=true)

# File memory
# Create ~/.claude/projects/<slug>/memory/feedback_<topic>.md with type: feedback
# Prefix name with "GLOBAL:" to distinguish from project-specific
```

#### Project-specific memories
For architecture decisions, file paths, crate names, phase numbers, or bugs that are specific to this repository.

```
# surreal-memory
add_memory(content="...", user_id="prometheus-skill-pack", metadata={"type":"project","feature":"<name>"})

# Cortex
cortex_remember(content="...", projectId="prometheus-skill-pack")

# File memory
# Create ~/.claude/projects/<slug>/memory/project_<feature>.md with type: project
```

#### What to record

- **Architecture decisions** — what was chosen and why (include the alternatives considered)
- **Gotchas and bugs** — with exact file paths for project-specific issues
- **Patterns validated here** — Rust idioms, skill authoring conventions, etc.
- **Subagent lessons** — what context was missing that caused a subagent to go wrong

---

## Subagent Dispatch Rules

When dispatching a subagent to implement a task:

1. **Always state the exact absolute path** of the working directory
2. **State what must NOT be touched** — list other crates or dirs that exist nearby
3. **Provide key file content inline** when it's short — don't make the subagent guess
4. **Check memory first** before writing the prompt — include any relevant lessons

Example:
```
Working directory: /Users/.../tools/my-new-crate
This crate does NOT exist yet — you are creating it from scratch.
Do NOT modify tools/prometheus-cli or any other existing crate.
```

---

## Rust-Specific Rules (from this repo's experience)

### Binary+lib crate module scope

When a Rust crate has both `[[bin]]` and `[lib]` targets:
- Modules declared with `mod foo;` in `main.rs` are **binary-only**
- They can use `crate::AppContext` (defined in `main.rs`)
- They must import lib types as `use <libname>::module::Type`, not `use crate::module::Type`
- Never put phase runners (that reference `AppContext`) in `lib.rs`

### clap 4 global flag position

Global flags on the parent `Cli` struct must appear **before** the subcommand:
```bash
my-tool --format json subcommand    # CORRECT
my-tool subcommand --format json    # WRONG
```

### Cargo .gitignore before git add

Always create `tools/<crate>/.gitignore` containing `target/` before the first `git add`. Staging `target/` is a multi-MB mistake requiring a cleanup commit.

---

## Skill Authoring Rules

- `argument-hint` in SKILL.md frontmatter registers the `/skill-name` argument syntax for Claude Code
- Global flags in CLI examples must appear before the subcommand name
- Run `npm run validate:strict skills/<category>/<name>` before committing any new skill
- All scripts in `skills/*/scripts/` must be executable (`chmod +x`)

---

## surreal-memory Tool Reference

When surreal-memory tools are available:

```
# Knowledge graph
create_entity(name, entityType, observations[])
add_observations(entityName, observations[])
create_relation(from, to, relationType)
search_entities(query)
semantic_search(query)
read_graph()

# Scoped memory (mem0-compatible)
add_memory(content, user_id, metadata)
search_memories(query, user_id)
hybrid_search_memories(query, user_id)
get_all_memories(user_id)
compress_memories(user_id)

# Graph traversal
find_path(from, to)
expand_neighbors(entityName)
get_related(entityName)
```

Use `user_id="global"` for cross-project patterns. Use `user_id="prometheus-skill-pack"` for project-specific memories.
