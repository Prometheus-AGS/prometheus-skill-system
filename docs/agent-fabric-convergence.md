# Agent Fabric Convergence: prometheus-skill-pack

## Current capability and boundary

This planning note is grounded in
`add9b48949371f647350ee0d318bfc0672a861ff` on
`codex/agent-fabric-convergence`. The full pack already ships four related
procedures: `agent-team-creator`, `agent-team-manage`, `agent-team-models` and
`agent-team-handoff`. Their shared runtime validates a portable team manifest,
stages native exports for eight execution harnesses plus BossFang, tracks local
revisioned tasks, selects models from explicit evidence and creates accepted
handoff packets.

The current documentation is also explicit about the boundary: export does not
install or execute a team; local ownership paths are not filesystem enforcement;
task state is not a distributed lease; UAR export asserts no persistent native
team API; and neither a skill nor a handoff transfers credentials, approvals or
authority. Those are durable constraints for this initiative.

## Dependency on UAR and the convergence plan

The pack must not revise its UAR/BossFang adapters until `D-UAR-P1` accepts exact
registration, run, identity, approval and lifecycle contracts. C03 must accept
the collaboration document profile before this repository publishes it as a
standard. C09 supplies runtime team/task semantics, while C15 extends portable
adapters and C16 adds specialist business/design templates. Full/mini parity and
the exact Node.js/TypeScript pins remain gates.

No pack artifact may grant execution authority. Runtime bindings must reference
current Cedar policy and private installed grants at the destination; a portable
document may declare required authority but cannot supply it.

## Draft collaboration document profile

The standard should remain a family of separately versioned documents rather
than one overloaded “agent file.” These names come from the convergence plan and
remain draft until C03 review:

| Document | Portable responsibility | Must remain outside it |
| --- | --- | --- |
| `AgentDefinition` | identity, purpose, instructions, inputs/outputs, skills, model requirements, tools requested, evidence contract | credentials, approval decisions, installed grants |
| `TeamDefinition` | roles, dependencies, review separation, shared artifact/mailbox declarations, aggregate constraints | live task state, unioned member permissions |
| `WorkflowDefinition` | typed steps, joins, waits, retry/compensation intent, required effect classes | executor state, provider secrets, fabricated exactly-once claims |
| `DeploymentBinding` | runtime instance, placement, exact definition/skill/schema revisions, capability requirements, workspace/artifact locations and credential references | credential values, portable authority |
| `RepresentationGrant` | represented human/office, issuer, consent, organization assignment, data/communication/action scopes, disclosure, expiry and revocation | public package content; this is private installed authority |

Each document needs schema version, stable identity, provenance, required versus
optional semantics and an extension mechanism whose unknown mandatory fields
cause refusal. Compilers/exporters must report field-level loss and preserve
native options without claiming they were understood. Runtime receipts bind the
effective definition, skills, policy/grant revisions, model and deployment; a
signature proves origin/integrity, not permission.

The existing team schema is a useful v1 source, not the final runtime team model.
Its `roles`, `owns`, `dependsOn`, model policy and native source/version fields
should migrate losslessly. Live tasks, runs, attempts, effects, approvals and
memory cursors remain runtime records with their own identifiers.

## Skills and team-template responsibility

The full pack should own the canonical authored skill family, schemas, adapters,
references and source templates. Mini should receive the reviewed portable
subset with byte-for-byte runtime parity where promised. Harness-specific native
details belong in adapters and source/version receipts, while the common profile
stays vendor-neutral.

C16 templates should be outcome-focused and minimal: development, product,
marketing/brand, logo/mobile/design, documentation and customer-feedback teams.
Roles declare accountable outputs, allowed write scopes, required evidence,
skills and an independent evaluator where needed. Generic executive-office roles
may be templates, but named-human behavior and authority require the private
RepresentationGrant and C17 controls. No “CFO” or “CEO” title confers financial,
legal, publication or personnel authority.

## Codex CLI as the first reference adapter

The locally inspected `codex-cli 0.154.0` is a native arm64 executable. It reports
stable `multi_agent` and `goals` features, browses sessions through a shared local
app-server daemon, and exposes worktree, sandbox and approval controls. The
[official build instructions](https://github.com/openai/codex/blob/main/docs/install.md)
build it from the `codex-rs` Cargo workspace. This is a
strong reference for the first end-to-end adapter because it separates reusable
agent definitions from live sessions and makes task isolation and tool posture
explicit.

The pack should use Codex to prove the portable profile through a Rust-native
harness: generate reviewed `.codex/agents/*.toml`, preserve supported native
options, launch through the installed CLI contract, record exact session/task
identity and collect evidence. Codex remains the native executor and permission
owner. The adapter must not translate Codex approvals into Cedar grants, invent a
plugin `agents` field, or claim a Codex session can migrate to UAR. Refresh the
installed CLI schema and primary documentation at the C15 checkpoint; the
existing 0.144.1 evidence is historical.

## Non-goals

- Turning the skill pack or its local team ledger into a scheduler, runtime,
  Cedar authority, secrets store or always-on service.
- Shipping credentials, installed grants or named-human profiles in portable
  agent/team packages.
- Treating native option preservation as semantic validation or source export as
  live execution evidence.
- Auto-installing every skill, assigning every specialist, or giving a role
  external-send/publish/spend authority.
- Editing generated distributions directly or diverging the full and mini
  runtimes.

## Next repository-scoped KBD child

After C03 accepts the document family and `D-UAR-P1` pins the UAR adapter, create
`agent-team-collaboration-profile`. It should own the canonical schemas,
conversion diagnostics and migration fixtures in the existing agent-team skill
family, plus one Codex adapter acceptance fixture. Split specialist templates
into the later `agent-team-specialist-templates` child gated by C15/C16. The first
child must prove legacy v1 team round-trip, explicit refusal of required loss and
full/mini parity before publishing a new standard version.
