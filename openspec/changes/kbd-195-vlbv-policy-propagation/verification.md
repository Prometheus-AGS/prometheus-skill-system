# Production integration acceptance

Parent phase: issues-194-195-196. Scenario definitions only; executable test authoring and execution are deferred until every parent production change is complete. The phase adopted-contract.json is the single finite acceptance source; the owner-approved content is canonically bound at plan revision 19 with SHA-256 c6c2db6d12c9588a9213fe7f53efbd316cab2155c432c45a814307ebbf0f5734. No acceptance run is claimed.

Source identity: bfe572ff8a97a21c7610c44ea6b39d5f97a857b7 is planning baseline only. Execute must freeze actual candidate/build/generated/configuration/reference/fixture hashes.

## POL-001: Fresh delegated and resumed policy agreement

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-195-vlbv-policy-propagation/2 |
| Production entry point | context-bootstrap production entry; team creator Node CLI --input <fixture>; KBD handoff/reanchor/resume production entries and selected authorized native local sessions |
| Real collaborators | bootstrap dispatcher; team compiler/export CLI; handoff/recovery subprocesses; authorized local native sessions; contract/decision store |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Lead/fresh/delegated/resumed observations identify exactly the same selected method, contract revision/digest, included value/exclusions and stage authority; unsupported enforcement is labelled instruction-only. |
| Negative control | Missing or changed reference is reported, not guessed; text-only export never substitutes for actual required native-session observation. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/POL-001/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## POL-002: Whole-value boundary and governing conflict

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-195-vlbv-policy-propagation/1 |
| Production entry point | selected local native session follows production method-context and dispatch/recovery instructions |
| Real collaborators | actual native local session; exported role/context; real command observation; host policy |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Task/hour/commit/reviewer/stale-command triggers lead to no generated-code check while production incomplete; incompatible compiler exception is surfaced without operator-policy rewriting. |
| Negative control | A request to run a test/dry-run/build/import/probe early does not create execution authority; incompatible mandatory host rule blocks dependent action. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/POL-002/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## POL-003: Supersession budgets and uncertain effects

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-195-vlbv-policy-propagation/3 |
| Production entry point | KBD child/harness handoff, pause/resume and bootstrap compaction recovery entries |
| Real collaborators | canonical decisions; handoff provenance files; actual recovery entries; selected native local session |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Latest authorized supersession replaces retired method while original evidence, candidate freshness, consumed budgets and uncertain effects remain; no retry before effect reconciliation. |
| Negative control | Stale handoff or memory cannot resurrect retired workflow, reset budget, overwrite evidence or treat uncertain effect as clean. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/POL-003/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## Exact proposed local packet

```sh
cargo build --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli
node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issues-194-195-196/adopted-contract.json --prometheus tools/prometheus-cli/target/debug/prometheus --evidence-root .kbd-orchestrator/phases/issues-194-195-196/evidence
node scripts/generate-skill-system-distribution.js --check
```

The integration driver is a future boundary-authored acceptance artifact, not an existing passing command. Missing driver/tools/identity/permissions is BLOCKED. PASS requires actual oracle and negative-control observation; exit 0 alone is insufficient, exit 2 never passes. Preserve failed attempts, consumed budgets and uncertain cleanup. No hosted CI, unit-only acceptance or automatic successor.
