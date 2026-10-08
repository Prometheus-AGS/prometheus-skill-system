# Source disposition: phase-learning-deploy-and-debt

Change `change-ldd-13-source-convergence`, backend task `1`: Complete worktree branch dirty-source and archive disposition inventory.

Captured 2026-10-05T13:04:18.802151+00:00 through 2026-10-05T13:05:58.758478+00:00. This is a source inventory, not a source freeze or certification. Editorial dispositions were completed at 2026-10-05T13:12:35.593263+00:00.

Actual implementation route: OpenAI / gpt-6.1-sol, xhigh, Codex desktop native fresh-context collaboration worker. The lead owns all KBD transitions and source imports. No builds, tests, reviewers, model inference, commits, fetches, checkout cleanup, version/tag changes or user-home/cache mutations were performed. Remote identities use read-only `git ls-remote`; PR states use read-only GitHub PR queries and no workflow information.

The exhaustive machine record is [evidence/execute-source-inventory.json](evidence/execute-source-inventory.json). Every registered worktree, changed tracked/untracked path, local branch/tag/archive/stash ref, local-only commit and full/mini gitlink has an explicit disposition and reason. The capture script is [evidence/execute-source-inventory.provenance.py](evidence/execute-source-inventory.provenance.py); its exact SHA-256 and starting-pointer hashes are in the JSON. `disposition_postprocessing` documents subsequent owner-informed refinements.

Local-only means not reachable from the currently cached remote refs; it does not prove a commit has never been published. Squashed/rewritten history and old preserved tags retain distinct IDs. Ancestor checks justify `already-merged`; preserved divergent archive refs are `superseded` for automatic source restoration, with their exact source identities retained. Dirty files always have their own disposition separate from their containing branch.

## Selected current source and PR disposition

| Item | Exact identity | Disposition and ownership |
|---|---|---|
| Full selected baseline | `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61` | Already merged; immutable owner-selected pin. Main checkout remains older and dirty; preserve it. |
| Full #157 | `c80baa03887e2e110a9d825de5092d3e65568372` | MERGED at 12:23:19Z; hook/source-doctor lineage `72e0d184150e7f38f35bfaa2168bbaffa313298b` is already in selected baseline. |
| Full #158 | `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61` | MERGED at 12:25:24Z; task schema lineage `34beecdfbda23748cdb4d423e2401aa270ebce3f` is already in selected baseline. |
| Mini #41 | `62dc8243f34008e3e73dab48c910e418af7aef12` | OPEN; include acceptance-schema parity locally. Lead fast-forwarded ldd-mini-final to this existing commit; user owns PR merge. |
| Mini hook branch | `0e80cd5ca883d24c309b67a58ff49b2f2813a0dd` | Include only `scripts/hook-entry.mjs`, `lib/doctor/plugin-source.mjs`, `lib/doctor/registry.mjs`; prior-session checkout preserved. README/site/version/generated/compatibility-test changes deferred to their owning tasks. |
| Surreal-memory baseline | `0cf8c7e19b52a0f0b6f8fad0f92448bc8f3561d1` | Exact current remote baseline; reconcile selected local source in ldd-memory-final. |
| Surreal-memory local skill | `704528b03a06566dc7c84357a1deade816dcd232` | Include `skills/surreal-memory/SKILL.md` as an explicit patch; local main is one ahead/nine behind remote, not pin authority. |
| Surreal-memory dirty source | `src/main.rs`, `src/lib.rs`, `src/hook.rs` | Include production hook/statusline client code; preserve original whole-file bytes and exclude newly uncommitted unit-test additions. |
| Companion source | `773aa5e33b2217135bc423e526c497cceb0e7058` | Include intended sync/control/workspace and migrated source for isolated change-14 capture; no configured remote. Draft `docs/ui/` roadmap excluded. |

Full and surreal-memory had zero open PRs at the recorded query time; mini had exactly open #41. This does not settle future arriving work or authorise user merges.

## All full and mini registered worktrees

| Repository and absolute checkout | HEAD | Dirty/untracked files | Disposition |
|---|---|---:|---|
| full: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack` | `f35fe2ad33ab96706e1bf83f3937bbdc7bd3d837` | 601 | already-merged; separate dirty-file dispositions govern |
| full: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.claude/worktrees/elastic-moore-46d8e7` | `c308f9762635076b3e98ce0e68224cddfa8f6e84` | 4 | already-merged; separate dirty-file dispositions govern |
| full: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.claude/worktrees/goofy-ritchie-6e4a6c` | `dc56eb81b24e692552eb3a6e4e91ddef05f1cf70` | 4 | already-merged; separate dirty-file dispositions govern |
| full: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.claude/worktrees/optimistic-morse-06f4c1` | `dc56eb81b24e692552eb3a6e4e91ddef05f1cf70` | 5 | already-merged; separate dirty-file dispositions govern |
| full: `/Users/gqadonis/Projects/prometheus/worktrees/deploy-main` | `f35fe2ad33ab96706e1bf83f3937bbdc7bd3d837` | not inspected | out-of-scope; registered metadata only, no checkout access |
| full: `/Users/gqadonis/Projects/prometheus/worktrees/hook-activation-closure` | `72e0d184150e7f38f35bfaa2168bbaffa313298b` | 1 | already-merged; separate dirty-file dispositions govern |
| full: `/Users/gqadonis/Projects/prometheus/worktrees/ldd-machine-main` | `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61` | 0 | already-merged; active lead deployment ownership |
| full: `/Users/gqadonis/Projects/prometheus/worktrees/learning-deploy-and-debt` | `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61` | 113 | already-merged; separate dirty-file dispositions govern |
| full: `/Users/gqadonis/Projects/prometheus/worktrees/uar-task-acceptance-full` | `34beecdfbda23748cdb4d423e2401aa270ebce3f` | 0 | already-merged; separate dirty-file dispositions govern |
| mini: `/Users/gqadonis/Projects/prometheus/prometheus-skills-mini` | `6044ecdeb5c8c957646fef422d886a4568212888` | 68 | already-merged; separate dirty-file dispositions govern |
| mini: `/Users/gqadonis/Projects/prometheus/worktrees/ldd-mini-final` | `62dc8243f34008e3e73dab48c910e418af7aef12` | 0 | include |
| mini: `/Users/gqadonis/Projects/prometheus/worktrees/mini-hook-activation` | `0e80cd5ca883d24c309b67a58ff49b2f2813a0dd` | 1 | include; other prior session, preserve |
| mini: `/Users/gqadonis/Projects/prometheus/worktrees/uar-task-acceptance-mini` | `62dc8243f34008e3e73dab48c910e418af7aef12` | 0 | include |

Older full worktrees have zero unique commits versus current origin/main. Their dirty data consists of learning/bookkeeping or the preserved dependency-link artifact, not selected unique product work. The protected deploy-main entry and deploy/main ref were read only through the owning repository’s registered metadata; no status, diff, traversal, hashing or checkout command ran there.

Original full main has 601 file-level status entries, predominantly phase/learning artifacts plus the memory gitlink and a derived Python bytecode file. Original mini main has 68 file-level entries: phase/learning data, browser/history/memory artifacts and historical cadence specs/receipts. All remain in place. Status totals use `--untracked-files=all`, so they exceed earlier directory-collapsed counts.

## Dependency gitlinks and exact remote source

| Consumer | Gitlink | Committed pin | Observed remote main | Disposition |
|---|---|---|---|---|
| full | `skills/imported/artifact-refiner` | `18d000f3e46952418f682159b227dbf71ce61dcb` | `18d000f3e46952418f682159b227dbf71ce61dcb` | already-merged |
| full | `skills/imported/prometheus-entity-management` | `d1588d8c93cc267774d1911e8bd53bd3918a7f12` | `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565` | include |
| full | `skills/imported/sycophancy-correction` | `bc348fffb4b622c142c1082b281dd2f44b74d37b` | `bc348fffb4b622c142c1082b281dd2f44b74d37b` | already-merged |
| full | `tools/cowork-skills` | `77edcf8a49237dc9b8f6733e794c48fa4e6da832` | `77edcf8a49237dc9b8f6733e794c48fa4e6da832` | already-merged |
| full | `tools/disk-space-guardian` | `26487db64d5c44b00b57d19a649a82632dfe94d3` | `26487db64d5c44b00b57d19a649a82632dfe94d3` | already-merged |
| full | `tools/liter-llm` | `12a2fae9675e34b88e9373caa2bca9959f416493` | `12a2fae9675e34b88e9373caa2bca9959f416493` | already-merged |
| full | `tools/openai-proxy` | `7833663d3b46f7467f2017f2cce392c09ec1b7ac` | `ad32f5dc1f676dc1b40b01de8a4f043a6ee6928d` | include |
| full | `tools/prometheus-knowledge` | `4b3a6744a4211512ff88a1cb96cf1e2a7db4c3ef` | `4b3a6744a4211512ff88a1cb96cf1e2a7db4c3ef` | already-merged |
| full | `tools/surreal-memory-server` | `0af8ae1f3c7486bf7128beff8ef83aaf1a832ea6` | `0cf8c7e19b52a0f0b6f8fad0f92448bc8f3561d1` | include |
| mini | `tools/liter-llm` | `12a2fae9675e34b88e9373caa2bca9959f416493` | `12a2fae9675e34b88e9373caa2bca9959f416493` | already-merged |
| mini | `tools/prometheus-knowledge` | `4b3a6744a4211512ff88a1cb96cf1e2a7db4c3ef` | `4b3a6744a4211512ff88a1cb96cf1e2a7db4c3ef` | already-merged |
| mini | `tools/surreal-memory-server` | `0af8ae1f3c7486bf7128beff8ef83aaf1a832ea6` | `0cf8c7e19b52a0f0b6f8fad0f92448bc8f3561d1` | include |

All nine full and three mini gitlinks were resolved, including imported skill packs. The JSON records actual initialized checkout HEADs independently of committed pins. Newer openai-proxy `ad32f5dc1f676dc1b40b01de8a4f043a6ee6928d`, entity-management `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565` and surreal-memory `0cf8c7e19b52a0f0b6f8fad0f92448bc8f3561d1` require compatible-source disposition and approved final pin decisions; observing newer source does not authorise repinning. Recheck every origin at the change-03 freeze.

The independent knowledge checkout is older `1bbaecc61391257e4bb69d67d5c3716aa8dece4d` with no unique commits, while the committed gitlink and exact remote source are `4b3a6744a4211512ff88a1cb96cf1e2a7db4c3ef`; its historical dirty learning data is preserved. Every initialized dependency repository’s refs/worktrees/local-only identities are in the machine record. Legacy liter-llm tagged/rewrite history is retained as unselected historical lineage, not adopted over its observed remote main.

## Relevant consumer source and Companion

UAR registered metadata lists 45 worktrees. Only `uar-skill-deployment-catalog` at `4e0d69378e44dbeafc0da58937db9c161723824c` and `uar-c14-teams-work` at `d8896d743cd945d40f918ff8ca397909f6c1fe22` were inspected for pack contracts. Direct task-acceptance/strict oneOf schemas, host-envelope acceptance, scalar skill-tag compatibility and deployment/model catalog work are consumer provenance. Whole UAR application/convergence branches and its other 43 checkouts are out of scope. Catalog commit labels are historical source claims; no current pricing or inference was verified. The C14 dirty files are 14 generated sidecar payload paths, preserved and excluded from source import.

Companion has no remote, one registered original checkout and two bootstrap local commit identities. Its 191 file-level dirty/untracked entries include relocated sovereign-sync/client, kbd-mobile and iroh adapter crates; supervisor/headless/identity/pairing source; service installers/manifests; sync skills; tray/brand assets; tests; historical phase data; and draft UI docs. Migrated kbd-mobile source is retained as migration provenance, with no native/mobile execution claim. Change 14 owns precise source-only selection, reviewable capture and private publication after local gates. Protected `versions.toml` is inventoried by hash, never edited; its application needs explicit approval. Local agent settings remain owner data.

The root Companion Cargo manifest contains four skill-system git-rev consumers at `cfbc262b47ce794cb2ea845c64dc33e70b10e890`; change 14 must align the isolated source candidate to the owner-selected exact `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61`. This is distinct from memory release pins.

## Dirty source hash provenance

| Source checkout | Path | SHA-256 | Selection |
|---|---|---|---|
| `/Users/gqadonis/Projects/prometheus/surreal-memory-server` | `src/lib.rs` | `453bae1bb44c73272dcca73d0990836bd6141aaf261709e4f2c4328d46fad4cf` | Include production; preserve/exclude new unit hunks |
| `/Users/gqadonis/Projects/prometheus/surreal-memory-server` | `src/main.rs` | `8f8141672a3d2860ae3fb07e6555e70bf329938abbb6ea3dc30d4c14fb91de33` | Include production; preserve/exclude new unit hunks |
| `/Users/gqadonis/Projects/prometheus/surreal-memory-server` | `src/hook.rs` | `3240321264a6bd5148ff412ce04081e40a54b15b1047f6e4f1e963419cf3e9e2` | Include production; preserve/exclude new unit hunks |

The JSON contains SHA-256, length, file mode, mtime and per-read stability for every enumerated relevant changed tracked/untracked regular production file. Gitlinks are directory source identities rather than file bytes. Sensitive/local configuration content was not emitted; no credentials or environment values were captured. Test, generated, bookkeeping and draft paths have explicit preserve/exclude dispositions. Source may continue moving after capture; the lead must compare hashes before copying and record final candidate identities.

## Full and mini preserved refs

These tables enumerate every observed local head/tag/archive/stash ref, including its source identity and explicit grouped disposition. Full commit detail and local-only ref provenance are in the JSON; no archive or stash was applied.

| Repository | Ref | Commit identity | Local-only IDs count | Disposition |
|---|---|---|---:|---|
| full | `refs/heads/claude/goofy-ritchie-6e4a6c` | `dc56eb81b24e692552eb3a6e4e91ddef05f1cf70` | 0 | already-merged |
| full | `refs/heads/claude/optimistic-morse-06f4c1` | `dc56eb81b24e692552eb3a6e4e91ddef05f1cf70` | 0 | already-merged |
| full | `refs/heads/codex/ldd-machine-main` | `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61` | 0 | already-merged |
| full | `refs/heads/codex/learning-deploy-and-debt` | `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61` | 0 | already-merged |
| full | `refs/heads/codex/uar-task-acceptance-schema` | `34beecdfbda23748cdb4d423e2401aa270ebce3f` | 0 | already-merged |
| full | `refs/heads/deploy/main` | `f35fe2ad33ab96706e1bf83f3937bbdc7bd3d837` | not inspected | out-of-scope |
| full | `refs/heads/fix/hook-activation-closure` | `72e0d184150e7f38f35bfaa2168bbaffa313298b` | 0 | already-merged |
| full | `refs/heads/fix/prometheus-exec-path-independent-build` | `c308f9762635076b3e98ce0e68224cddfa8f6e84` | 0 | already-merged |
| full | `refs/heads/main` | `f35fe2ad33ab96706e1bf83f3937bbdc7bd3d837` | 0 | already-merged |
| full | `refs/stash` | `7fc61e06f7169b8eebd774441ab3fcac4160bd3f` | 3 | out-of-scope |
| full | `refs/tags/archive/20261005/automation/docs-sync` | `52f2eaa0538e56cbc1a905606e24545018b06e23` | 1 | superseded |
| full | `refs/tags/archive/20261005/backup/skill-pack-1-10-pre-push-rewrite` | `45b61a2bef130a3e98f0b917b0159ab4be08ce55` | 12 | superseded |
| full | `refs/tags/archive/20261005/codex/agent-fabric-convergence` | `49899d3d109d6c43b6670a39b3b2aff05fbf475c` | 1 | superseded |
| full | `refs/tags/archive/20261005/codex/agent-fabric-convergence-local` | `497db1cb1ff66f103dfa20d97b07184b0d5b2a0d` | 2 | superseded |
| full | `refs/tags/archive/20261005/codex/delivery-cadence-pipeline` | `f87e5320c8a5c8311ccb8bd76e759779f8370449` | 1 | superseded |
| full | `refs/tags/archive/20261005/codex/delivery-cadence-recovery` | `9d83f0d108719a8926b5473f0f84ed1e7d7a3a47` | 1 | superseded |
| full | `refs/tags/archive/20261005/codex/hma-adjacent-plugin` | `814d4763fd14e24183631c407982ffa80b9eb6ac` | 1 | superseded |
| full | `refs/tags/archive/20261005/codex/liter-llm-2.1-pin-pack` | `10d1851e6e6cbaa3f47cc5dc58a9d679e4071df7` | 2 | superseded |
| full | `refs/tags/archive/20261005/codex/recover-qualified-task-subjects` | `39e05212e4f295652dadc28ad3abe17f31aa41a6` | 1 | superseded |
| full | `refs/tags/archive/20261005/feat/cpc-001-002-integration-contract` | `35d2f7b304fd3544326914d9493ce74932a2c96f` | 7 | superseded |
| full | `refs/tags/archive/20261005/gofast/liter-llm-mcp-secrets` | `2b1ce32c04317aecfbe235e559a0fccee120c964` | 1 | superseded |
| full | `refs/tags/archive/20261005/win-001-host-portable-activation` | `108dd562febb69913f938ab962febc6ad59a1aba` | 2 | superseded |
| full | `refs/tags/kbd/canonical-lifecycle/reflected` | `b3e7d75e8410fef4abdc6e8ea9d30df336aa2cc8` | 0 | already-merged |
| full | `refs/tags/kbd/position-and-handoff-guarantee/reflected` | `a76416eb9abbcab147a7e0d23c6ee0182fc0f541` | 0 | already-merged |
| full | `refs/tags/kbd/safeguards/reflected` | `bb490dec0e0e5581859f10d7547b9d55fb1271f6` | 0 | already-merged |
| full | `refs/tags/pruned/archive-prometheus-main-convergence-20260823T175006Z-parent-feature-tip` | `a0027fff86322a2bd6b49074457f4e4e512fe179` | 17 | superseded |
| full | `refs/tags/pruned/automation-docs-sync` | `4043e8b948064572ea950c5b03c007635b38b1ae` | 1 | superseded |
| full | `refs/tags/pruned/chore-kbd-phase-ci-all-green` | `a14a022bd103213b9863b0dd1b7c4c8e4a40ebf2` | 1 | superseded |
| full | `refs/tags/pruned/ci-green-bdd` | `e027b25941a50aca305a5ed502524599657cde72` | 2 | superseded |
| full | `refs/tags/pruned/ci-green-formatting-forge` | `5cf03c9fc71f7fce1754008f7e50b94ec6ed3d5f` | 3 | superseded |
| full | `refs/tags/pruned/ci-hardening-bearer-auth` | `bbef4e91694c71bc5536d9ee4ebfd7a4088e428b` | 1 | superseded |
| full | `refs/tags/pruned/ci-hardening-toolchain-crossqa` | `b3d6ad28d3526afc5f832b786beb0ef09601754b` | 3 | superseded |
| full | `refs/tags/pruned/claude-festive-banach-45ff2f` | `f3078bbc23f3904ecd9503db0eb82edccf69d0f8` | 1 | superseded |
| full | `refs/tags/pruned/claude-hungry-mcclintock-b74cd3` | `0895057f1de9fe23ef2ff62cb433ff713beea4fa` | 1 | superseded |
| full | `refs/tags/pruned/claude-recursing-liskov-28f391` | `8999b59e360608a040d33c73a71acd00145b177e` | 1 | superseded |
| full | `refs/tags/pruned/codex-1.7.0-version-docs-hotfix` | `15c56860b04b102d582f28ce85b3114031303ae9` | 1 | superseded |
| full | `refs/tags/pruned/codex-align-devin-install-target` | `c387411846ab61acf944ea41a3cabf97f8697ce3` | 0 | superseded |
| full | `refs/tags/pruned/codex-all-tools-reinstall-20260823` | `9ccad59832ff80423bc38869ad4a849addd7877d` | 0 | superseded |
| full | `refs/tags/pruned/codex-base-rules-v3` | `76043ef4b386511091bf5e4acfebce69e6ba5ebc` | 1 | superseded |
| full | `refs/tags/pruned/codex-docs-doctor-release` | `fe0175c490e575e146df7bd57fd01967bb9850db` | 0 | superseded |
| full | `refs/tags/pruned/codex-dynamic-operations-docs` | `4bbecc468004fd4335f71640ff6cb1dd8b930e75` | 1 | superseded |
| full | `refs/tags/pruned/codex-final-main-convergence` | `c2d0753205094f771bf1ff6fe5fa2e65d7478fa3` | 0 | superseded |
| full | `refs/tags/pruned/codex-git-installable-skill-system-1-7-0` | `e2c97c6c27d5dc2dd0fbe8eb0f43fca9af8cd0f0` | 1 | superseded |
| full | `refs/tags/pruned/codex-karpathy-recovery` | `7e83779bd583b6663b132c7f16c4cf84188b36e8` | 0 | superseded |
| full | `refs/tags/pruned/codex-kbd-docs-release` | `4800e78e748bdd48f9bde78a66e2a7718c40bd5c` | 4 | superseded |
| full | `refs/tags/pruned/codex-kbd-integrity-release` | `e188e3f7c93cae46e213653df03c103dd1768493` | 0 | superseded |
| full | `refs/tags/pruned/codex-kbd-migration-focus-repair` | `a8722682bfecba0af35b51a963cd49c703b1ac0a` | 0 | superseded |
| full | `refs/tags/pruned/codex-kbd-runtime-release` | `5387ef63abe487603ff6ee4bc5abca6b9382f594` | 1 | superseded |
| full | `refs/tags/pruned/codex-local-only-release-controls` | `2bf70450b433ee0ab8f72763d602c673179c7f45` | 2 | superseded |
| full | `refs/tags/pruned/codex-local-release-reconciliation` | `4862ef5382c4ff1fc2438102b986ca2ecd79091c` | 0 | superseded |
| full | `refs/tags/pruned/codex-main-convergence-20260823` | `f90ee200ddd6101ac767c6b0c5a227c12bbe1445` | 0 | superseded |
| full | `refs/tags/pruned/codex-pin-entity-main-552e57c` | `0f00b8f7a8d88ceeb1fa4a71114928b010a5637a` | 0 | superseded |
| full | `refs/tags/pruned/codex-pin-entity-main-ba8ab44` | `36a6896cf79ca2b999b21e4b7f6ac06b1a2dc573` | 0 | superseded |
| full | `refs/tags/pruned/codex-pin-knowledge-main-5a175d1` | `e072cce3098d85e6c7e9a47af3d8a5be9b2c008a` | 0 | superseded |
| full | `refs/tags/pruned/codex-regenerate-harness-install` | `4c458b029ac97c09dd12f61327871e1fb88aea19` | 0 | superseded |
| full | `refs/tags/pruned/fix-preflight-resolver-lookup` | `9b328adc24d374b47373db1d6f62d73ed6a13bd9` | 1 | superseded |
| full | `refs/tags/pruned/fix-remove-kbd-mutation-fence` | `00a3a52bd395d70bcda9e514899dcab45710a765` | 1 | superseded |
| full | `refs/tags/pruned/fix-surreal-memory-native-config` | `d428e2a6636f34d0295d666524725f5190c436ad` | 0 | superseded |
| full | `refs/tags/pruned/kbd-openspec-mirror-drift-cleanup` | `48f4c62e394592210fe320b2f0c4212b1df450ec` | 20 | superseded |
| full | `refs/tags/v1.1.0` | `adf969fce3cb0573fe201049cdaa0bd510cb6f3b` | 0 | already-merged |
| full | `refs/tags/v1.10.0` | `d33339e94334235069b9c62e18e651a4a5c7daec` | 0 | already-merged |
| full | `refs/tags/v1.2.0` | `f80adf2c028bbf8934b451e67f39fea802ecc753` | 0 | already-merged |
| full | `refs/tags/v1.6.0` | `c345c9ca37558685b30b3b7aa95956a3fd115b18` | 0 | already-merged |
| full | `refs/tags/v1.7.0` | `5855f1d62b908785695443215d19d3f82c557162` | 0 | already-merged |
| full | `refs/tags/v1.8.0` | `1dc5a670fa0659a41fc9cedea2020db19e4a59a3` | 0 | already-merged |
| full | `refs/tags/wip/pem-302-repin` | `eba2f40565d653f018e1a8c02d319ddf768b1057` | 0 | already-merged |
| full | `refs/tags/wip/preserve-final-main-convergence` | `c2d0753205094f771bf1ff6fe5fa2e65d7478fa3` | 0 | already-merged |
| full | `refs/tags/wip/preserve-kbd-drift-cleanup` | `48f4c62e394592210fe320b2f0c4212b1df450ec` | 20 | superseded |
| mini | `refs/heads/codex/ldd-mini-final` | `62dc8243f34008e3e73dab48c910e418af7aef12` | 0 | include |
| mini | `refs/heads/codex/uar-task-acceptance-schema` | `62dc8243f34008e3e73dab48c910e418af7aef12` | 0 | include |
| mini | `refs/heads/fix/hook-activation-closure` | `0e80cd5ca883d24c309b67a58ff49b2f2813a0dd` | 1 | include |
| mini | `refs/heads/main` | `6044ecdeb5c8c957646fef422d886a4568212888` | 0 | already-merged |
| mini | `refs/tags/archive/20261005/codex/afc-c03-team-authoring` | `5d8e85d8d0ec3d2ad75e38922943f772e8603ce9` | 23 | superseded |
| mini | `refs/tags/archive/20261005/codex/agent-fabric-convergence` | `d0bd4f9e41b7edf7383874d4bbcf2c050ad840c3` | 2 | superseded |
| mini | `refs/tags/archive/20261005/codex/delivery-cadence-recovery` | `087c71b6fb7c59403c985eebb03e37b7d0fb91c3` | 1 | superseded |
| mini | `refs/tags/archive/20261005/codex/hma-adjacent-plugin` | `31fb2ce024d94aa76ed3b59a581b34064b235c4f` | 1 | superseded |
| mini | `refs/tags/archive/20261005/codex/liter-llm-2.1-pin-mini` | `a3089b4ee2151993b5cc76c7d7d95a7d7229fd82` | 2 | superseded |

## Owner/activity and remaining source checkpoints

- Active phase worktrees and ldd-machine-main are lead-owned. Mini hook checkout belongs to another prior session. Dirty original full/mini/memory/Companion checkouts retain their author ownership; no cleanliness/mtime observation proves an owner is idle.
- Parent confirmed the sole installed surreal-memory team is `memory-core` under `.agent-team/memory-core`; retain its selection. Local routing/stash changes are preserved outside product-source recovery.
- Source freeze still requires hash comparison and provenance for moving owner source, scoped mini hook source import, production-only memory import, exact compatible dependency pin disposition and Companion source capture. The broad local surreal-memory dependency-update branch `dd7fdcd6d8974af4059d1d51401bd33ae29f65db` is explicitly out of selected hook/skill recovery scope and preserved; any later compatibility need must get its own source decision.
- Mini #41 remains user-merge owned. No protected versions/tags, installed-cache, user-home or source-freeze approval is inferred. No archive/worktree deletion is authorised by this inventory.
- Parent reported another session running `cargo check --workspace` PID 22976. This inventory ran no builds; that report is activity provenance, not independently verified process status or accepted validation.
- Existing historical canonical task flags/receipts remain unchanged. This artifact completes inventory/disposition work only; the lead decides task transitions. Acceptance and certification remain under the completed-production change-12 boundary.
