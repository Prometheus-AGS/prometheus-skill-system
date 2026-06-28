# Goal Directory Layout

All goal state lives under `.kbd-orchestrator/goals/<slug>/`. The slug is
derived from the goal description + creation date (e.g., `build-standup-gen-20260627`).

```
.kbd-orchestrator/
├── goals/
│   └── <slug>/
│       ├── goal.json      # Goal definition and status (see goal.schema.json)
│       ├── IDEAS.md       # Ideation phase output — scored candidates
│       ├── SPEC.md        # Specification phase output — user stories, API contract
│       ├── TASKS.md       # Creation phase task checklist
│       └── STATE.md       # Live execution state
└── loops/
    └── <slug>/
        └── loop.json      # Outer loop definition (pmpo-outer-loop schema + phases[])
```

A parallel `loop.json` is written to `.kbd-orchestrator/loops/<slug>/` so that
`/loop-tick` can drive goal phase advancement using the existing outer-loop
infrastructure.

## File Formats

### `goal.json`

See [`schemas/goal.schema.json`](schemas/goal.schema.json) for the full schema.
Key fields: `slug`, `phases[]`, `active_phase`, `status`, `tool`.

### `IDEAS.md` (Ideation phase output)

```markdown
# Ideas — <goal description>

## Scored Candidates

| Candidate | Feasibility | Pain | Stack Fit | Buildability | Aggregate | Verdict |
|-----------|------------|------|-----------|--------------|-----------|---------|
| Weekly standup generator | 9 | 8 | 9 | 10 | 9.0 | PASS |
| PR summary slackbot | 7 | 7 | 8 | 8 | 7.5 | PASS |
| Commit message linter | 8 | 6 | 9 | 10 | 8.25 | PASS |

## Survivors (≥7.0 aggregate)

1. **Weekly standup generator** (9.0) — Reads git log, groups by day, outputs Slack-ready bullet list
2. ...

## Selected (human gate)

> Human selects one candidate here before Specification phase begins.

**Selected:** Weekly standup generator
```

### `SPEC.md` (Specification phase output)

```markdown
# Specification — <selected idea>

## User Stories

| # | As a... | I want to... | So that... |
|---|---------|-------------|------------|

## CLI Contract

\`\`\`
standup [--since <date>] [--repo <path>] [--format slack|markdown]
\`\`\`

## Acceptance Criteria

| ID | Criterion | Checkable by |
|----|-----------|-------------|
| AC-01 | Groups commits by day, max 5 bullets per day | script: test/verify-grouping.sh |

## Non-Goals

- Does not send to Slack automatically
- Does not parse issue tracker links
```

### `TASKS.md` (Creation phase task checklist)

```markdown
# Tasks — <goal slug>

## Checklist

- [ ] task-001: Scaffold Go module with `go mod init` [AC: go.mod exists, go build exits 0]
- [/] task-002: Implement git log parser [AC: test/parser_test.go passes]
- [x] task-003: Implement day-grouping logic [AC: test/group_test.go passes]
- [~] task-004: Add Slack formatter — promoted to child: task-004-slack-formatter
```

Task statuses:
- `[ ]` pending
- `[/]` in progress
- `[x]` complete
- `[~]` promoted to child phase

### `STATE.md` (Live execution state)

```markdown
# STATE — <goal slug>

**Status:** running
**Active phase:** creation
**Tool:** claude-code

## Progress

- completed: 2
- total: 8
- active_task: task-004

## Tasks

| ID | Status | Fail Count |
|----|--------|-----------|
| task-001 | complete | 0 |
| task-002 | complete | 0 |
| task-003 | complete | 0 |
| task-004 | promoted | 3 |

## Escalations

_None_

## Promotions

| Task | Child Phase | Reason |
|------|-------------|--------|
| task-004 | task-004-slack-formatter | fail_count reached 3 |
```
