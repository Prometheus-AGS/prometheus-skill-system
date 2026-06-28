# pmpo-elicit escalation points

When each KBD/PMPO stage calls `/pmpo-elicit`, how it works on each platform,
and the shared file contract used across all callers.

---

## Section 1 — Stage trigger map

When a stage hits an unknown it cannot responsibly default, it raises an elicitation.

| Stage | Trigger condition | Criticality | State dir |
|-------|-----------------|-------------|-----------|
| `kbd-analyze` | Stack contest: top two candidates within 15% score gap | high | `.kbd-orchestrator/phases/<phase>/` |
| `kbd-goal` — Ideation→Spec gate | Human gate at phase boundary (skipped when `--auto-gates`) | high | `goals/<slug>/` |
| `kbd-goal` — Spec→Creation gate | Human gate at phase boundary (skipped when `--auto-gates`) | high | `goals/<slug>/` |
| `kbd-goal` — Creation inner-loop | Any escalation[] entry during task execution (ambiguity, security concern) | medium | `goals/<slug>/` |
| `pmpo-outer-loop` — `loop-tick` | Regression or `max_no_progress_ticks` reached | blocking | `.kbd-orchestrator/loops/<name>/` |
| `pmpo-outer-loop` — `loop-define` | Ambiguous field during interactive loop definition | medium | `.kbd-orchestrator/loops/<name>/` |

**Criticality guide:**

| Value | Meaning |
|-------|---------|
| `low` | Informational — loop can proceed with an implicit default if the operator doesn't respond |
| `medium` | Should be answered — loop pauses but can time-out to an implicit default |
| `high` | Must be answered — loop blocks until the operator responds |
| `blocking` | Hard gate — loop cannot proceed without an explicit decision |

---

## Section 2 — Platform routing table

How elicitation works on each AI platform:

| Platform | Question UI | Sync/Async | Resume signal |
|----------|------------|------------|---------------|
| **Claude Code** | `AskUserQuestion` built-in tool | Synchronous — no checkpoint file needed | Immediate; optionally write `result.json` for audit trail, then call `pmpo-elicit-resume.sh` |
| **Codex CLI** | `request-prompt.txt` displayed after agent stops | Async file-based | User writes `result.json`; re-invoke `codex` to continue |
| **OpenCode** | `update_goal` tool surfaces question in goal state | Async file-based | Next agent tick polls `result.json` |
| **Kimi Code** | `kbd-goal-check` detects `pending_elicitation` in `goal.json`; queues response step | Async file-based | `kbd-goal-check` reads `result.json`, unblocks `/goal next` queue |
| **Zed (standalone)** | `request-prompt.txt` in workspace; background task paused | Async file-based | User writes `result.json`; re-arm the background task |
| **Zed (ACP-connected)** | Delegates to the connected agent's native UI | Depends on connected agent | Depends on connected agent |

---

## Section 3 — Shared file contract

**Elicitation directory path:**
```
<state-dir>/elicitations/<caller>-<unix-timestamp>/
```

**Files in the elicitation directory:**

| File | Written by | Required | Purpose |
|------|-----------|----------|---------|
| `request.json` | `pmpo-elicit-checkpoint.sh` | YES | Machine-readable request (`elicitation.schema.json`, kind=request) |
| `checkpoint.json` | `pmpo-elicit-checkpoint.sh` | YES | Caller metadata: id, caller, timestamp, status |
| `request-prompt.txt` | `pmpo-elicit-checkpoint.sh` | YES | Human-readable question + instructions for writing result.json |
| `result.json` | Operator or Claude Code handler | YES (to resume) | Machine-readable result (`elicitation.schema.json`, kind=result) |

**Scripts:**

- `skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh` — writes all three checkpoint files; exits 2 (BLOCKED)
- `skills/process/pmpo-elicit/scripts/pmpo-elicit-resume.sh` — reads result.json; outputs `{"answer","provenance","id"}` to stdout; exits 0 on success, 1 if not ready

See `references/checkpoint-contract.md` for the full caller integration protocol with code examples.

---

## Section 4 — Recording resolved answers

After `pmpo-elicit-resume.sh` returns successfully, the calling stage MUST record
the `elicitation_id` and `provenance` alongside the resolved value. This makes
implicit decisions explicit and auditable.

**In `decision-log.md` (kbd-analyze pattern):**
```markdown
### <timestamp> — Contested stack choice
Options: <A> vs <B> | Score gap: <N>%
Decision: <chosen> | Provenance: <user|research|implicit>
Elicitation ID: <id>
```

**In `goal.json` (kbd-goal pattern):**
```json
{
  "phases": [{
    "name": "ideation",
    "human_gate_result": {
      "decision": "<candidate>",
      "provenance": "user",
      "elicitation_id": "<id>"
    }
  }]
}
```

**In `loops/<name>/journal.md` (pmpo-outer-loop pattern):**
```markdown
### <timestamp> — Loop stall escalation
Loop: <name> | Ticks stalled: <N>
Decision: continue | Provenance: user
Elicitation ID: <id>
```
