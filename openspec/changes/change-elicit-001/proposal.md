---
id: change-elicit-001
title: Async checkpoint/resume infrastructure for pmpo-elicit
phase: pmpo-elicit
gaps: [G-01, G-02, G-04]
goals: [G4]
priority: HIGH
effort: M
agent: claude-code
status: done
scope:
  - skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh
  - skills/process/pmpo-elicit/scripts/pmpo-elicit-resume.sh
  - skills/process/pmpo-elicit/references/checkpoint-contract.md
---

# change-elicit-001 — Async checkpoint/resume infrastructure

## Context

The current SKILL.md documents `AskUserQuestion` (inline-fallback) and mentions a future
child-isolated mode. Non-Claude-Code platforms (Codex, OpenCode, Kimi, Zed standalone)
cannot use `AskUserQuestion` — they need a file-based async mechanism: write a checkpoint,
pause the loop, wait for the user to write `result.json`, then resume.

This change ships the two scripts and the contract doc that all subsequent wiring changes
depend on.

## Scope

**New files:**

### `skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh`

Bash script. Arguments:
```
pmpo-elicit-checkpoint.sh <elicit-dir> <question> <criticality> <caller> [hint1] [hint2] ...
```

Behavior:
1. Creates `<elicit-dir>/` (including parent dirs)
2. Generates `id` as `<caller>-<unix-timestamp-seconds>`
3. Writes `<elicit-dir>/request.json` conforming to `elicitation.schema.json`:
   ```json
   {
     "kind": "request",
     "id": "<generated-id>",
     "question": "<question>",
     "criticality": "<criticality>",
     "caller": "<caller>",
     "hints": ["<hint1>", ...],
     "context": "",
     "write_back_path": ""
   }
   ```
4. Writes `<elicit-dir>/checkpoint.json`:
   ```json
   {
     "id": "<generated-id>",
     "caller": "<caller>",
     "timestamp": "<ISO-8601>",
     "status": "pending"
   }
   ```
5. Writes `<elicit-dir>/request-prompt.txt` — human-readable:
   ```
   [pmpo-elicit] Question from <caller>
   ID: <id>

   <question>

   Hints: <hint1>, <hint2>, ...
   Criticality: <criticality>

   To respond, write result.json in this directory:
   {
     "kind": "result",
     "id": "<id>",
     "answer": "<your answer>",
     "provenance": "user"
   }
   ```
6. Exits with code 2 (BLOCKED — awaiting elicitation result)

### `skills/process/pmpo-elicit/scripts/pmpo-elicit-resume.sh`

Bash script. Arguments:
```
pmpo-elicit-resume.sh <elicit-dir>
```

Behavior:
1. Checks `<elicit-dir>/result.json` exists; if not, exits 1 with error message to stderr
2. Validates `result.json` has `kind == "result"` and `answer` is non-null; if invalid, exits 1
3. Outputs JSON to stdout:
   ```json
   {"answer": "<answer>", "provenance": "<provenance>", "id": "<id>"}
   ```
4. Updates `<elicit-dir>/checkpoint.json → status` to "resolved"
5. Exits 0

### `skills/process/pmpo-elicit/references/checkpoint-contract.md`

Documents:
- `<elicit-dir>` path convention: `<caller-state-dir>/elicitations/<caller>-<timestamp>/`
- Files written by checkpoint.sh: `request.json`, `checkpoint.json`, `request-prompt.txt`
- Files written by the operator (or Claude Code's AskUserQuestion handler): `result.json`
- Files read by resume.sh: `result.json`
- How a caller integrates: call checkpoint.sh → if exit 2, pause; when result.json appears, call resume.sh → apply answer
- How Claude Code bypasses checkpoint/resume: uses AskUserQuestion, writes result.json directly, calls resume.sh
- Caller state fields in `checkpoint.json` (extensible: callers may add their own fields)

## Tasks

- [ ] 1. Write `scripts/pmpo-elicit-checkpoint.sh` with executable permission
- [ ] 2. Write `scripts/pmpo-elicit-resume.sh` with executable permission
- [ ] 3. Write `references/checkpoint-contract.md`
- [ ] 4. `npm run validate:strict skills/process/pmpo-elicit` passes clean
