EXECUTION: sovereign-sync-domain-adapters
Project: prometheus-skill-system
Date: 2026-07-29
Selected backend: openspec
Dispatched to: SELF (claude-code, this session, driven via /kbd-apply)
Backend rationale: openspec/ exists at project root with CLI available; all 3
  changes already have OpenSpec proposal.md/tasks.md from /kbd-plan
  (openspec/changes/change-verify-p2p-transport, change-kbd-presence-peer-auth,
  change-learner-model-e2e-test). Native backend would be less inspectable —
  OpenSpec gives spec-backed traceability for a security-sensitive change
  (peer auth) touching a live P2P sync daemon.
Backend entrypoint: /kbd-apply <change-id> (list → begin-task → implement →
  end-task loop), never bare /opsx:apply
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/sovereign-sync-domain-adapters/plan.md

MODEL ROUTING

- project.json has no model_policy block — per references/model-routing.md
  fallback rule, all changes route frontier. Logged to model-routing.log.
  All changes are self-executed by the calling frontier session (no
  external tool dispatch), so this is informational, not a routing decision.

EXECUTION SCOPE

- change-verify-p2p-transport: verify real P2P transport between the
  already-paired Mac Pro and laptop (no code change — operator verification)
- change-kbd-presence-peer-auth: wire real peer authentication into
  kbd-control presence sync (substrate/sovereign-sync)
- change-learner-model-e2e-test: add a learner-model end-to-end replication
  test mirroring the existing skill-index test (substrate/sovereign-sync)

DISPATCH CONTRACTS

All three changes are self-executed in this session — no external tool
handoff. Included for routing metadata only:

- change-verify-p2p-transport → manual (operator + this session on the Mac
  Pro side; laptop side requires the user)
  Entry: N/A — no code; see proposal.md's verification checklist
  Model class: n/a
  Concrete model: n/a
  Model rationale: pure operator verification, no reasoning task
  Progress file: .kbd-orchestrator/phases/sovereign-sync-domain-adapters/progress.json
  Handoff: record pass/fail + logs in the change's proposal.md, then archive
    or file a follow-up change per its own Impact section

- change-kbd-presence-peer-auth → SELF (claude-code)
  Entry: /kbd-apply loop over openspec/changes/change-kbd-presence-peer-auth/tasks.md
  Model class: frontier (no model_policy configured; also matches
    opsx-apply-high heuristics — security-sensitive peer-trust gate,
    crosses domains.rs + rest_api.rs, plan.md calls for rust-reviewer)
  Concrete model: session default (claude-sonnet-5) — model_policy.registry
    not configured in project.json
  Model rationale: security-sensitive trust boundary; no prior art in this
    crate for authenticated-vs-generic domain routing
  Progress file: .kbd-orchestrator/phases/sovereign-sync-domain-adapters/progress.json
  Handoff: cargo test -p sovereign-sync green, then artifact-refiner QA +
    adversarial-review diff mode before archive

- change-learner-model-e2e-test → SELF (claude-code)
  Entry: /kbd-apply loop over openspec/changes/change-learner-model-e2e-test/tasks.md
  Model class: frontier (no model_policy configured; would otherwise score
    small/low — 4 tasks, single test file, direct analog exists in
    tests/domain_sync.rs's skill-index test)
  Concrete model: session default (claude-sonnet-5)
  Model rationale: mechanical mirror of an existing test pattern; frontier
    only because model_policy is unconfigured, not because the task demands it
  Progress file: .kbd-orchestrator/phases/sovereign-sync-domain-adapters/progress.json
  Handoff: cargo test -p sovereign-sync green, then artifact-refiner QA +
    adversarial-review diff mode before archive

APPROVAL GATES

- change-kbd-presence-peer-auth: security-sensitive — user visibility on the
  diff before archive, given it changes a live trust boundary in a P2P daemon
- change-verify-p2p-transport: requires explicit user action on the laptop
  side (this session can drive the Mac Pro side only)

FALLBACK CONDITIONS

- If /kbd-apply cannot produce inspectable per-task progress for either Rust
  change, or a task turns out to need a design decision not covered by
  plan.md/proposal.md, stop and surface to the user rather than improvising
  scope — do not fall back further since openspec is already the selected
  backend.

VERIFICATION REQUIREMENTS

- cargo check -p sovereign-sync after each task (cohesive checkpoints per
  KBD implementation-first completion mode)
- cargo test -p sovereign-sync (deferred to a single consolidated run once
  both Rust changes reach zero known task gaps, per completion-mode policy)
- change-verify-p2p-transport: GET /api/v1/sync/status on both machines
  shows a non-empty peers array; content pushed from one is visible on the
  other

PROGRESS LEDGER

- [PENDING] change-verify-p2p-transport — manual (operator); not started,
  needs the user to drive the laptop side
- [DONE] change-kbd-presence-peer-auth — SELF; all 5 tasks complete. User
  chose device-key-signed SyncEnvelope (kbd_runtime::DeviceSigner, the same
  Ed25519 identity Event signing already uses) over building new
  Raft-membership-based trust after the original approach was found to
  authorize the wrong subsystem (see decisions
  `change-kbd-presence-peer-auth-paused-no-gossip-trust` and
  `...-resolved-device-signed-envelopes` in `prometheus kbd audit`).
  53/53 sovereign-sync tests pass (incl. 8 new focused auth unit tests).
  Archived.
- [DONE] change-learner-model-e2e-test — SELF; all 4 tasks complete,
  cargo test -p sovereign-sync green (45/45). Not yet archived — QA gate
  pending (see task #6).

Also fixed this session, outside the original plan scope (both committed and
pushed to origin/main independently):
- Removed the required --lease-id/--fencing-token CLI plumbing from
  phase/stage/change/task/completion/decision/blocker commands (commit
  e606118) — the runtime never consulted them for those command kinds; only
  the CLI's required-arg surface forced callers through an unnecessary
  `claim` first.
- Fixed a real production incident in apply_to_state_machine: a failed
  per-entry apply used `?` and aborted the whole batch before
  persist_state_machine ran, so the log position could never advance past a
  bad entry — every later write got stuck behind it, and the sovereign-sync
  daemon crash-looped on restart replay (commit 4e3308d). Verified live:
  stopped the crash-looping daemon, confirmed the fix survives restart with
  the historical bad entries present, handed back to launchd cleanly.

OUTPUTS

- NONE yet — populated as each change completes

BLOCKERS

- change-verify-p2p-transport needs the user to run the laptop-side daemon
  restart/push steps in tasks.md — this session can prepare and drive the
  Mac Pro side but cannot act on a second physical machine

REFLECTION HANDOFF

- Whether real P2P transport was confirmed live (or what follow-up change
  was filed if not) — determines whether "sovereign-sync-domain-adapters"
  can be considered production-safe for real network use, independent of
  the in-process test coverage already proven
- cargo test -p sovereign-sync pass/fail status after both Rust changes
- Any deviation from plan.md's authentication-source decision for
  change-kbd-presence-peer-auth (KbdStateV2.devices vs. a new mapping) —
  record as a Decision if it diverges from the plan's suggestion

EXECUTION READY
