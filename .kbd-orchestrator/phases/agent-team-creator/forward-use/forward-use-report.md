# Agent-team forward-use evidence

Request: redesign an existing mobile checkout, implement it, obtain independent review, balanced budget; start in Codex, reviewer uses Claude Code.

## Actual result

Prepared three roles: designer (Codex, design specification), implementer (Codex, mobile implementation and verification), reviewer (Claude Code, independent findings). The guide initially proposed implementer/designer/mobile-specialist/reviewer; the skill explicitly permits refinement, so mobile expertise was combined with implementation for a smaller team. All source paths remain proposed because the exercise supplied no actual application or framework.

The actual compiled CLI ran 18 invocations; 17 exited 0. The intentional premature implementation start exited 1: `{"error":"Dependency design-checkout is not complete"}`. State remained byte-identical after that rejection: True.

Three models-select calls returned selected:null with an empty declared availability list. No available native IDs, prices or capabilities were invented. The exported native model fields are omitted (inheritance); medium design/implementation and hard review remain unresolved declared policy, not verified configured model choices.

Both codex and claude exports succeeded. All three Codex files parsed with Python's standard tomllib; Claude frontmatter and role prompt were inspected. Native CLIs, plugin installation and live invocation were not run. The adapter's source-only diagnostics remain in each export response and receipt.

Local state revision is 4. Design, implementation and review are all pending. The review task currently belongs to implementer/Codex solely for dispatch preparation and depends on implement-checkout; its prepared handoff targets reviewer/Claude. This preserves a distinct review task instead of transferring implementation ownership and conflating the roles.

Handoff ID: `bdd62d67-49b0-427c-adb0-75b79706c411`. It is unaccepted; ownership has not transferred. `acceptance.request.json` is a future request only. Re-read current state and refresh expectedRevision before destination acceptance; do not run the saved request blindly after other task changes. No product design, code implementation, verification or review was marked complete.

## Observed ambiguities and remaining work

- The guide's complex mobile/design request proposes a separate mobile specialist even though an implementer is already present. The supplied guidance successfully supports reducing the team; this is an editable heuristic, not a runtime failure.
- The manifest has one default harness and no per-role harness field. Actual mixed-harness assignment belongs to task records and handoff destinations. Exporting each target produces all roles, so the operator must select the Codex designer/implementer and Claude reviewer definitions rather than install both whole sets unintentionally. A concrete mixed-harness recipe would make this step easier to discover.
- Preparing an independent review handoff before work exists requires keeping review as a distinct dependent task. I used implementer/Codex as the dispatch owner until reviewer/Claude accepts. The skills explain transfers correctly but do not give this peer-review preparation example; no contradictory completion was necessary.
- Framework, actual project ownership paths, installed destination skills and model availability remain unverified. Suggested skill IDs were removed rather than claimed installed. These are missing scenario inputs, not evidence of a passing implementation.
- The handoff correctly records the containing real Git worktree and dirty=true, not a fabricated clean application repository. This exercise's scratch folder is inside that worktree. Native execution proof and cross-platform certification remain absent.

## Evidence inventory

- `commands.json`: every exact argv, cwd, exit code and response filename.
- `01-guide.request.json` / `01-guide.stdout.json`: original intake and four-role proposal.
- `team.json`, `scope-notes.md`: reviewed three-role definition and assumptions.
- `02-model-*.stdout.json`: no-model selection explanations.
- `export-codex/` and `export-claude/`: staged native definitions and provenance receipts.
- `06-design*`, `07-implement*`, `08-review*`: assignment requests/results with actual revisions.
- `state.json`, `11-final-status.stdout.json`: pending tasks and unaccepted packet.
- `handoff-packet.json`, `reviewer-prompt.md`, `acceptance.request.json`: prepared cross-harness context and future acceptance request.
- `12-premature-start.stderr.json`: dependency rejection without state mutation.

No skill/runtime/test sources, global installs, network writes, native LLM calls, real application files, commits or canonical KBD state were changed.
