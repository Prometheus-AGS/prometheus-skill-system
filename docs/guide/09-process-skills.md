# 09 · Process & orchestration skills

Process skills organize intent, state and ownership. Their prompts can compose
workflows, but a diagram or model-routing directive does not launch collaborators
or prove an inference route. Read [Loop Architecture](03-loop-architecture.md)
for the distinction between standing-loop instructions and executable helpers.

## Choose the entry point

| Skill | Request | Responsibility |
|---|---|---|
| Iterative evolver | `/evolve "<name>"` | Organize assess, analyze, plan, execute and reflect for a domain. |
| KBD | `/kbd-init`, `/kbd-plan`, `/kbd-execute`, `/kbd-apply <change>` | Coordinate canonical engineering tasks and their receipts. |
| Elicitation | `/pmpo-elicit "<question>"` | Resolve an unknown from a user, source, bounded research or recorded assumption. |
| Standing loop | `/loop-define <name>`, `/loop-tick <name>`, `/loop-report <name>` | Define feedback and bounded continuation; the shell tick helper does not launch an evolver. |
| Skill creator | `/create-skill`, `/clone-skill`, `/extend-skill` | Produce or refine a reusable skill package. |
| Native-agent generator | `/create-native-agent` | Produce source for a separately operated service. |
| Model bridge | `/liter-llm-bridge install`, `configure`, `route` | Configure gateway routes; actual completions need independent evidence. |
| Ideation | `/ideation-mindmap <concept>` | Produce a structured concept map using the available path and disclosed fallback. |
| Evolution brief | `/kbd-evolve [name]` | Research candidate future work without accepting it as an active task. |
| Strategy evolver | `/pmpo-evolver [project-name]` | Choose among competitive, trend, unique-product, idea-validation and self-learning perspectives. |
| Goal definition | `/kbd-goal [phase-name]` | Record goals, success criteria and scope. |
| Goal assessment | `/kbd-goal-check [phase-name]` | Compare recorded evidence with goals; do not certify unfinished production. |

Slash requests are instructions consumed by the selected harness. Read the installed
skill's contract for arguments, state provider and prerequisites; they are not shell
executables or a guarantee that every tool exists in the current session.

## Canonical lifecycle and ownership

For current KBD projects, `.prometheus/project.json` identifies the repository and
the signed local journal owns state. `.kbd-orchestrator/` contains phase artifacts
and compatibility projections. A direct edit to `progress.json` or a waypoint
cannot create an accepted task, append a causal event or acquire another role's path.

The lifecycle is assess → analyze → spec → plan → execute → reflect. Plan records
scope, role/path ownership and intended model routes. The driver starts and completes
tasks through canonical claims and receipts, preserving stale-revision and conflict
failures. [Agent Teams](24-agent-teams.md) covers those executable request contracts.

Optional evolver bridge artifacts correlate strategic items with KBD results; absent
bridges leave the workflows independent. Prompts may request composition, but the
caller must actually dispatch it and retain the result. In particular, a shell loop
counter or a recorded perspective is not evidence that a strategic model ran.

## Implementation and evidence boundaries

Complete every planned production change in the phase before tests, validators,
compiler checks or review. Then run the applicable local full-integration gate and
record its command, environment and result. Keep reviewer and verifier roles dormant
until that boundary. A creator's score, schema pass or successful model call cannot
substitute for production-path acceptance.

Updates proposed from learning remain reviewable diffs. The creator's explicit
`--update` procedure requires approval before modifying an installed skill; it is
not an automatic self-rewriting loop. Memory publication keeps owner and visibility
scope and must not turn remote text into portable authority.

Use [Task model assignments](/docs/kbd/task-model-assignments) for planned versus
actual route evidence, and [Service operations](26-service-operations.md) for optional
services and their platform prerequisites. The pack does not own Companion's
replication daemon or require it for ordinary local KBD authority.

*Previous: [Skills Overview](08-skills-overview.md) · Next: [Language Skills](10-language-skills.md)*
