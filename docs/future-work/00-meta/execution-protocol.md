# Execution Protocol

The contract a Claude Code session follows when picking up a task from this pack. Read this once at the start of any session that touches `docs/future-work/`.

## Picking up a task

1. **Read `STATUS.md`** at the root of `docs/future-work/`.
2. **Filter** to tasks where:
   - `status: ready`
   - `agent_role` matches your session's role (see `parallel-agent-routing.md`)
   - You are not already working on another task in the same `category` from a different concurrent session (avoid cross-stream contention)
3. **Pick** the highest-priority match (P0 > P1 > P2). Among same-priority tasks, prefer the one with the most `unblocks` entries — finishing it frees the most downstream work.
4. **Open** the task document at the path implied by ID (e.g. `SP-013` → `01-skill-pack-fixes/SP-013-sycophancy-reflector-hook.md`).
5. **Read it in full**, including:
   - `Problem`
   - `Evidence`
   - `Why it matters`
   - `Proposed fix`
   - **`Trade-offs and risks`** — do not skip this section
   - `Acceptance criteria`
   - `Implementation steps`
   - `Dependencies`
   - `Open questions`
6. **Do not skip the trade-offs section.** Tasks in this pack were authored with explicit honest framing. If you read only the proposed fix, you will miss the reasons the proposed fix is bounded the way it is. Several tasks (notably BDD-006) are deliberately scoped narrower than the original ask — read the why before you "improve" the scope.
7. **Update STATUS.md** before starting:
   ```yaml
   - id: SP-013
     status: in-progress
     assigned_to: claude-code-<your-session-id>
     started_at: <ISO-8601 now>
   ```
8. If `surreal-memory` is online, mirror that update into the Surreal graph. If not, continue with STATUS.md only.

## Doing the work

The task document is intentionally not a step-by-step transcript. The `Implementation steps` are a starting trajectory; deviations are expected, but they should be noted.

Required behaviours during work:

- **Stay in scope.** The task ID names the scope. If you find yourself touching files unrelated to the listed `Acceptance criteria`, stop and ask whether you should split the work into a follow-up task.
- **Run typecheck/lint after every meaningful step.** This is not a CI courtesy — it's a regression backstop given that the surrounding code has invariants several layers deep. Specifically:
  ```bash
  pnpm exec tsc --noEmit
  pnpm run lint
  ```
  for TypeScript projects, or:
  ```bash
  cargo check --workspace
  cargo clippy --workspace -- -D warnings
  ```
  for Rust crates.
- **Verify against acceptance criteria before declaring done.** Each criterion in the task doc is testable. Run the test described, paste evidence in your session, then declare done.
- **If you discover the task is wrong or redundant**, do not silently abandon it. Update STATUS.md to `status: blocked` or `status: abandoned` with a `notes:` line explaining why. The doc itself stays — it's a record, not just an instruction.

## Updating sibling resources

Most tasks update one or more of:

1. **The skill-pack itself** at `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/`.
2. **The prometheus-knowledge crate** at `/Users/gqadonis/Projects/prometheus/prometheus-knowledge/`.
3. **The SSR frontend project** at `/Users/gqadonis/Projects/sansaba/ssr-frontend/`.
4. **The doc-generation-agent** at `/Users/gqadonis/Projects/sansaba/document-generation-agent/`.

Each task doc lists the affected files in its `Implementation steps`. Treat that list as the upper bound — if you must modify more, add a `Scope expansion` note to STATUS.md.

## Finishing a task

1. Confirm every acceptance criterion is met, with evidence.
2. Run typecheck/lint/tests one more time.
3. Commit the work with a message that includes the task ID:
   ```
   feat(SP-013): wire sycophancy-correction into reflector SubagentStop hook

   See docs/future-work/01-skill-pack-fixes/SP-013-sycophancy-reflector-hook.md
   for design rationale and acceptance criteria.
   ```
4. Update STATUS.md:
   ```yaml
   - id: SP-013
     status: done
     completed_at: <ISO-8601 now>
     notes: <any divergence from the task doc, optional>
   ```
5. If finishing this task **unblocks** other tasks (i.e. they had this in `depends_on`), set those tasks' `status` to `ready` in STATUS.md.
6. If `surreal-memory` is online, mirror to Surreal.
7. **Do not auto-pick up another task.** A new session should pick up the next one. This is a deliberate context-window discipline: tasks accumulate context, and chaining them in one session leads to context exhaustion before a clean handoff point.

## Concurrency rules

Multiple Claude Code sessions can run concurrently as long as:

- They pick from different `agent_role` pools (see `parallel-agent-routing.md`), OR
- They pick tasks in the same role pool but with no overlapping file modifications.

To reduce risk of two sessions touching the same files:

- Each session should `git checkout -b future-work/<task-id>` before starting work.
- Merge to main only after the task's STATUS.md update is committed.
- The PR description should link to the task doc.

## What this protocol does NOT do

- It does not enforce the no-cross-pollination rule between PMPO Reflect-phase and the critic agent's input. That's a separate concern handled by SP-013 inside the reflector hook.
- It does not gate skill mutations through Cedar — that's SP-011's job.
- It does not validate that work conforms to KDD/PMPO. The task docs are written assuming the reader already follows those conventions.

If any of those gaps surprise you, they are intentional. This protocol is a thin coordination layer; the substantive enforcement lives in the skill-pack itself.
