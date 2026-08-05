EXECUTION: prometheus-exec-code-execution-engine
Project: prometheus-skill-system
Date: 2026-08-04
Selected backend: openspec
Dispatched to: current Codex session through kbd-apply
Backend rationale: The four changes introduce security and execution boundaries across multiple crates and require spec-to-test traceability. OpenSpec is already available and KBD owns every task transition through kbd-apply.
Backend entrypoint: /kbd-apply
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/prometheus-exec-code-execution-engine/plan.md

EXECUTION SCOPE

- change-exec-001-contracts-verification: portable contracts, signed receipts, offline verification, and immutable receipt segments
- change-exec-002-tier-p-sidecar: sandboxed native execution, CAS, policy/grants, idempotent service, and local sidecar
- change-exec-003-tier-w-mobile: Wasmtime 46 component execution and embedded/mobile interfaces
- change-exec-004-remote-mcp-docs: remote dispatch, MCP parity, installation, canonical docs, and certification

DISPATCH CONTRACTS

- change-exec-001-contracts-verification → OpenSpec through kbd-apply
  Entry: /kbd-apply change-exec-001-contracts-verification
  Model class: frontier
  Concrete model: current Codex session
  Model rationale: new portable cryptographic envelope and archive format across Rust/CLI/schema boundaries
  Progress file: .kbd-orchestrator/phases/prometheus-exec-code-execution-engine/progress.json
  Handoff: implementation committed locally as 632981a; evidence and checklist complete

- change-exec-002-tier-p-sidecar → OpenSpec through kbd-apply
  Entry: /kbd-apply change-exec-002-tier-p-sidecar
  Model class: frontier
  Concrete model: current Codex session
  Model rationale: OS sandbox, authorization, durable service, and API boundaries are security-critical and cross-domain
  Progress file: .kbd-orchestrator/phases/prometheus-exec-code-execution-engine/progress.json
  Handoff: macOS Tier P implementation and local certification complete at `1b8d905`; public evidence is under `evidence/change-exec-002-tier-p-sidecar*`. Linux runtime and Windows Tier P remain explicitly unavailable/pending as recorded in the evidence.

- change-exec-003-tier-w-mobile → OpenSpec through kbd-apply
  Entry: /kbd-apply change-exec-003-tier-w-mobile
  Model class: frontier
  Concrete model: current Codex session
  Model rationale: Wasmtime component-model execution plus cross-platform FFI and supply-chain verification requires architectural consistency
  Progress file: .kbd-orchestrator/phases/prometheus-exec-code-execution-engine/progress.json
  Handoff: external platform evidence remains pending unless run on the named platform/device

- change-exec-004-remote-mcp-docs → OpenSpec through kbd-apply
  Entry: /kbd-apply change-exec-004-remote-mcp-docs
  Model class: frontier
  Concrete model: current Codex session
  Model rationale: remote authorization, surface parity, installation, and canonical documentation cross the complete product boundary
  Progress file: .kbd-orchestrator/phases/prometheus-exec-code-execution-engine/progress.json
  Handoff: GitHub remains deployment-only; publication requires explicit user authorization

APPROVAL GATES

- Privileged executions require an SSH-signed grant or trusted-host interactive approval; no auto-approval for network egress or writes outside outputs/.
- Pushes, PRs, merges, and external configuration changes require explicit user authorization.
- Linux, Windows, iOS, Android, physical-device, and remote multi-peer claims remain pending without their named environment.

FALLBACK CONDITIONS

- If an OS cannot provide the claimed Tier P sandbox, report Tier P unavailable and never run unsandboxed while emitting an attested receipt.
- If a task cannot remain inspectable through OpenSpec/KBD, stop that task and update the design rather than bypassing the ledger.

VERIFICATION REQUIREMENTS

- Local Rust format, check, warnings-denied Clippy, unit, integration, property, sandbox escape, idempotency, restart, and generated-diff checks for affected crates.
- Local documentation and workflow-policy gates only; GitHub Actions are not product-test evidence.
- Redacted evidence records distinguish locally certified, externally pending, and installed-service states.

PROGRESS LEDGER

- [DONE] change-exec-001-contracts-verification — OpenSpec/kbd-apply-equivalent task ledger, local commit 632981a
- [DONE] change-exec-002-tier-p-sidecar — OpenSpec through kbd-apply, implementation commit `1b8d905`, 86 local tests and three Linux-musl cross-Clippy gates passing
- [DONE] change-exec-003-tier-w-mobile — OpenSpec through kbd-apply, implementation commit `e929449`, 130 local tests, deterministic FRB dispatcher, signed replay evidence, and distinct-model review passing; mobile release size and physical-device evidence remain explicitly pending
- [PENDING] change-exec-004-remote-mcp-docs — OpenSpec through kbd-apply

OUTPUTS

- Four dependency-ordered implementation commits plus local evidence and publication-ready branch state

BLOCKERS

- NONE for completed desktop Tier P and Tier W work on the certified macOS host. Linux kernel runtime certification, Windows Tier P, the mobile 12 MiB size gate, and physical-device evidence remain unavailable or pending as recorded; none is represented as green.

REFLECTION HANDOFF

- Compare every claim against the four evidence records, unresolved platform dispositions, installed binary state, documentation parity, and branch publication state.

EXECUTION READY
