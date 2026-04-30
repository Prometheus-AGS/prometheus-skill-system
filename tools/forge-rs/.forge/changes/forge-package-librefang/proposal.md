# forge package-librefang — Subcommand Proposal

> Queued for `phase-librefang-wasm-onramp` after change-005 of
> `phase-compliance-and-power-multiplier` ships the surrounding skills.

## Why

Today, packaging a native-agent's `agent-skill` crate as a `.lf-skill.zip` for
upload to bossfang requires four manual steps:

1. `cargo build --target wasm32-unknown-unknown --release -p <agent>-skill`
2. Locate the produced `.wasm` under `target/wasm32-unknown-unknown/release/`
3. Copy `<agent>_skill.wasm`, `skill.toml`, and `README.md` into a staging dir
4. `zip` the staging dir

`/start-business-build`'s stage 6 currently prints these as a manual fallback
when `forge package-librefang` is missing. Adding the subcommand collapses
the sequence to a single command that participates in `forge`'s existing
template/registry/MCP infrastructure.

## What

New `forge package-librefang [path]` subcommand. When run from a project
root (i.e., a directory containing `skill.toml` at the top level and an
`agent-skill` crate in `crates/`):

1. Validate the project layout. Reject if:
   - `skill.toml` missing or has `[runtime].type ≠ "wasm"`.
   - `crates/agent-skill/` missing or its `Cargo.toml` doesn't declare
     `crate-type = ["cdylib"]`.
2. `cargo build --target wasm32-unknown-unknown --release -p <agent>-skill`.
   Stream cargo output to the user; abort on non-zero exit.
3. Run the same `validate-wasm-abi.sh` logic (port to Rust using `wasmparser`)
   to confirm the produced module exports `memory`, `alloc`, `execute` and
   imports only from `librefang`.
4. Compute the staging set:
   - `<agent>_skill.wasm` → renamed to `<agent>.wasm`
   - `skill.toml` from project root
   - `README.md` from project root (if present)
   - Anything declared under `[package].assets` in skill.toml (future-extension)
5. Produce `<agent>.lf-skill.zip` with deterministic ordering and timestamps
   (so identical sources produce byte-identical zips — useful for content
   hashing during install).
6. Emit JSON to stdout: `{"zip": "...", "size_bytes": N, "skill_name": "...", "wasm_sha256": "..."}`.

## Crate changes

| Crate | Change |
|---|---|
| `forge-cli` | New `Commands::PackageLibrefang { path: PathBuf }` variant. ~20 LOC of clap, dispatches to `forge-skills::package_librefang` |
| `forge-skills` | New module `package.rs`. Loads SkillManifest, runs cargo, zips. Uses `zip` + `wasmparser` crates |
| `forge-core` | New struct `PackageReport { zip_path, size, sha256, skill_name }` |
| `forge-mcp` | Add `forge_package_librefang` tool that wraps the subcommand |

Estimated effort: 2–3 days for one Rust engineer, including test fixtures
(a fixture project with `agent-skill` + `skill.toml`) and the integration test
that loads the produced zip via `librefang-skills::PreparedLocalSkill`.

## Acceptance Criteria

- `forge package-librefang ./test-agent` from a generated native-agent
  project produces `test-agent.lf-skill.zip` matching the schema in
  `librefang-skills/src/publish.rs::PreparedLocalSkill`.
- The produced zip loads cleanly via `librefang-skills::install_from_zip`
  (integration test).
- Re-running on identical sources produces a byte-identical zip
  (deterministic build).
- `forge package-librefang --help` documents the subcommand and references
  `skills/process/native-agent/skills/upload-to-bossfang/SKILL.md`.

## Why Not Inline This in change-005

Three reasons:
1. **Scope discipline**. change-005 is already covering: orchestrator skill,
   upload skill (with SSRF audit), threat model, marketplace entry, and slash
   command registration. Adding a Rust subcommand impl would push the change
   over its M effort budget into L territory.
2. **Tractability**. The actual cargo invocation + zip produces real I/O that
   needs integration tests against fixture projects; that's a separate
   review surface.
3. **Pipeline integrity**. The orchestrator's stage 6 prints a manual fallback
   when the subcommand is missing, so users can still complete the pipeline
   end-to-end. The subcommand is a quality-of-life upgrade, not a blocker.

## Tasks (for the Rust impl phase)

- [ ] Add `Commands::PackageLibrefang` to `forge-cli/src/main.rs`
- [ ] New `forge-skills/src/package.rs` with `package_librefang` fn
- [ ] Add `wasmparser` and `zip` to `forge-skills/Cargo.toml`
- [ ] Fixture project under `forge-skills/tests/fixtures/agent-skill-min/`
- [ ] Integration test asserting the produced zip loads via
      `librefang-skills::PreparedLocalSkill::from_zip`
- [ ] Determinism test: build twice, byte-compare the zips
- [ ] Update `forge-mcp/src/lib.rs` with `forge_package_librefang` tool
- [ ] Update `forge-rs/README.md` CLI Reference section
- [ ] Update `forge-rs/CLAUDE.md` Build commands section
