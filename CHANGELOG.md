# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Next] - not yet released

- A new KBD phase can opt in to outcome-first scope with `/kbd-new-phase <name> --outcome-first --outcome "<text>"`. Resume in that phase selects eligible pending work ahead of blocked work, honours `change register --depends-on`, and the waypoint, position reminder and progress ledger show the outcome, the last evidence and the next authorized action. Existing phases are unchanged (issue #256, first of five parts).
- `bootstrap.sh --audit-conflicts` lists, without changing anything, rule-file guidance that contradicts the VLBV boundary rule across root and nested `AGENTS.md`/`CLAUDE.md`, imports, `.claude/rules` and other worktrees, and reports whether the installed generation verifies. Leftover guidance that described per-change QA, per-iteration triggers, several checkpoints per phase or the v3 tier ladder now states the single boundary run (issue #256, second of five parts).
- `prometheus kbd guard advise` reads canonical state and proposes one next step (`setup-repair`, `measurement-repair`, `authority-handoff` or `architecture-child`) when the same blocker was attempted again without new evidence. It appends no event, stops nothing and creates nothing; waits on an approval or a resource are never counted, and a running gate answers `in-flight: do not stop`. `blocker record` takes optional `--fingerprint`, `--class` and `--evidence` (issue #256, third of five parts).
- The closure packet shows optional evidence references from `boundary-results.json`: the candidate a result belongs to, the venue, and per-scenario scope, fixture and oracle references. A result recorded for an earlier candidate is labelled stale and counts as NOT_EXERCISED for the current one. The label changes no verdict and no disposition (issue #256, fourth of five parts).
- During Execute an agent makes one bounded project-memory lookup when a task starts, on resume and on a material blocker: `KBD_RECALL_QUERY` asks `kbd-memory-recall` about the work in hand and writes `task-recall.md`, leaving `prior-context.md` as it is. The lookup also resolves child phases. Recalled lessons are evidence to check and never authorize a check, widen scope or replace an approval; with no source available the agent proceeds. `learning_write.py` refuses credential-shaped text at `user` and `global` visibility. Four opt-in qualification cases are in `evals/vlbv-protocol/cases/kbd-memory-recall.json` (issue #256, fifth of five parts).

## [1.18.0] - 2026-11-24

- Two KBD sessions on one machine can exchange signed work updates, resume from a saved position and accept task handoffs through the same project authority.
- The CLI reports which authority answered a read. The installer and doctor use the selected profile and discovery harnesses when checking installation health.
- Restored missing OpenSpec deltas and refreshed the Claude Code and Codex skill packages.

## [1.17.0] - 2026-11-23

- Installed KBD lifecycle hooks record phase and reflection events, and signed operator exits can close eligible phases.
- Apple Silicon builds of `prometheus-exec` select the certified macOS SDK and linker to reproduce the pinned binary.
- The doctor checks the enabled marketplace plugin and distinguishes standalone KBD projects from configured control endpoints and rollouts.
- Skill discovery measurements now cover Claude Code, Codex, OpenCode and Kimi. Codex verification checks the skills selected for its installed catalog.

## [1.16.2] - 2026-11-20

- Extend Context Bootstrap with read-only audit and preview, and explicit digest-bound apply for the two resident instruction files of an existing Git checkout. Preserve prior instructions, routing and the legacy scaffold flow.

## [1.16.1] - 2026-11-19

- Bind Bootstrap and KBD acceptance completion to delivered user value, relevant VLBV evidence, guidance/access, honest limits and reflection; distinguish review-ready from done. Existing gates unchanged.

## [1.16.0] - 2026-11-19

- Make user goals/value and plain-language business rules mandatory in Context Bootstrap, KBD selection/planning and VLBV. Trace accepted scenarios to reachable outcomes, controls and evidence; preserve existing phase gates and mature release processes.
- Update source templates and generated distributions through the existing generators. No runtime binary or dependency upgrades.

## [Unreleased]

### Changed

- **OpenSpec 2.1.0** is the new exact pin (was 2.0.0), in `package.json`, `config/spec-engines.json` and the managed runner. Node.js 22.20.0 or newer is now required. The runner passes `--scope project` on `init`, because 2.1.0 installs skills globally by default in a new repository. Its session-start check reads `openspec update --dry-run --json`, since 2.1.0 no longer stamps `generatedBy` into its skills. A refresh skips the network when nothing would change, and one that cannot reach GitHub reports `refresh-unavailable` without stopping phase entry. The repository's own `openspec-*` skills are regenerated; `.devin/skills` is new because 2.1.0 writes Devin (formerly Windsurf) its own copies.

### Added

- **VLBV protocol evals** (`evals/vlbv-protocol/`, CR-31): the 22 VLBV acceptance tests and the §18 fresh-session check as eval cases. Each installed harness (Claude Code, Codex, OpenCode, Kimi, Qwen Code) gets one report with pass, fail and not-run counts. Agent cases run 3 samples each in isolated scratch homes and are graded cross-family; machinery cases run once against the real KBD scripts. Uninstalled harnesses are reported as not run.
- **Boundary authorization event** (`shared/scripts/kbd-boundary-authorize.mjs`): records when the owner authorized B4 or B5, before the first attempt.
- **Pre-authorization artifact scan** (CR-33): the closure packet lists runner output under the contract's new `boundaryPacket.runnerOutputGlobs` that is older than the B4 authorization. Report only; it never blocks.

### Changed

- `scripts/check-harness-adapters.js`: every `supported` capability field must carry its own evidence, either a committed file or a protocol-eval case of the same harness whose verdict is PASS and which declares that field. A new `--capabilities <file>` option validates a copy.
- **The `prometheus-exec` build hash is pinned per Rust host target.** `config/prometheus-exec-binary.json` replaces the single `expectedBuildSha256` with `expectedBuildSha256ByTarget`, read through the new `scripts/prometheus-exec-pin.sh`. A manifest with only the old field still works. The installer names the target in its messages and records it in the install receipt. Recertifying now means every pinned target, each on a machine of that target. The `aarch64-apple-darwin` entry has not been re-measured since the build change below; run `bash scripts/build-prometheus-exec.sh --print-hash` on Apple Silicon and update it if it differs.

- **`tools/surreal-memory-server` moves to `dfae309`** (1.11.0 plus bounded memory enumeration, upstream issue #62). **Breaking upstream:** `GET /api/v1/memory` and the `get_all_memories` MCP tool now return one page, `{ memories, next_cursor, has_more }`, instead of a bare array of every memory. A page holds 50 memories by default (200 at most), is ordered by record ID, and omits embeddings unless asked. The pack's lesson recall follows the pages (see Fixed).

### Fixed

- **`skill-certification` 1.0.1**: a `description` written as plain text continued on indented lines is read as one scalar. It was read as a nested mapping, so a valid skill failed level C0 with a false `description` finding (found while certifying the mini pack: `vercel-composition-patterns`, `vercel-react-native-skills`).
- **The liter-llm MCP server starts in a harness launched without the keys in its environment.** Every harness entry ran `liter-llm mcp --transport stdio --config ...` directly. The config reads `master_key = "${LITER_LLM_MASTER_KEY}"`, and the value lives only in `~/.prometheus/kbd/secrets.env`, so a harness started from the Dock, Finder or a stale shell (desktop apps, Zed, MiniMax Code) got a server that exited at once and an MCP client that said only "Connection closed". Entries now run a launcher, `~/.local/bin/liter-llm-mcp` (source: `shared/scripts/liter-llm-mcp-launch.sh`), which sources the secrets file and then execs the server, as `liter-llm-api-launch.sh` already did for the gateway. It prints nothing to stdout before the exec. A missing master key, binary or config is a message on stderr and exit 78, 127 or 66; the first names the secrets file. `liter-llm-mcp --check` runs the checks without starting the server. No secret enters any harness config.
  - **Every writer of the entry uses the launcher:** `scripts/mcp-port-table.json`, `configure-mcp-all-tools.sh`, the Kimi block of `install-skills-flat.sh` and `install-platforms.ts`, `/liter-llm-bridge`'s `configure-mcp.sh register`, the repository's `.codex/config.toml`, and the plugin `.mcp.json` (which cannot hold a machine path, so it runs `/bin/sh -c` and falls back to the copy inside the plugin). The runtime plugin now ships the launcher, and the distribution generator rejects an MCP template that starts liter-llm any other way.
  - **Existing installs are moved by the new `scripts/install-liter-llm-mcp-launcher.mjs`,** which `install-binaries.sh`, `install-skills-flat.sh` and `configure-mcp-all-tools.sh` run. It installs the launcher, then rewrites only an entry that is exactly what the pack used to write (`liter-llm mcp --transport stdio`, with or without `--config` and the default config path, no `env`), keeping a `.bak.<timestamp>` copy. An entry with another binary path, another config path, extra arguments or an `env` block is reported and left alone. `--check` and `--dry-run` write nothing. The step is separate from the mergers because they never rewrite an existing entry, and the cross-tool merger does not run without `TAVILY_API_KEY`.
  - **`scripts/check-model-config.sh` checks the launcher** in an empty environment, the way a Dock-launched harness runs it, and says for each unset `${VAR}` whether the secrets file defines it.
  - A harness the pack does not configure (Zed, for example) needs its liter-llm command pointed at `liter-llm-mcp` by hand.
- **Lesson recall reads a whole scope listing on a paging memory server.** `shared/scripts/lib/learning_recall.py` read one response from `GET /api/v1/memory` and sorted it by time. Against a server that pages (above), that is the first 50 memories by random record ID, so the team digest and the search fallback reported the "most recent" of an arbitrary sample: in the new test, 37 of 50 entries were wrong. The recall now requests pages of 200 and follows `next_cursor` to the end, and still accepts an older server's bare array. A listing cut short by the deadline, a failing later page, a 25-page cap or a cursor that does not advance is named in the result's new `partialListings` field instead of passing as complete. New integration test: `shared/scripts/tests/test-recall-paging.sh`, run against both server generations with `TLI_SM_BIN`.
- **The scratch memory-server test helper starts off Apple Silicon.** `shared/scripts/tests/lib/scratch-surreal.sh` required the MLX embedding executor, which exists only on Apple Silicon, so every test using it was blocked on Intel and Linux. It now uses the `candle` backend on the CPU there (`TLI_SM_EMBEDDING_BACKEND` overrides).
- **Installers read binaries from cargo's real target directory.** `install-binaries.sh` and `install-skills-flat.sh` assumed `<workspace>/target/release`. With `build.target-dir` or `CARGO_TARGET_DIR` set, that path was missing (the installer aborted and left later binaries uninstalled) or held an older build (which was installed as if new). Both now ask cargo through the shared `scripts/cargo-release-dir.sh`.
- **`install-binaries.sh --dry-run` changes nothing.** The dsg and prometheus-research sections built for real, the dsg and cowork download paths fetched release archives, and the research section re-registered its launchd service during a dry run.
- **A liter-llm or dsg lookup can no longer abort the installer.** `find` on a release directory that was never built exits non-zero, which is fatal in an assignment under `set -e`.
- **A liter-llm build failure no longer aborts the installer.** The build was unguarded under `set -e`, so a failure left every later binary uninstalled (the 2026-07-30 incident). It is now recorded, reported at the end with a non-zero exit, and the remaining binaries still install. A binary left in the release directory by an older build is not installed after a failed build.
- **`prometheus-exec` builds reproducibly with `rust-src` installed.** With that toolchain component present (it arrives with rust-analyzer), `rustc` rewrote standard-library paths to the local toolchain directory, so the binary embedded the home directory and its hash changed. `scripts/build-prometheus-exec.sh` now remaps that directory back to `/rustc/<commit>`.
- **`prometheus doctor`, `prometheus setup` and `prometheus memory install` find built binaries through `CARGO_TARGET_DIR`.** They read a hardcoded in-repository path, so on a machine with a redirected target directory doctor reported the hash of a stale binary as the "source" of a newer installed one. The CLI does not read cargo config files; export the variable on such a machine.
- **The liter-llm gateway binds to `127.0.0.1` by default.** liter-llm's own default is `0.0.0.0`, which exposed the gateway to the local network. The launch wrapper passes `--host`; `LITER_LLM_HOST` overrides it. A gateway that was already running keeps its old bind until it is restarted.

### Documentation

- New site pages: [boundary authorization and early-execution evidence](site/docs/kbd/boundary-authorization.md) and [protocol evals and capability evidence](site/docs/kbd/protocol-evals-and-capability-evidence.md).
- The exec installation page documents the per-target pin, what makes the build reproducible and when to recertify. The operations pages document installer dry runs, where build output lives and the doctor's managed-binary check. The guide documents the loopback gateway bind and the boundary scripts.

## [1.15.0] - 2026-11-15

### Changed

- **OpenSpec 2.0.0** is the primary spec engine, pinned exactly (`package.json`,
  `config/spec-engines.json` and the managed runner's new `PINNED_VERSION`; the
  runner minimum is now 2.0.0). The runner no longer follows npm `latest`: a newer
  release is reported as `newerAvailable`, a newer major as `migrationRequired`,
  and adopting either is a pack release or an operator pin
  (`PROMETHEUS_OPENSPEC_VERSION`). OpenSpec skills now live once in
  `.agents/skills/openspec-*`, with per-skill links in `.claude/skills`.
- **Session start verifies, phase entry refreshes.** The SessionStart hook runs the
  runner's new `verify` operation, which never installs or writes; the mutating
  `refresh` runs only at phase entry. The backup footprint also covers legacy
  `~/.codex/prompts/opsx-*.md`.
- **VLBV is the single verification method.** Projects carry
  `verificationMethod: "VLBV"` (the pack default); a per-phase deviation is a
  signed `KBD-METHOD` decision naming the method, its execution cadence and its
  reason, disclosed in the closure packet. No per-phase decision is needed.
  `method-context.mjs` reads the files the active harness actually loads (harness
  profiles; Zed first-match and Codex `AGENTS.override.md` shadowing are reported)
  and a marked `<!-- kbd-policy:begin/end -->` block instead of free-text matching.
- **One boundary taxonomy** (`references/boundaries.md`, B0–B6) and one eight-step
  B4 sequence. Generated code executes only at B4 (the boundary packet, once) or
  B5 (an owner-authorized repair). `kbd-execute`, `value-scenario-verify`,
  `kbd-reflect`, `cross-tool-protocol.md`, the artifact-refiner integration
  (one cumulative pass, no build trigger) and the Rust phase-gated rules now point
  at it. OpenSpec verify and archive run only after the phase is CLOSED.
- **Delivery cadence attaches to the boundary**: in KBD mode `checkpoint` runs only
  boundary-packet entries, `report` derives `canonicalOutcome` from the KBD
  closure, work-ahead needs the successor phase's contract to be promotable, and
  `refresh-skill-pack.sh` refuses (exit 3) while a bound phase is open.
- **Adopted contract v2** (`adopted-contract.v2.schema.json`; v1 contracts stay
  valid until their phase closes): selection rationale, structured oracles with an
  independent truth source, negative controls bound to typed packet commands, real
  and substituted boundaries, limitations, per-scenario invalidation keys and
  restoration, an optional stochastic profile, calibration and method identity
  (with `specDeltaDigest`) and four closure states. `kbd-contract-adopt.sh` adds an
  adoption lint (`--lint-only` runs it alone) and refuses adoption while a CRITICAL
  review finding on acceptance content lacks an owner decision. Readiness returns
  REJECT-STALE when a bound OpenSpec requirement changes. `kbd-phase-close-eval.sh`
  writes `closure.json` and `closure-packet.md`. Templates' field lists are
  generated from the schema (`npm run check:contract-templates`). The scenario ID
  pattern is widened to `^[A-Z][A-Z0-9]*(-[A-Z0-9]+)*-[0-9]+$`.
- **One installed skill copy.** Skills install once to `~/.agents/skills` as real
  directories, with symlink shims only for Claude Code (`~/.claude/skills`) and
  Qwen Code (`~/.qwen/skills`). Install removes only pack-owned entries from the
  12 retired per-harness roots and records them in
  `receipts/retired-target-cleanup.json`. Older signed generations stay verifiable.
- **Harness capability profiles** in `shared/harnesses/capabilities.json`
  (Claude Code, Codex, OpenCode, Kimi, Zed, MiniMax Code, MiMo-Code, Qwen Code). A
  capability is `supported` only with committed evidence of an exercised run;
  `kbd-doctor.sh profile` prints them. Handoff packets carry the receiver's
  profile, a held-out calibration check and unresolved effects. Skill text names
  skills without a harness-specific prefix.
- **Apple on-device Foundation Models**: on an eligible Mac (macOS 26+, Apple
  Silicon, Apple Intelligence on) installation sets up `apfel` as the
  `ai.prometheus.apple-fm` LaunchAgent on `127.0.0.1:11437`, registers
  `apple-foundationmodel` with liter-llm and the KBD `local` role. Silent and
  non-fatal elsewhere; never a judge or critic.
- **prometheus-context-bootstrap** generates VLBV-conformant `AGENTS.md`,
  `CLAUDE.md` and stack rules: the policy block, a first-action protocol that works
  without hooks, precedence over built-in TDD/verify/apply skills, and no compiler
  exception. `verify.sh`/`verify-v4.sh` check this on generated files.
- New `vlbv` entry skill with the hash-pinned method text (VLBV 01–22, AT 01–22,
  checklist, supersession record); the testing plugin (1.6.0) lists all VLBV skills.


- The native SurrealDB service is certified against SurrealDB 4.0.0
  (`SURREALDB_VERSION`), matching surreal-memory-server's `=4.0.0` SDK pin.
  SurrealDB 4.0 has no RocksDB engine, so the launchd and systemd templates
  start the same data directory as `surrealkv:` and drop
  `SURREAL_ROCKSDB_BLOCK_CACHE_SIZE`. They set
  `SURREAL_STARTUP_OPERATION_TIMEOUT=30m`: startup runs SurrealDB's own data
  migrations inside one bounded step, and the 60 s default aborts a 3.x
  upgrade of a real store.
- Upgrading an existing 3.x store is a one-way, opt-in operation. With both
  services stopped, take a `surreal export` of every namespace/database and a
  copy of the data directory, install surreal 4.0.0 to
  `~/.prometheus/bin/surreal`, start it once by hand with
  `SURREAL_EXPERIMENTAL_UPGRADE_FROM_3X=true` on the `surrealkv:` path until
  queries succeed, stop it, then run `scripts/install-mcp-services.sh
  --restart`. A 3.x release cannot open the converted files; restore from the
  copy or the export to go back.

### Removed

- **Breaking:** the pack no longer emits OpenSpec `/opsx:*` commands (removed by
  OpenSpec 2.0); changes are created with `openspec new change` through the
  managed runner, and the stale `.agents/skills/source-command-opsx-*` wrappers are
  gone. The `opsx-*` model-routing keys remain as compatibility-only labels.
- **Breaking:** the AGENTS.md/CLAUDE.md compiler-check exception is retired (owner
  decision D1); the second final-gate run exists only as B5 reverification.
- **Breaking:** BDD retry-until-green guidance (`--retry`, `@flaky`, flake budgets,
  `scripts/flake-budget.sh`) and the continuous-testing branch are removed;
  nondeterminism is declared in the contract's stochastic profile.
- **Breaking:** the 12 per-harness skill install targets (OpenCode, Kimi, MiniMax,
  Cursor, Codex, Gemini, Roo, Devin, Windsurf, Zed ×2, Cline) — those harnesses read
  `~/.agents/skills`.
- Existing `KBD-POLICY-ACK` acknowledgements match the reworded conflict issue
  text only after being re-recorded; files carrying the policy block no longer
  conflict.

### Fixed

- Journals signed by 1.11.x that contain zeespec stage receipts replay again.
  `BoundaryKind::Zeespec` is restored as a legacy, replay-only variant: new
  zeespec receipts are still refused at command admission, so the retired
  zeespec workflow stays gone. This reverses the breaking replay removal listed
  under 1.14.0; signed history is append-only and could not be migrated away.
  The replay fixture and `tests/pre_1_12_zeespec_replay.rs` are restored.

## [1.14.0] - 2026-11-05

### Changed

- OpenSpec is the primary spec engine, at **1.16.0** (exact pin and runner
  minimum). The managed runner treats a cached selection below the minimum as
  stale and installs the current release, and refuses Node.js older than
  22.12.0 (OpenSpec 1.16's npm floor) with a receipt naming the standalone
  binary. The root `engines.node` floor is now `>=22.12.0`.
- `/kbd-init --spec-engine openspec|speckit|native-kbd` (default `openspec`)
  adopts an engine for a new project and records `specBackend`; existing
  projects are never switched. The runner gains an `init` operation wrapping a
  non-interactive `openspec init`.
- `kbd-spec`, `kbd-plan` and the plan prompt write OpenSpec changes by default
  and no longer fall back silently to native files. They read the same
  `specBackend` key as `kbd-apply`; the legacy `change_backend` key is honored
  when `specBackend` is absent.
- GitHub Spec Kit is the supported backup engine, pinned at **1.3.0**; the
  `sk_*` adapter was checked against a real `specify init` project. `native-kbd`
  is kept for compatibility and is no longer offered as a default.

### Removed

- The retired zeespec boundary (`BoundaryKind::Zeespec`) is gone from
  kbd-runtime and the `prometheus` CLI, with its replay fixture and test.
  **Breaking:** a journal signed by 1.11.x that contains zeespec receipts no
  longer replays.
- Stale slash commands: `scripts/generate-commands.js` and
  `scripts/register-slash-commands.sh` now delete the command and prompt files
  they generated for skills that no longer exist (for example the old
  `zeespec-*` and `sync-*` commands). Files without the generator's marker are
  left alone.

## [1.13.0] - 2026-10-18

### Added

- KBD phase readiness contract: a versioned, content-digested exit contract
  adopted through a canonical writer before execution, a Plan→Execute readiness
  gate, and a terminal evidence-bound closure evaluator (#186, bounded
  #184/#185 amendment).
  - `shared/scripts/kbd-contract-adopt.sh` records adoption as an immutable
    canonical decision keyed to the contract SHA-256 digest, with supersedes
    lineage (`--supersedes`) and idempotent re-adoption (`ALREADY-ADOPTED`).
  - `shared/scripts/kbd-phase-readiness-check.sh` validates adoption
    presence/unambiguity, project/run/phase identity, digest freshness
    (`REJECT-STALE`), task coverage, and contract quality (scenario matrix,
    budgets with UNKNOWN-token fallback, verification packet).
  - `skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh` refuses
    Plan→Execute dispatch unless readiness is READY, and refuses to dispatch
    closed phases.
  - `shared/scripts/kbd-phase-close-eval.sh` closes a phase only when every
    adopted scenario has valid PASS evidence bound to the current run and
    adopted contract digest, with recomputed source freshness and a budget
    predicate against declared ceilings. Missing `budgetConsumption` accounting
    is a `BLOCKED` refusal (`budget-consumption-missing`), never waived; a
    valid unchanged replay is an idempotent `ALREADY-CLOSED` no-op.
  - Documentation: [phase readiness contract](site/docs/kbd/phase-readiness-contract.md).
  - Enforcement is a shell-layer convention with disclosed bypass paths and
    self-reported (not independent) budget accounting; the documented limits
    are part of the release. Candidate qualification evidence:
    `docs/releases/1.12.2-candidate-qualification.md`.

## [1.12.2] - 2026-10-12

This release also covers 1.12.0 and 1.12.1, which were never tagged.

### Added

- `kbd-apply reconcile [<phase>] [--repair] [--json]` is trustworthy and phase-safe
  in both the full and mini packs (#176, mini #52). It exits 0 only for a complete
  clean scan, 1 for drift and 2 for invalid input or an incomplete scan; JSON output
  gains `errors`. Repair acts only on the active phase, passes the canonical task
  identity (`c-t1`, not `c/1`) to the boundary guards, and honors an explicit
  `KBD_ORCHESTRATOR_ROOT`. Archived OpenSpec, native KBD and Spec Kit artifacts are
  supported; ambiguous archives are errors. Windows behavior is unverified.
- The Codex skill catalog is trimmed by a managed, reversible block in
  `~/.codex/config.toml`, driven by `config/codex-catalog.txt` and applied on every
  install and update (#173). Undo it with
  `bash shared/scripts/codex-catalog-config.sh --uninstall`.
- Delivery-cadence adopts proven historical publications without delivery credit
  (#180) and mirrors the encrypted Cadence IPFS history reference (#177).
- Team export preserves capabilities and cross-harness handoff semantics (#175), and
  scoped team handoff provenance can be inspected (#166).
- **Spec-engine support: OpenSpec (default), GitHub Spec Kit, native-kbd, and a
  documented extension path.** `kbd-apply` now implements the full five-op
  adapter contract (list / progress / mark-done / verify / archive) for all
  three engines: `sk_verify` (structural gate — every checkbox in
  `specs/<change>/tasks.md` checked plus `spec.md` present) and `sk_archive`
  (moves `specs/<change>` to `specs/archive/<date>-<change>`) join the existing
  speckit ops; change-scoped detection accepts `specs/<change>/{tasks,spec,plan}.md`.
  New `config/spec-engines.json` registry records engine metadata and exact
  version pins (openspec `@fission-ai/openspec` 1.14.0, GitHub Spec Kit
  `specify` 1.1.2) with an update procedure for new upstream releases, documented
  in `docs/guide/spec-engines.md` together with the add-a-new-engine contract.
  New `shared/scripts/spec-engine-info.mjs` (pure Node, zero deps) reports the
  default engine, the registry, and the detected engine for a project root.

### Removed

- **The unused Zachman 5W1H interrogator skill is removed entirely**: the skill
  (and its four sub-skills), the coverage gate in `kbd-spec`, its stage in
  `start-business-build`, its boundary kind in `kbd-runtime` and
  `prometheus-cli`, its state-dir position-ingest in `shared/lib/position.sh`,
  and every documentation, guide, site, and baseline reference. The enforced
  pipeline is now PMPO → OpenSpec/Spec Kit → forge-rs; historical run-history and
  review-packet records were redacted in place.

- `prometheus-context-bootstrap --layout v4`: Prometheus Rules Architecture v4. Seeds `rules/src/`
  (constitution with v3 §A verbatim plus A-15…A-17, a named skill-routing table, seven stack rule files,
  four domain rule files), the `rules/build.sh` generator with `--check`, `scripts/check-file-lines.sh`
  (500-line limit with an allowlist), `scripts/check-architecture.sh` (feature-based clean-architecture
  edges for React, Rust and Flutter, kebab-case names in React packages), the `file-lines-guard` and
  `build-guard` hooks, and a `commit-msg` hook for `Assisted-by` / no `Signed-off-by`. `verify.sh` detects
  the layout and runs v4 checks. Opt-in: without the flag both scripts behave exactly as before.

### Fixed

- Embeddings run on the CPU by default off Apple Silicon, `surreal-memory-server` is
  pinned to v1.10.2 (which restarts on the CPU and replays after a Metal fault), and
  the doctor surfaces verifier errors and tells a connection failure from an HTTP
  failure (#170, #171).
- The installer no longer stages `.claude-plugin` layout symlinks into the
  generation (#169), and hook-written runtime state no longer counts as an
  uncommitted source (#174).
- Hook activation recovers cleanly and installed payloads stay immutable (#168).
- Cadence fixes: promotion of committed nested source repairs (#164), reconciliation
  of already-operated frozen artifacts (#165), preserved native credential account
  identity (#178), and selection of the packaged Boss main window (#181).
- The operated proxy tool-argument repair is pinned (#179).
- `prometheus-context-bootstrap` no longer installs `deny: Edit(.kbd-orchestrator/**)`.
  The rule blocked `/kbd-init` and every KBD stage skill from writing their
  artifacts. `bootstrap.sh` removes the legacy entry from existing projects
  (`REPAIR`), and `verify.sh` fails any project whose deny list covers the whole
  directory so the rule cannot return unnoticed.

## [1.11.2] - 2026-10-05

### Added

- `scripts/tests/payload-activation.test.mjs`: copies the built plugin payload to a temp directory, uses an empty `HOME` so activation must bootstrap from nothing, and executes every shipped hook. It fails on the released 1.10.0/1.11.0 payloads (missing `scripts/lib` modules) and runs in `npm run check:distribution`.
- `prometheus doctor` checks `plugins.source-topology` (fails when a registered marketplace source is missing; warns on a topic-branch or dirty source and when clients disagree; a linked release-line worktree is only noted) and `plugins.native-cache-skew` (advisory: installed plugin vs active generation, and live sessions still on a superseded version). The logic lives in `scripts/lib/plugin-source-topology.js`, with `scripts/check-plugin-source.js` as the entry point.
- `update-skill-pack.sh` and `refresh-native-plugin-installs.sh` refuse to refresh from a checkout that is a registered plugin source and is on a topic branch, unless `--allow-topic-branch` is passed.
- Operations page: plugin sources and hook failures.
- `scripts/check-plugin-source.js` ships in the plugin payload and generation. The doctor runs it only from `PROMETHEUS_SOURCE_ROOT` or the installed generation, never from the current working directory.

### Changed

- `hook-entry.mjs` classifies a failed bootstrap (`PAYLOAD_INCOMPLETE`, `BOOTSTRAP_FAILED`, `NOT_ACTIVATED`) and prints one actionable line instead of a raw Node stack trace. `PROMETHEUS_HOOK_DEBUG=1` keeps the raw output.

## [1.11.1] - 2026-10-03

### Fixed

- The Claude plugin payload now ships every `scripts/lib` module that `install-plugin-generation.js` and
  `hook-entry.mjs` import. The distribution generator copied only `scripts/lib/skill-system.js`, so Stop
  hooks that ran the packaged installer failed with `ERR_MODULE_NOT_FOUND` for `./lib/capabilities.js`.
  The generator now copies the directory, and `skill-system-distribution.test.mjs` resolves every relative
  import of every packaged script against the payload so the omission cannot recur silently.

## [1.10.0] - 2026-09-19

### Added

- `karpathy-progress-memory`, a standalone process skill that records successful
  KBD task, change, and phase boundaries as bounded, secret-free, idempotent
  events. It validates canonical identity, appends chronological project
  history, submits through project-scoped `pk`, and queues a durable outbox
  operation when the memory service is unavailable.
- Deep-research stage contracts, resumable checkpoints, package manifests,
  provenance sidecars, four-state claim labels, source scoring, contradiction
  detection, and headless daemon job execution.
- Learner-model gap, session, certification, and FSRS write paths plus one
  artifact contract shared by teaching, retention, grading, and certification.
- OpenSpec capability deltas for KBD progress continuity, research pipeline
  execution, and learner-model coherence.

### Changed

- KBD now derives the current and next change/task from canonical task state.
  Stored `exactNextCommand` text remains visible as historical operator intent
  and no longer selects work.
- Task completion and bottleneck evaluation carry qualified change/task
  subjects, so duplicate task IDs cannot resolve to the wrong change.
- Phase, child-phase, change, task, restart, and resume transitions regenerate
  consistent JSON, Markdown, status, and reminder projections.
- Process skills are version 1.6.0, research skills 1.1.0, the research server
  1.7.0, and learn 1.6.0.
- The research package uses one output root and plain package directories under
  `~/.prometheus/research`.

### Removed

- Migrated sovereign-sync skills, service installers, and stale pack-owned
  companion references. The pack no longer depends on the retired companion
  surfaces.
- Redundant recovery backups, prompt snapshots, and transient captures whose
  information was already preserved in canonical project history.

### Fixed

- KBD could retain a completed task as its apparent next command for hundreds
  of revisions while the canonical journal had advanced. Context compression
  or restart then made the real position difficult to recover. Cursor folding,
  rendering, reanchoring, and durable progress receipts now agree on the same
  canonical position.
- Deep-research previously emitted stage completion without executing work,
  accepted invalid stage artifacts, and could format checkpoint timestamps in
  1970. Stage execution and boundary validation now fail closed.
- Research daemon jobs previously received an undefined `--daemon-job` flag.
- Learn skills previously disagreed on artifact paths and consumed grounding
  fields that the grounding script did not emit.

## [1.8.0] - 2026-08-30

- Recorded the pack version across the release matrix and generated distribution, and documented installation and rollback for the 1.8 generation. [Tagged source](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/v1.8.0)

## [1.7.0] - 2026-08-03

### Added

- Signed, exact-replay Sovereign Sync push resources with durable receipts,
  resumable events, shared REST/MCP execution, and Rust-generated OpenAPI 3.1.
- Private Unix-socket transport, durable P2P identity, secret pairing tickets,
  endpoint/signing-key enrollment, replay defense, and explicit token-authenticated
  loopback TCP.
- Signed folded-state checkpoints, frontier caches, immutable hash-linked journal
  archive segments, and rollback metadata.
- Ed25519-signed plugin generations, a separate trust store, one canonical skill
  index across host/agent/mobile surfaces, and 14 signed target receipts.
- Git-object protected-test certification with SSH-signed change approvals and
  cumulative local adversarial-review receipts.
- Deterministic `docs:sync`, generated route/schema/CLI/capability references,
  five architecture decisions, and a documentation-only main-branch sync bot.

### Changed

- Learner storage is Loro-only. Immutable uniquely keyed evidence is folded
  deterministically into mastery and conservative FSRS state after local writes
  and remote imports; legacy snapshots are preserved during migration.
- KBD pause is explicitly advisory. Journal transactions and causal-frontier
  validation remain the write-concurrency boundary; agent Bash and Python are
  unrestricted.
- Installation is strict by default, with explicit `--skills-only` and
  non-certifying `--best-effort` modes plus post-install artifact verification.
- Release validation is local-only. Hosted automation is limited to deterministic
  managed-document synchronization and GitHub Pages packaging/deployment.

### Removed

- The obsolete KBD voter/quorum facade, mutation-policing `PreToolUse` hooks,
  hosted test workflows, and stale Automerge/operator-ID documentation.

## [1.6.2] - 2026-08-03

### Fixed

- Generate distinct Claude Code and Codex hook manifests from one declarative contract, with every command pinned to a content-addressed runtime bundle.
- Bootstrap missing bundles only from the native plugin payload after hash verification; hook execution no longer follows mutable `stable` or `current` projections.
- Validate the source manifest, immutable generation, bundle index, fixed runner, and actual Codex cache in `prometheus doctor`.
- Install versioned hook runtime receipts and retain older bundle mappings so already-open sessions remain resolvable during cache turnover.

## [1.6.0] - 2026-07-08

- Added the native Rust research executable in the substrate. [Tagged source](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/v1.6.0)

## [1.4.0] - 2026-06-28

### Added

**Learn Domain — Feynman-Spine Learning & Education Capability (12 skills)**

- `skills/learn/ui-surface` — Cross-harness UI rendering primitive; Tier 0 (text), Tier 1 (AskUserQuestion / file-pair), Tier 2 (surface-bridge MCP App)
- `skills/learn/learn-goal` — Learning desire intake with deep research scoping, honest feasibility gate (GREEN/YELLOW/RED), KB adapter integration (`--kb` flag)
- `skills/learn/learn-survey` — Diagnostic placement; 11 items (5 conceptual, 3 procedural, 3 misconception probes); sets recursion floor; seeds learner model
- `skills/learn/learn-plan` — Concept dependency DAG in surreal-memory; topological sort; time-budgeted curriculum with `--replan` support
- `skills/learn/feynman-loop` — Core PMPO learning loop (Spec=concept+depth, Plan=explanation structure, Execute=plain-language+analogies+skeptic, Reflect=grade+gaps); vertical recursion with floor guard (max depth 3); horizontal escalation (novice→peer→skeptic)
- `skills/learn/learn-grade` — External, source-grounded, sycophancy-corrected grader (S-02 pattern); pass = score ≥ 0.7 AND misconceptions_absent
- `skills/learn/learn-retain` — FSRS-6 spaced retrieval; reads due queue; four-tier rating mapping (Easy/Good/Hard/Again by score)
- `skills/learn/learn-practice` — Derivation/implementation/transfer modes; mastery-gated access; interleaved schedule
- `skills/learn/learn-certify` — OB 3.0 / W3C VC JSON-LD credential; integrity guardrail (Δmastery > 0.4 → integrityNote); self-issued via did-plc
- `skills/learn/learn-kb` — KB registry management; four adapter types (dify:, palace:, local:, web:); privacy guarantee — never forwards KB content to external APIs
- `skills/learn/learn-about-system` — Zero-friction adoption entry for the Prometheus stack; uses meta-corpus files for KBD and skill-pack self-teaching
- `skills/learn/learn-harness` — Harness auto-detection; 13-row capability map table (5 harnesses); per-harness orientation; `--map-only` flag

**Substrate Crates (Layer A)**

- `substrate/storage-provider` — `StorageProvider` + `CrdtEngine` async traits; `LocalDirAdapter` (default); `AutomergeEngine` (automerge 0.5); `IrohDocsAdapter` stub
- `substrate/learner-model` — automerge-backed CRDT learner model; simplified FSRS-6 scheduler with Rating enum; JSON-RPC stdin/stdout shell interface; PFA mastery update at ≥5 observations; binary + lib crate
- `substrate/surface-bridge` — Axum 0.7 MCP App server on `127.0.0.1:7890`; `/health`, `/mcp/detect-surface-tier`, `/mcp/render-ui-intent`, `/mcp/collect-response`; macOS launchd plist included

**Schemas & Corpora**

- `docs/learn/schemas/learner-model.schema.json` — JSON Schema Draft-07 for LearnerModel, ConceptState, FSRSCard, GapRecord, LearnerModelSeed
- `docs/learn/schemas/grounding-corpus.schema.json` — JSON Schema for content-grounding output; six source_type values
- `docs/learn/schemas/kb-corpus.schema.json` — Extends grounding-corpus; adds `kb_source` and `privacy_mode` required fields
- `docs/learn/crdt-conflict-semantics.md` — Field-level CRDT merge rules; mastery=LWW+vc, observations=union-append, fsrs_card.due=min, fsrs_card.stability=max
- `docs/learn/surface-tier-detection.md` — Tier 0/1/2 detection signals per harness
- `docs/learn/kb-adapter-guide.md` — Privacy-safe KB adapter usage reference (195 lines)
- `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` — 18 source entries + 8 misconceptions for KBD self-teaching
- `docs/learn/meta-corpus/skill-pack-corpus.json` — 15 source entries + 9 misconceptions for skill-pack self-teaching

**Shared Scripts**

- `shared/scripts/content-grounding.sh` — 4-tier source chain (Dify KB → palace RAG → MCP filesystem → Firecrawl web); `--include-misconceptions` flag
- `shared/scripts/content-grounding-kb.sh` — Privacy-safe KB adapter; NEVER forwards KB content to external APIs; warns on external API env vars
- `shared/scripts/detect-surface-tier.sh` — `default`/`--print`/`--json` modes; reads env vars + surface-bridge.pid check

**Integration Tests**

- `tests/learn/integration-basic-flow.sh` — write-goal → write-survey → write-artifact → write-grade pipeline
- `tests/learn/integration-full-loop.sh` — FSRS card mutation, practice-result fields, VC JSON-LD structure, integrity guardrail
- `tests/learn/integration-kb.sh` — local adapter, privacy guardrail, corpus schema validation, KB registry
- `tests/learn/integration-meta.sh` — KBD corpus, skill-pack corpus, detect-surface-tier, learn-about-system, learn-harness, all 12 skills validate

**Documentation**

- `docs/guide/10-learn-skills.md` — Operator guide chapter for the learn domain (274 lines)
- `CLAUDE.md` — New `## Learn Domain` section: four-layer architecture, substrate crates, surface tier degradation contract, KB adapter pattern, mastery criterion, anti-sycophancy mandate

**Infrastructure**

- `scripts/install-skills-flat.sh` — `install_learn_substrate` function: builds storage-provider, learner-model, surface-bridge Rust crates; installs learner-model binary to `~/.local/bin`; installs surface-bridge as macOS launchd service
- `shared/scripts/detect-toolchain.sh` — Added `learner-model` binary check and `surface-bridge` HTTP reachability check
- `.claude-plugin/plugin.json` — Added all 12 `./skills/learn/<skill>` paths
- `marketplace/marketplace.json` — Added `learn` domain entry with 12 skills

### Changed

- `skills/process/kbd-process-orchestrator/prompts/reflect.md` — inlined sycophancy-correction invocation contract and scoring thresholds directly into the "Sycophancy Self-Check (MANDATORY)" section.
- `skills/process/kbd-process-orchestrator/prompts/assess.md` — added invocation subsection mirroring reflect.md with Assess-appropriate thresholds.
- `skills/process/kbd-process-orchestrator/prompts/plan.md` — added Sycophancy Self-Check section (previously missing).
- `skills/process/kbd-process-orchestrator/references/integrations/sycophancy-correction.md` — Plan phase promoted from exclusion to first-class checked phase.
- `scripts/check-prerequisites.sh` — invokes smoke-test.sh after sycophancy-correction binary install to verify functionality.
- `scripts/validate-skills.js` — hardened against symlink loops and recursive structures (lstat + isSymbolicLink skip, realpath-based visited-set, expanded skip list).
- `CLAUDE.md` — directory structure diagram updated with `skills/learn/` subtree and `substrate/` crates; `## Learn Domain` section added
- `README.md` — learn domain added to directory structure and skills section

### Submodule bumps
- `skills/imported/sycophancy-correction` → d973ef370fe238ceeed72c1b462e85ee83144734

[1.4.0]: https://github.com/Prometheus-AGS/prometheus-skill-system/releases/tag/v1.4.0

## [1.2.0] - 2026-04-15

- Integrated Surreal memory and the sycophancy CLI and wired Cargo dependencies for those tools. [Tagged source](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/v1.2.0)

## [1.1.0] - 2026-04-15

### Added

**Skills (61 total, 0 errors, 0 warnings)**

- React entity management suite (27 skills): setup, CRUD, GraphQL, Prisma, realtime, optimization
- Process orchestration (20 skills): KBD orchestrator, iterative evolver, PMPO skill creator
- GitOps CI/CD (4 skills): bootstrap, transform, ArgoCD multi-cloud, Kustomize overlays (TJ-CICD-001)
- BDD testing (1 skill): Cucumber.js + Playwright with video recording
- Imported: artifact-refiner (9 skills) via git submodule

**Nested PMPO Pipeline**

- Iterative-evolver outer loop delegates to KBD inner loop for software domain execution
- KBD auto-detects OpenSpec for structured change management
- Artifact-refiner QA gate per completed change
- Evolver bridge file (`evolver-bridge.json`) maps plan items to KBD changes
- KBD reflect reports back to evolver with artifact quality metrics

**Rust CLI (tools/prometheus-cli/)**

- 4-crate workspace: prometheus-cli, prometheus-agents, prometheus-learn, prometheus-cedar
- 15 subcommands: install, uninstall, list, search, audit, verify, doctor, status, generate, validate, build, memory, evolve, learn, optimize
- 10-platform adapter library (Claude Code, OpenCode, Cursor, Codex, Gemini CLI, Roo Code, Windsurf, Amp, Cline, Kilo Code)
- Cross-platform `TraceCapture` protocol for self-learning pipeline
- Cedar Skill Mutation PEP: gates skill.mutate/generate/promote/trace.capture
- Self-learning pipeline: trace capture, evaluation, knowledge compilation scaffolding, dspy-rs optimization scaffolding

**Surreal-Memory Integration**

- Root `.mcp.json` with surreal-memory, tavily, sequential-thinking servers
- Entity mapping patterns for all skill domains (evolver, KBD, GitOps, artifact-refiner)
- Comprehensive integration reference (`shared/references/surreal-memory-integration.md`)
- Graceful degradation when surreal-memory unavailable

**OpenCode Support**

- 3 TypeScript tool definitions (`.opencode/tools/`): evolve, kbd, gitops
- `.opencode/package.json` for auto-dependency installation
- Compatibility declared for 8 platforms in plugin.json

**Distribution**

- Marketplace with 5 granular plugin entries (full pack + domain-specific)
- TypeScript multi-platform installer (`scripts/install-platforms.ts`)
- GitHub Actions CI: validate skills, check formatting, cargo check + clippy
- Skills.toml/Skills.lock format (cowork-compatible)

**Governance & Architecture**

- Cedar default policies: development (permit all), staging (require validation), production (deny mutations)
- Self-learning architecture reference (`shared/references/self-learning-architecture.md`)
- UAR-embeddable library design (prometheus-learn as library, CLI as thin wrapper)
- Unified hooks.json with 5 events: SessionStart, PreToolUse, PostToolUse, SubagentStop, Stop

### Changed

- Recursive skill validator now scans sub-skills at any nesting depth
- Validator excludes backslashes inside code blocks from path separator warnings
- Validator adds line count checks (warning at 500, error at 800)

### Fixed

- Missing SKILL.md in prometheus-entity-skills container directory
- Duplicate pmpo-skill-creator in skills/creation/ (removed, kept skills/process/)
- Backslash path separators in bdd-testing, entity-crud-table, entity-realtime-channel
- pmpo-skill-creator sub-skill directory names mismatched frontmatter (clone→clone-skill, etc.)
- Empty skill categories (rust, ui-ux, devops, documentation) removed from manifests
- Plugin.json upgraded to full schema (author object, mcpServers, hooks path)

[1.1.0]: https://github.com/Prometheus-AGS/prometheus-skill-system/releases/tag/v1.1.0
