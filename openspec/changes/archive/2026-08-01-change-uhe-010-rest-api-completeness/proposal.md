# REST covers installation and query, proven per verb

**Change:** `change-uhe-010-rest-api-completeness`
**Phase:** uar-host-execution
**Goal:** R4

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Task 1 — every skill endpoint, enumerated

Mounted at **two** prefixes (`src/server.rs:875` and `:1000`), both serving the
same `build_router()`:

- `/api/uar/skills`
- `/api/skills`

| Method | Path | Purpose | R4 verb |
|---|---|---|---|
| `GET` | `/` | list all skills | **query** |
| `POST` | `/` | create a skill | **install** |
| `GET` | `/{id}` | get one skill | **query** |
| `PUT` | `/{id}` | update a skill | — |
| `DELETE` | `/{id}` | delete a skill | — (refused for builtins, R2) |
| `POST` | `/{id}/toggle` | enable/disable | **toggle** |
| `GET` | `/match` | search by query | **query** |
| `GET` | `/provenance` | active pack version/commit/count | — (R5) |
| `POST` | `/refresh` | re-read from providers | — |
| `POST` | `/import` | install from disk | **install** |
| `GET` | `/config` | matching config | — |
| `PUT` | `/config` | set matching config | — |

Agent-scoped bindings (`build_agent_skills_router`, mounted under agents):

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/{agent_id}/skills` | skills bound to an agent |
| `PUT` | `/{agent_id}/skills` | replace the binding set |
| `POST` | `/{agent_id}/skills/{skill_id}` | bind one |
| `DELETE` | `/{agent_id}/skills/{skill_id}` | unbind one |

**Every R4 verb has an endpoint.** So task 5 has nothing to add — but per the
acceptance criteria that is *not* the finding that matters. "Endpoints exist" is
not acceptance; tasks 2-4 supply the passing per-verb tests.

## The finding: 26 passing tests, 0 persistence assertions

Tasks 2-4 asked for "a passing request/response test" per verb. Those already
existed — `tests/skills_api_integration_test.rs` has **26 passing tests**
covering install, query (list/get/match), and toggle.

Stopping there would have satisfied the letter of the tasks and missed the point
the plan itself warned about: *"'Endpoints exist' is not acceptance."*

**Every one of those 26 builds the service without persistence:**

```rust
let service = Arc::new(SkillService::new(None, None));
//                                       ^^^^ no persistence layer
```

They assert status codes and response bodies against an in-memory registry.
`grep -c 'PersistenceLayer'` over that file returns **0**.

### Why that matters concretely

This is the exact seam the two `change-uhe-008` defects lived in. `POST /skills`
returns `201 Created` with a completely correct body while the row is silently
dropped, because `SkillRegistry::register` **logs** persist failures without
propagating them. An empty pgvector value and a notify trigger that aborted every
insert both hid there, and 26 green tests said nothing about either.

**"The endpoint returned 201" is not "the skill was installed."**

### What was added

`tests/rest_api_persistence.rs` — five tests that attach a **real** persistence
layer and assert against the database, not the response body:

| Verb | Assertion |
|---|---|
| install | `POST` writes a row; DB count goes 0 → 1 |
| install | the created skill is in persistence, not only retrievable over HTTP |
| query | `GET /skills` count **equals** the DB row count |
| query | `match` finds an installed skill with **no embedder configured** |
| toggle | the new `enabled` state is durable, and the skill still exists |

Deliberately built with `SkillService::new(Some(db), None)` — a database and
**no** `VectorMatcher`, which is the embedded configuration under which the
uhe-008 bug occurred.

### Task 5: nothing to add

Every R4 verb has an endpoint (see the enumeration above). No gap to record.

### Three defects caught before compiling

Written by reading the 26 working tests rather than inferring the API:

- `TestServer::new` returns `TestServer`, not `Result` — `.expect()` would not compile
- the match param is **`q`**, not `query`
- the payload needs `triggers`/`prompt_overlay`/`preferred_tools`, not `instructions`

Each would have cost a full build cycle. The existing tests were the spec.
