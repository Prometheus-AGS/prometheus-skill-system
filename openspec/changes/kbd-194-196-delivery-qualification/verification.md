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

## SCN-001: Calibrated bounded scenario discovery

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-196-value-scenario-workflow/1 |
| Production entry point | /value-scenario-discovery <phase-contract-ref> (new procedure); Assess/Spec/Plan routing |
| Real collaborators | new packaged discovery procedure; existing phase stages; owner-reviewed calibration references; selected native local session |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Finite proposals retain all relevance fields; reviewed ordinary useful journey and rare mandatory integrity case are selected, irrelevant available case deferred with reason; no invented likelihood or scope adoption. |
| Negative control | Unreviewed calibration or changed-model unfamiliar candidate remains proposed/UNKNOWN; optional available benchmark does not silently become blocking scope. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/SCN-001/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## SCN-002: JS and Rust author-only without execution

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-196-value-scenario-workflow/2 |
| Production entry point | bdd-cucumber-js author-only <contract-ref> <IDs>; bdd-cucumber-rs author-only <contract-ref> <IDs> (explicit skill routes to add) |
| Real collaborators | existing packaged BDD adapters; actual local authoring session; filesystem; command observation; approved scenario contract |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Approved scenario maps to feature/assertion/step/config/oracle source and excluded false positive; actual command observations show zero checks, support imports, installs or protected-file writes. |
| Negative control | Unknown ID or disallowed authoring stage produces explicit refusal; dry-run is not permitted as author-only. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/SCN-002/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## VER-001: Real useful operation and oracle sensitivity

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-196-value-scenario-workflow/3 |
| Production entry point | /value-scenario-verify <contract-ref> (new procedure), existing runner executes actual scratch CLI repair-and-resume operation |
| Real collaborators | existing JS/Rust runner; built production CLI; actual journal/backend; assertions; negative control |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Current identity-bound real repair/resume operation produces intended observable state; safe incorrect candidate/assertion challenge cannot pass; required assertion actually observes the intended result. |
| Negative control | Zero exit with absent/disabled/always-true assertion cannot pass; a disconnected return-false probe is not a sensitivity challenge. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/VER-001/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## VER-002: Honest evidence/environment dispositions and lineage

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-196-value-scenario-workflow/3 |
| Production entry point | value-scenario-verify through actual selected runner and existing evidence/closure adapters |
| Real collaborators | actual runner subprocess; fixtures/configurations; canonical evidence adapter; owned resources; cleanup records |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Finite cases distinguish stale/wrong candidate/configuration/oracle, missing runner/permission, unexercised assertion, failed cleanup and uncertain effect; original failure/consumption and targeted repair lineage retained. |
| Negative control | Only exercised product mismatch is FAIL; missing prerequisite/stale evidence/cleanup failure is BLOCKED and unobserved obligation NOT_EXERCISED; no retry after unknown effects. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/VER-002/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## VER-003: Canonical closure and no automatic work

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-196-value-scenario-workflow/3 |
| Production entry point | existing prometheus kbd phase closure/evidence writer via value-scenario-verify |
| Real collaborators | existing typed closure; signed store; real observations; receipt artifacts |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Reports authored implementation, measured acceptance, demonstrated runtime value, gaps, release eligibility and release authority distinctly; supported canonical closure persists without new phase/task/publication. |
| Negative control | Unsupported cancelled/deferred closure remains refused; pending review/unmet evidence never passes or triggers automatic successor. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/VER-003/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## PKG-001: Finite source/generated/scratch-installed parity

| Field | Required declaration |
|---|---|
| Canonical identity | issues-194-195-196 / kbd-194-196-delivery-qualification/2 |
| Production entry point | repository generator and installer dispatch; built CLI, source and generated Claude/Codex plus scratch-installed payload production entries |
| Real collaborators | repository generators; actual installer dispatch; generated Claude/Codex payloads; scratch installed skills; selected native local sessions |
| Boundary exercised | Actual subprocess/filesystem/signed journal and backend boundaries; native-session/installer/runner boundaries where specified; no substitute mocks. |
| Observable result | Same adopted finite corpus retains scenario/authority/diagnostic semantics at selected entry points; actual paths/versions/content identities recorded; two generator outputs byte-identical; actual installer dispatch used. |
| Negative control | Changed payload/reference, unsupported host or absent required native session cannot be certified by matching labels/bytes or helper-only tests. |
| Isolation and fixture identity | Owned mkdtemp project, child HOME/CODEX_HOME/XDG/config/data/key/hook/memory roots and isolated services/ports; fixture manifest records exact bytes and SHA-256 before execution. No real user stores or credentials. Fixture identity pending boundary freeze; missing manifest is BLOCKED. |
| Prerequisites | Complete parent production; current adopted contract and approved IDs/calibration; available actual local tools/session/permissions; exact fixture/candidate identities; final independent review for certification. |
| Final local gate | From repository root, invoke the ordered boundaryPacket.commands in the phase contract; no execution during Spec. |
| Evidence | evidence/PKG-001/: argv/environment allowlist, candidate/build/config/fixture/scenario/assertion hashes, source revisions, stdout/stderr, original observations, all material attempts, canonical receipt lineage and cleanup. Pending; no execution claimed. |
| Limitations | Original #194 incident, other hosts and Windows remain unqualified. Instruction-only exports do not prove enforcement; missing required native observation blocks. |

## Exact proposed local packet

```sh
cargo build --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli
node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issues-194-195-196/adopted-contract.json --prometheus tools/prometheus-cli/target/debug/prometheus --evidence-root .kbd-orchestrator/phases/issues-194-195-196/evidence
node scripts/generate-skill-system-distribution.js --check
```

The integration driver is a future boundary-authored acceptance artifact, not an existing passing command. Missing driver/tools/identity/permissions is BLOCKED. PASS requires actual oracle and negative-control observation; exit 0 alone is insufficient, exit 2 never passes. Preserve failed attempts, consumed budgets and uncertain cleanup. No hosted CI, unit-only acceptance or automatic successor.
