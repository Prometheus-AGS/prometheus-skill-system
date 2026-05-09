# Memory Bootstrap

How to hydrate the `surreal-memory` graph from `STATUS.md` once the MCP is online.

## Prerequisite

You have:

- The `surreal-memory-server` MCP loaded and connected (its 42 MCP tools available in your session).
- A running Surreal instance reachable at the configured endpoint.
- The schema defined in `00-meta/memory-schema.surql` applied.

If the schema is not applied, run:

```bash
surreal sql \
  --conn ws://<your-surreal-host>:8000 \
  --user root --pass root \
  --ns prometheus --db future_work \
  < /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/docs/future-work/00-meta/memory-schema.surql
```

## Bootstrap steps

The bootstrap is a one-time operation that reads `STATUS.md` and creates a `task` record per entry, plus `blocks` relations for every `depends_on` edge.

### Option 1: One-shot via Claude Code

If you are a Claude Code agent picking this up, the shortest path is:

1. Read `STATUS.md` and parse the YAML block under `## Tasks`.
2. For each task entry, call the `surreal-memory` `create_entity` tool with:
   - `entity_type: "task"`
   - `name: "<id>"` (e.g. `"SP-001"`)
   - `observations:` an array containing `title`, `category`, `priority`, `effort`, `agent_role`, `status`, `notes` (if any)
3. For each task with non-empty `depends_on`, call `create_relation` with:
   - `from_entity: <dep>` (the blocker)
   - `to_entity: <task id>` (the blocked task)
   - `relation_type: "blocks"`

The exact `surreal-memory` tool names may vary — verify against `tool_search` in your session. The semantic operation is what matters: create one node per task, one `blocks` edge per `depends_on` entry.

### Option 2: Direct SurrealQL

If `surreal-memory` is unavailable but Surreal itself is reachable, run the following template, populating from STATUS.md:

```sql
-- Per task:
CREATE task:`SP-001` SET
    title = 'Two CLAUDE.md files unification',
    category = 'skill-pack-fixes',
    priority = 'P1',
    effort = '1d',
    agent_role = 'skill-pack-maintainer',
    status = 'ready',
    doc_path = '01-skill-pack-fixes/SP-001-claude-md-unification.md';

-- Per dependency edge:
RELATE task:`SP-006`->blocks->task:`SP-012`;
```

A small Python script that parses STATUS.md and emits the SurrealQL is the right tool. Suggested location for that script if you write it: `prometheus-skill-pack/scripts/bootstrap-future-work-memory.py`.

## Keeping the two in sync

`STATUS.md` and the Surreal graph are equivalent representations. After bootstrap:

- **Authoritative for humans and PR review**: STATUS.md (it's in git).
- **Authoritative for queries and progress dashboards**: Surreal.
- **Sync direction**: STATUS.md → Surreal (one-way, on bootstrap or after major batch updates).

Do not let agents update Surreal without also updating STATUS.md, or the two will drift. The `prometheus doctor` command (XC-004) should validate that the two match and fail the loop if they diverge.

## Operations after bootstrap

### Find the next P0 task that's ready

```sql
SELECT * FROM task WHERE status = 'ready' AND priority = 'P0' ORDER BY id;
```

### Mark a task in-progress

```sql
UPDATE task:`SP-013` SET
    status = 'in-progress',
    assigned_to = 'claude-code-session-2026-05-10-abc123',
    started_at = time::now();
```

When done:

```sql
UPDATE task:`SP-013` SET
    status = 'done',
    completed_at = time::now(),
    notes = 'Wired existing sycophancy-correction skill into shared/scripts/forge-reflect-on-stop.sh; verified critic agent receives only the artifact, not generation history.';
```

After marking SP-013 done, also update STATUS.md to match.

### Promote `planned` tasks to `ready` after their dependencies finish

```sql
-- Find tasks that should be ready now
SELECT id FROM task
  WHERE status = 'planned'
  AND id IN (
      SELECT out FROM blocks
      WHERE in IN (SELECT id FROM task WHERE status = 'done')
  )
  AND id NOT IN (
      SELECT out FROM blocks
      WHERE in IN (SELECT id FROM task WHERE status != 'done')
  );

-- Promote them
UPDATE task SET status = 'ready' WHERE id IN <list-from-above>;
```

This should be a recurring job — either on every task completion (preferred) or periodically.

## When this becomes obsolete

After every task in this pack is `done`, the entire `future_work` Surreal database can be dropped or archived. The `STATUS.md` file should be marked complete and committed in that final state. Any learnings worth keeping should be promoted to `prometheus-knowledge` wiki entries via the librarian (which itself is task SP-019).
