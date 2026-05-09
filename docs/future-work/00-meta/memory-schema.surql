-- ── prometheus-skill-pack-future-work :: Surreal schema ───────────────
--
-- Mirror of STATUS.md as a Surreal entity graph.
-- Run with: surreal sql --conn ws://localhost:8000 --user root --pass root --ns prometheus --db future_work < memory-schema.surql
--
-- Each task in STATUS.md becomes a `task` record. Each dependency edge
-- becomes a `blocks` relation. Agent roles are tags.
-- ──────────────────────────────────────────────────────────────────────

DEFINE NAMESPACE prometheus;
USE NS prometheus;

DEFINE DATABASE future_work;
USE DB future_work;

-- A task in this pack. id matches the SP-NNN / BDD-NNN / XC-NNN ID.
DEFINE TABLE task SCHEMAFULL;
DEFINE FIELD title         ON task TYPE string;
DEFINE FIELD category      ON task TYPE string ASSERT $value IN ['skill-pack-fixes', 'bdd-testing', 'cross-cutting'];
DEFINE FIELD priority      ON task TYPE string ASSERT $value IN ['P0', 'P1', 'P2'];
DEFINE FIELD effort        ON task TYPE string;     -- e.g. '1d', '1-2w', 'recurring'
DEFINE FIELD agent_role    ON task TYPE string ASSERT $value IN [
    'skill-pack-maintainer', 'hooks-engineer', 'rust-codegraph',
    'bdd-engineer', 'docs-writer'
];
DEFINE FIELD status        ON task TYPE string ASSERT $value IN [
    'planned', 'ready', 'in-progress', 'done', 'blocked', 'abandoned'
] DEFAULT 'planned';
DEFINE FIELD assigned_to   ON task TYPE option<string>;
DEFINE FIELD started_at    ON task TYPE option<datetime>;
DEFINE FIELD completed_at  ON task TYPE option<datetime>;
DEFINE FIELD doc_path      ON task TYPE string;     -- relative to docs/future-work/
DEFINE FIELD notes         ON task TYPE option<string>;
DEFINE FIELD created_at    ON task TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at    ON task TYPE datetime DEFAULT time::now() VALUE time::now();

DEFINE INDEX task_status_idx   ON task FIELDS status;
DEFINE INDEX task_priority_idx ON task FIELDS priority;
DEFINE INDEX task_role_idx     ON task FIELDS agent_role;

-- Dependency relation. `blocks` means: source must be `done` before target can be `ready`.
DEFINE TABLE blocks SCHEMAFULL TYPE RELATION FROM task TO task;

-- Looser cross-reference for related work (not blocking).
DEFINE TABLE related_to SCHEMAFULL TYPE RELATION FROM task TO task;

-- Conversation learning — discrete pieces of insight from the source session,
-- linked back to the tasks that enacted them. Lets a future agent ask
-- "why does this task exist?" and get the conversation context.
DEFINE TABLE learning SCHEMAFULL;
DEFINE FIELD content       ON learning TYPE string;
DEFINE FIELD source_turn   ON learning TYPE int;     -- conversation turn number
DEFINE FIELD topic         ON learning TYPE string;
DEFINE FIELD created_at    ON learning TYPE datetime DEFAULT time::now();

DEFINE TABLE enacts SCHEMAFULL TYPE RELATION FROM task TO learning;

-- Useful queries (run after seeding):
--
--   -- All P0 tasks ready to start now:
--   SELECT * FROM task WHERE status = 'ready' AND priority = 'P0';
--
--   -- Tasks blocked by a specific task:
--   SELECT * FROM task WHERE id IN (SELECT out FROM blocks WHERE in = task:`SP-008`);
--
--   -- Critical path: longest blocks chain
--   SELECT * FROM task WHERE id NOT IN (SELECT in FROM blocks);   -- terminal tasks
--   SELECT * FROM task WHERE id NOT IN (SELECT out FROM blocks);  -- root tasks
--
--   -- Agent workload:
--   SELECT agent_role, count() AS n, count(IF status = 'done' THEN 1 END) AS done
--   FROM task GROUP BY agent_role;
--
--   -- Why does task SP-013 exist? (assuming learnings linked):
--   SELECT * FROM learning WHERE id IN (SELECT out FROM enacts WHERE in = task:`SP-013`);
