# adversarial-review-gating Specification

## Purpose
TBD - created by archiving change change-arc-001-ratify-goal-wording. Update Purpose after archive.
## Requirements
### Requirement: Producer model is required, never synthesized

A creator SHALL refuse to dispatch an adversarial review when `KBD_PRODUCER_MODEL` is
unset, rather than substituting a default value. A synthesized producer identity would
cause the review to record `cross_model_check: verified-distinct` for a comparison that
never occurred.

#### Scenario: Unset producer model refuses dispatch

- **GIVEN** a creator about to dispatch an adversarial review
- **WHEN** `KBD_PRODUCER_MODEL` is unset or empty
- **THEN** the creator exits with status 2
- **AND** no findings artifact is written
- **AND** an explanatory refusal is emitted on stderr.

#### Scenario: Set producer model permits dispatch

- **GIVEN** a creator about to dispatch an adversarial review
- **WHEN** `KBD_PRODUCER_MODEL` names the model running the session
- **THEN** the review dispatches
- **AND** the findings artifact records `cross_model_check: verified-distinct`.

### Requirement: Single enforced sycophancy gate

`validate-skill.sh` SHALL be the single enforcement point for the sycophancy screen, and
SHALL invoke the existing `check-findings-sycophancy.sh` helper rather than reimplementing
its logic. Creators SHALL invoke `validate-skill.sh` and SHALL NOT call the helper directly.

#### Scenario: Sycophancy failure fails validation

- **GIVEN** a generated skill whose report contains no edge cases or failure modes
- **WHEN** `validate-skill.sh` runs
- **THEN** the sycophancy check group reports a failure
- **AND** the helper's non-zero exit is propagated into the `FAIL` counter
- **AND** the feedback appears in the `=== RESULT ===` block.

#### Scenario: Creators do not bypass the gate

- **GIVEN** a creator completing its Reflect phase
- **WHEN** it enforces the sycophancy screen
- **THEN** it invokes `validate-skill.sh`
- **AND** it does not invoke `check-findings-sycophancy.sh` directly.

### Requirement: Shared producer-model guard

The producer-model refusal SHALL be implemented once, as `kbd_require_producer_model()` in
`shared/scripts/lib/kbd-model-resolve.sh`, and SHALL be sourced by every creator entry point
that dispatches an adversarial review. A per-creator reimplementation would let the two
creators drift, so that one fails closed and the other silently fabricates a producer.

This requirement covers the *mechanism*. The behavioural contract it enforces —
"Producer model is required, never synthesized" — is already specified by
`change-arc-001-ratify-goal-wording` and is not restated here.

#### Scenario: Guard is defined in the shared resolver

- **GIVEN** the shared model-resolution library
- **WHEN** it is sourced
- **THEN** `kbd_require_producer_model` is defined as a shell function
- **AND** it is defined in exactly one file across the repository.

#### Scenario: Guard returns 2 without side effects

- **GIVEN** the shared library has been sourced
- **WHEN** `kbd_require_producer_model` is called with `KBD_PRODUCER_MODEL` unset or empty
- **THEN** it returns status 2
- **AND** it writes an explanatory refusal to stderr naming the variable
- **AND** it does not assign any default value to `KBD_PRODUCER_MODEL`.

#### Scenario: Guard is transparent when the producer is known

- **GIVEN** the shared library has been sourced
- **WHEN** `kbd_require_producer_model` is called with `KBD_PRODUCER_MODEL` set to a non-empty value
- **THEN** it returns status 0
- **AND** it writes nothing to stderr
- **AND** the value of `KBD_PRODUCER_MODEL` is unchanged.

#### Scenario: Both creator entry points invoke the guard

- **GIVEN** the skill creator and the agent creator
- **WHEN** either is about to dispatch an adversarial review
- **THEN** it has sourced `shared/scripts/lib/kbd-model-resolve.sh`
- **AND** it calls `kbd_require_producer_model` before building the review packet
- **AND** a non-zero return aborts the dispatch rather than being logged and ignored.

### Requirement: Skill review packet mode

`build-review-packet.sh` SHALL support `--mode skill`, producing a manifest-level packet for a
generated SKILL.md tree. The packet SHALL carry the `SKILL.md` body, its parsed frontmatter, an
inventory of `scripts/`, a cross-reference map of `references/` links, the output of the skill
validator, and the original generation intent. Full script source SHALL NOT be included — the
judge reviews the manifest, not the implementation.

#### Scenario: Skill packet carries the manifest surface

- **GIVEN** a generated skill directory containing `SKILL.md`, `scripts/`, and `references/`
- **WHEN** `build-review-packet.sh --mode skill` runs against it
- **THEN** the packet contains the `SKILL.md` body and its parsed frontmatter
- **AND** the packet contains a script inventory listing each file in `scripts/`
- **AND** the packet contains a cross-reference map of links into `references/`
- **AND** the packet contains the validator output and the original generation intent.

#### Scenario: Skill packet excludes full script source

- **GIVEN** a generated skill whose `scripts/` directory contains executable files
- **WHEN** the skill packet is built
- **THEN** the packet records each script's path and purpose
- **AND** the packet does not embed the full body of any script.

### Requirement: Agent review packet mode

`build-review-packet.sh` SHALL support `--mode agent`, producing a manifest-level packet for a
generated Cargo workspace. The packet SHALL carry `agent.toml`, `system_prompt.md`, the
workspace member list with a stated purpose per crate, the configured `mcp_servers`, and the
`cargo check` result. A generated workspace exceeds any judge's context window, so full source
SHALL NOT be included.

#### Scenario: Agent packet carries the workspace surface

- **GIVEN** a generated agent workspace
- **WHEN** `build-review-packet.sh --mode agent` runs against it
- **THEN** the packet contains `agent.toml` and `system_prompt.md`
- **AND** the packet lists each workspace member with its stated purpose
- **AND** the packet lists the configured `mcp_servers`
- **AND** the packet records the `cargo check` result.

#### Scenario: Agent packet excludes crate source

- **GIVEN** a generated workspace with multiple member crates
- **WHEN** the agent packet is built
- **THEN** the packet does not embed the source of any crate.

### Requirement: Truncation is recorded, never silent

Each creation packet SHALL be capped in size, and when the cap truncates content the packet
SHALL record that truncation occurred and what was dropped. A truncated packet that looks
complete would let a judge return `PASS` on material it never saw.

#### Scenario: Truncation is disclosed inside the packet

- **GIVEN** an artifact whose packet content exceeds the configured cap
- **WHEN** the packet is built
- **THEN** the packet contains a truncation record naming the cap and the omitted sections
- **AND** the truncation record is inside the packet the judge receives.

#### Scenario: An uncapped packet records no truncation

- **GIVEN** an artifact whose packet content fits within the cap
- **WHEN** the packet is built
- **THEN** the packet records no truncation
- **AND** the packet is not marked as partial.

### Requirement: Skill creator dispatches an adversarial review

`pmpo-skill-creator` SHALL dispatch `/adversarial-review --mode skill` during its Reflect
phase, after `validate-skill.sh` has run and before the loop-decision table is evaluated.
Reviewing before validation would judge an artifact that is already known to be malformed;
reviewing after the loop decision would let a skill ship before it was judged.

#### Scenario: Review runs between validation and the loop decision

- **GIVEN** the skill creator reaching its Reflect phase
- **WHEN** the phase executes
- **THEN** `validate-skill.sh` runs first
- **AND** `/adversarial-review --mode skill` is dispatched next
- **AND** the loop-decision table is evaluated only after the review returns.

### Requirement: Sycophancy screen is an enforced validator check group

`validate-skill.sh` SHALL carry a check group that shells out to
`check-findings-sycophancy.sh`, propagate that helper's non-zero exit into its own `FAIL`
counter, and surface the helper's feedback in the existing `=== RESULT ===` block. A helper
whose exit is discarded is advisory, not a gate.

#### Scenario: A sycophantic report fails validation

- **GIVEN** a generated skill whose report names no edge cases or failure modes
- **WHEN** `validate-skill.sh` runs
- **THEN** the sycophancy check group reports a failure
- **AND** the `FAIL` counter is incremented
- **AND** `validate-skill.sh` exits non-zero
- **AND** the helper's feedback appears in the `=== RESULT ===` block.

#### Scenario: A clean report does not fail validation

- **GIVEN** a generated skill whose report states its limitations and failure modes
- **WHEN** `validate-skill.sh` runs
- **THEN** the sycophancy check group passes
- **AND** the `FAIL` counter is not incremented by that group.

### Requirement: CRITICAL findings block, bounded by a two-round retry

A CRITICAL finding SHALL block the skill creator from declaring the skill ready. The creator
SHALL re-review after a fix, SHALL stop after two rounds, and SHALL then append an
"Unresolved review findings" section rather than looping indefinitely or silently passing.

#### Scenario: A CRITICAL finding triggers a fix and re-review

- **GIVEN** a review returning a CRITICAL finding on the first round
- **WHEN** the creator applies a fix
- **THEN** it re-reviews the corrected artifact
- **AND** it does not declare the skill ready on the basis of the first review.

#### Scenario: Repeated CRITICALs stop at two rounds

- **GIVEN** a review returning a CRITICAL finding on both rounds
- **WHEN** the second round completes
- **THEN** the creator stops re-reviewing
- **AND** it appends an "Unresolved review findings" section naming the surviving findings
- **AND** it does not report the skill as clean.

### Requirement: Agent creator dispatches an adversarial review before declaring ready

The native-agent generator SHALL dispatch `/adversarial-review --mode agent` at the end of
`generate.md`, after `cargo check` and `npm install` have run and before the readiness banner
is emitted. `cargo check` proves the workspace compiles; it says nothing about whether the
agent is any good, so a banner emitted before the review would declare an unjudged artifact
ready.

#### Scenario: Review runs after build steps and before the banner

- **GIVEN** the agent generator completing a workspace
- **WHEN** generation reaches its final stage
- **THEN** `cargo check` and `npm install` have already run
- **AND** `/adversarial-review --mode agent` is dispatched next
- **AND** the readiness banner is emitted only after the review returns.

### Requirement: The review blocks the readiness declaration, not the workspace

A blocking review SHALL suppress only the readiness declaration. The generated workspace
SHALL persist on disk regardless of the verdict, so the operator can inspect and repair what
was flagged rather than losing the work.

#### Scenario: A CRITICAL finding suppresses the banner but keeps the workspace

- **GIVEN** a generated workspace whose review returns a CRITICAL finding
- **WHEN** the generator finishes
- **THEN** the readiness banner is not emitted
- **AND** the generated workspace remains on disk
- **AND** the findings artifact is written alongside it.

#### Scenario: A clean review permits the banner

- **GIVEN** a generated workspace whose review returns no CRITICAL findings
- **WHEN** the generator finishes
- **THEN** the readiness banner is emitted
- **AND** the workspace remains on disk.

### Requirement: Agent creator uses the same bounded retry as the skill creator

The agent creator SHALL apply the same two-round retry loop as the skill creator, appending
an "Unresolved review findings" section when CRITICAL findings survive both rounds. One
retry policy across both creators keeps the bound from drifting between them.

#### Scenario: Repeated CRITICALs stop at two rounds

- **GIVEN** an agent review returning a CRITICAL finding on both rounds
- **WHEN** the second round completes
- **THEN** the creator stops re-reviewing
- **AND** it appends an "Unresolved review findings" section
- **AND** the readiness banner is not emitted.

### Requirement: The gate is proven to discriminate, not merely to run

Committed fixtures SHALL prove that the review gate distinguishes flawed artifacts from clean
ones. A test asserting only that a review completes would have passed throughout the period in
which eight consecutive reviews were same-model self-grades that all returned `PASS`.

Four fixtures SHALL be committed: `flawed-skill`, `flawed-agent`, `clean-skill`, `clean-agent`.

#### Scenario: Flawed fixtures are blocked and cross-model verified

- **GIVEN** the `flawed-skill` and `flawed-agent` fixtures
- **WHEN** each is submitted to the review gate
- **THEN** the verdict is `BLOCK`
- **AND** the findings artifact records `cross_model_check: verified-distinct`.

#### Scenario: Clean fixtures pass

- **GIVEN** the `clean-skill` and `clean-agent` fixtures
- **WHEN** each is submitted to the review gate
- **THEN** the verdict is `PASS`.

#### Scenario: An inverted result fails the suite

- **GIVEN** a run in which a flawed fixture passes or a clean fixture is blocked
- **WHEN** the suite evaluates the results
- **THEN** the suite exits non-zero
- **AND** the inversion is named in the failure output.

### Requirement: Fail-closed behaviour is asserted per creator

The suite SHALL assert the producer-model refusal at both creator entry points, since a guard
that exists but is unsourced by one creator fails silently exactly where it matters.

#### Scenario: Each creator refuses with the producer model unset

- **GIVEN** the skill creator and the agent creator
- **WHEN** each is invoked with `KBD_PRODUCER_MODEL` unset
- **THEN** each exits with status 2
- **AND** no findings file is written by either
- **AND** each emits a refusal on stderr.

### Requirement: The retry bound is asserted per creator

The suite SHALL assert that repeated CRITICAL findings terminate at two rounds for both
creators and that the "Unresolved review findings" section is appended.

#### Scenario: Both creators stop at two rounds

- **GIVEN** a fixture that yields a CRITICAL finding on every round
- **WHEN** each creator processes it
- **THEN** each stops after two review rounds
- **AND** each appends an "Unresolved review findings" section.

### Requirement: The suite is bounded and does not run on every commit

The suite SHALL cap live judge calls at six per run and SHALL execute on demand and at the
release gate only. An unbounded cross-model suite on every commit would make the gate too
expensive to keep enabled, and a disabled gate proves nothing.

#### Scenario: Judge calls stay within the ceiling

- **GIVEN** a full run of the fixture suite
- **WHEN** the run completes
- **THEN** no more than six live judge calls were issued.

#### Scenario: The suite does not run on ordinary commits

- **GIVEN** an ordinary commit to the repository
- **WHEN** the standard checks run
- **THEN** the fixture suite is not invoked
- **AND** it remains invocable on demand and at the release gate.

### Requirement: The sycophancy-screen cap is user-overridable within a hard ceiling

`check-findings-sycophancy.sh` SHALL read its rejection cap from
`PROMETHEUS_ADV_REJECT_CAP`, defaulting to 2, following the same pattern as
`PROMETHEUS_REFLECT_STRICTNESS`. A hardcoded cap decides on the operator's behalf in contexts
the author of the constant never saw.

The override SHALL be bounded by a hard ceiling of 5. A value above the ceiling SHALL be an
error rather than being clamped or ignored, so a typo cannot silently disable the bound.

This requirement governs the **sycophancy-screen** cap only. The two-round retry cap in the
skill and agent creators is a separate bound and is unchanged.

#### Scenario: The default cap is unchanged

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is unset
- **WHEN** the sycophancy screen runs
- **THEN** the cap is 2.

#### Scenario: A value within the ceiling is honoured

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is set to a value between 1 and 5
- **WHEN** the sycophancy screen runs
- **THEN** that value is used as the cap.

#### Scenario: A value above the ceiling is an error

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is set above 5
- **WHEN** the sycophancy screen runs
- **THEN** it exits non-zero with an error naming the ceiling
- **AND** it does not silently fall back to the default or to the ceiling.

#### Scenario: The creator retry cap is unaffected

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is set to 5
- **WHEN** a creator's review retry loop runs
- **THEN** the retry loop still stops after two rounds.

### Requirement: An overridden cap is recorded in the findings artifact

When the cap is overridden, the findings artifact SHALL record `cap_overridden: true` and the
value used. An override that leaves no trace makes a lenient run indistinguishable from a
strict one after the fact.

#### Scenario: An override is recorded

- **GIVEN** a run with `PROMETHEUS_ADV_REJECT_CAP` set above the default
- **WHEN** the findings artifact is written
- **THEN** it records `cap_overridden: true`
- **AND** it records the cap value used.

#### Scenario: A default run is recorded as not overridden

- **GIVEN** a run with `PROMETHEUS_ADV_REJECT_CAP` unset
- **WHEN** the findings artifact is written
- **THEN** `cap_overridden` is false or absent.

### Requirement: The override prompt never blocks non-interactive execution

When the cap is about to be exceeded in an interactive terminal, the screen MAY prompt once,
defaulting to accept. It SHALL NOT prompt when stdin is not a TTY, so CI runs and hooks can
never hang waiting for input.

#### Scenario: A non-interactive run does not prompt

- **GIVEN** the screen running with stdin not attached to a TTY
- **WHEN** the cap is reached
- **THEN** no prompt is emitted
- **AND** the run proceeds to completion without waiting for input.

#### Scenario: An interactive run prompts at most once

- **GIVEN** the screen running in an interactive terminal
- **WHEN** the cap is reached
- **THEN** at most one prompt is emitted
- **AND** the default answer is to accept.

