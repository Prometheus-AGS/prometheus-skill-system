# Production integration acceptance

Parent phase: issues-194-195-196. Scenario definitions only; executable test authoring and execution are deferred until every parent production change is complete. The phase adopted-contract.json is the single finite acceptance source; the owner-approved content is canonically bound at plan revision 19 with SHA-256 c6c2db6d12c9588a9213fe7f53efbd316cab2155c432c45a814307ebbf0f5734. No acceptance run is claimed.

Source identity: bfe572ff8a97a21c7610c44ea6b39d5f97a857b7 is planning baseline only. Execute must freeze actual candidate/build/generated/configuration/reference/fixture hashes.

## IDR-001: Registration/import/diagnostic integrity

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-194-change-identity-repair/1 |
| Production entry point | prometheus kbd --path <scratch> change register; real legacy import entry; kbd-apply.sh reconcile <phase> --json |
| Real collaborators | built CLI; signed journal/store; actual legacy importer; apply/reconcile subprocess; backend files |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Conflicting registration and duplicate legacy input preserve original tasks/history/bytes and report actual phase/competing IDs; duplicate projection rows are named, zero task matches distinguished from multiple matches. |
| Negative control | Equivalent command retry creates no duplicate; conflicting content, repeated legacy IDs and key/embedded-ID mismatch cannot silently overwrite or pass. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/IDR-001/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## IDR-002: Authorized append-only canonical repair

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-194-change-identity-repair/2 |
| Production entry point | prometheus kbd --path <scratch> change repair-identity --command-id <id> --phase <phase> --expected-revision <revision> --resolution-file <mapping.json> --reason <reason> (new command to implement) |
| Real collaborators | built CLI; actual signer; signed journal; repair mapping; cold replay |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Explicit authorized mapping yields one signed repair with original records/tasks/evidence/budgets/receipt lineage retained; fresh replay converges; identical retry is idempotent. |
| Negative control | Stale revision, missing authority, conflicting task outcomes, incomplete mapping or unresolved receipts refuse without mutation; no fabricated completion or start. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/IDR-002/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## IDR-003: Task continuation after identity repair

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-194-change-identity-repair/3 |
| Production entry point | bash <payload>/skills/kbd-apply/kbd-apply.sh begin-task <change> <backend-task>; end-task; reconcile <phase> --repair --json |
| Real collaborators | built CLI; apply driver; actual backend; signed boundary receipts; filesystem projections |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Actual registered task key is reused through repaired identity, valid receipt lineage permits continuation once and reconcile converges without new duplicate work. |
| Negative control | Missing start or uncertain prior effect remains explicit and blocked; implementation-complete flags cannot manufacture receipts. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/IDR-003/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## Exact proposed local packet

```sh
cargo build --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli
node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issues-194-195-196/adopted-contract.json --prometheus tools/prometheus-cli/target/debug/prometheus --evidence-root .kbd-orchestrator/phases/issues-194-195-196/evidence
node scripts/generate-skill-system-distribution.js --check
```

The integration driver is a future boundary-authored acceptance artifact, not an existing passing command. Missing driver/tools/identity/permissions is BLOCKED. PASS requires actual oracle and negative-control observation; exit 0 alone is insufficient, exit 2 never passes. Preserve failed attempts, consumed budgets and uncertain cleanup. No hosted CI, unit-only acceptance or automatic successor.
