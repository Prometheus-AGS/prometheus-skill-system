## ADDED Requirements

### Requirement: Handoff is lease-free and read-only on KBD state
`/kbd-handoff` SHALL NOT call any lease, heartbeat or ownership command and SHALL NOT
write `progress.json`, `current-waypoint.json`, `position.json`, `position-reminder.txt`
or task checkboxes. Its only writes SHALL be the packet and its JSON sidecar under
`.kbd-orchestrator/phases/<phase>/harness-handoffs/`, never under `handoffs/`.

#### Scenario: Packet written for a supported harness
- **WHEN** the operator runs `/kbd-handoff --to kimi-code` with no open change or task boundary obligation
- **THEN** the skill SHALL write `harness-handoffs/kimi-code-<UTC timestamp>.md` and a `.json` sidecar that validates against `harness-handoff.schema.json`
- **AND** the packet SHALL contain Position, Repositories, Read in order, Pending operator gates and decisions, Task execution rules, Never do, Open prerequisites and Start prompt sections

#### Scenario: Dry run
- **WHEN** the operator passes `--dry-run`
- **THEN** the skill SHALL print the packet and write nothing

### Requirement: Handoff refuses a half-closed boundary
The skill SHALL classify each open obligation in `prometheus kbd status --json`. It SHALL
ignore obligations on the active phase path and on the active change while that change is
not complete. It SHALL exit 1 without writing on an open `task` obligation, or on a
`change` or `phase` obligation the runtime marks complete or does not know, printing
`prometheus kbd guard evaluate --boundary <boundary> --edge after --subject <subject> --json --repair-projections`
for each. It SHALL warn, as carried work, on a `change` or `phase` obligation outside the
active position that was started and not finished. When status is unavailable it SHALL
fall back to the projections and record that obligations were not checked.

#### Scenario: Interrupted close
- **WHEN** `boundaryObligations` contains a `task` entry, or a `change` entry whose change the runtime marks complete
- **THEN** the skill SHALL exit 1, print the repair command, and write no packet

#### Scenario: Active and carried work
- **WHEN** the only open obligations are the active phase, the active in-progress change, and a pending change of an earlier phase
- **THEN** the skill SHALL write the packet and list the pending change as carried work

### Requirement: Target harness is validated and checked
The skill SHALL reject a `--to` value that is not a supported harness. For a supported
harness it SHALL report every installed `kbd-*` skill entry that does not resolve to a
directory containing `SKILL.md`, and, when the project has an agent team, every role
without a native definition for that harness. These findings SHALL be warnings recorded
in the packet, not refusals.

#### Scenario: Unknown harness
- **WHEN** the operator runs `/kbd-handoff --to notepad`
- **THEN** the skill SHALL exit 2 and list the supported harnesses (`claude-code`, `codex`, `kimi-code`, `opencode`)

#### Scenario: Broken installed skill link
- **WHEN** the target harness's skills directory holds a `kbd-handoff` symlink to a removed path
- **THEN** the packet SHALL list that link and its target under target harness readiness and open prerequisites
