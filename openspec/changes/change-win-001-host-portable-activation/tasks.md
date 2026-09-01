## 1. Host capability probing and key protection

- [x] 1.1 Add `scripts/lib/capabilities.js` probing symlink, junction, hardlink, and executable-bit support inside the store root, not a temporary directory
- [x] 1.2 Cache probe results at `~/.prometheus/capabilities.json` and invalidate on installer version change
- [x] 1.3 Replace the mode-`0600` gate with a platform-dispatched owner-only predicate returning a structured verdict with reason and remediation
- [~] 1.4 Add a Windows security-descriptor inspector to `crates/prometheus-exec` reporting owner SID, DACL protection, inherited ACE count, and trustee SIDs as JSON — **written but never compiled**; this host has no Cargo registry cache and building `prometheus-exec` pulls the full wasmtime tree, which the repository's Rust build discipline forbids during implementation. Needs a build leg.
- [x] 1.5 Report the `icacls` remediation without executing it, and remove every remaining inference from `process.platform`

## 2. Portable generation identity

- [x] 2.1 Emit manifest schema version 2 with entry type, normalized executable bit, content hash, size, and recorded link target
- [x] 2.2 Remove permission modes, timestamps, ownership, and platform attributes from the hashed payload and canonicalize under RFC 8785
- [~] 2.3 Re-apply `0755`/`0644` from the manifest at materialization on POSIX so identity is umask-independent — implemented in `applyManifestMode` and `normalizeDirectoryModes`; the assertion that two umasks materialize identical modes is skipped on this host and belongs to the Linux and macOS legs
- [x] 2.4 Write the unhashed materialization record with link strategy and per-entry degradations
- [~] 2.5 Accept schema version 1 in the verifier, refuse to create it, and re-pin the bundle identity in `hooks/hooks.json` in the same commit — the digest rule, the signature-envelope rule, and the re-pin are done and fixtured; end-to-end verification of a real schema-1 generation directory needs a host that already has one installed

## 3. Activation primitives

- [x] 3.1 Pass an explicit link type at every `fs.symlinkSync` call site and descend symlink, then junction, then copy
- [x] 3.2 Replace directory-link rename with unlink-and-create under the store mutex, leaving a recoverable pending breadcrumb
- [x] 3.3 Make the authoritative activation pointer a file swapped by rename; demote the `-L` assertion to an advisory check
- [x] 3.4 Resolve the generation through the pointer file in `hook-runtime-v1.sh` and `bootstrap-hook-runtime.sh`, preserving the containment check
- [~] 3.5 Strip the `\\?\` verbatim prefix before comparing a canonicalized path against the generation store root — done and fixtured in `scripts/lib/store-paths.js`, and defensively in the shell runtime. `substrate/exec-tier-w/src/authorization.rs` now reads `pointers/current` first, reduces an absolute junction target back to `generations/<id>`, and normalizes both sides of the containment comparison. That Rust is **type-checked but not compiled or run** (`rustc --emit=metadata` needs no linker); its rule is the same one the Node implementation carries, which is fixtured including the UNC case.

## 4. Shell-free hook dispatch

- [x] 4.1 Add hook dispatch subcommands to `crates/prometheus-exec` covering the hot-path hooks, with `sha2` replacing `shasum` and a held-open advisory file lock replacing the PID-file mutex — landed as **`crates/prometheus-hook`**, a separate crate rather than a `prometheus-exec` subcommand: 4.4 ships one copy per target and `prometheus-exec` links wasmtime, which would add roughly a gigabyte to the payload and pay wasmtime's startup on every PreToolUse. The release binary is **257 KB**. Two dependencies (`sha2` as specified, `serde_json`), `opt-level="z"` + LTO + strip. Builds clean with no warnings; both FIPS 180-2 known-answer tests for the digest gate pass. Verified byte-identical error envelopes against the shell runtime on a tampered dispatcher (`DISPATCHER_HASH`) and an unknown bundle (`NOT_ACTIVATED`). **The advisory file lock is still to do** — `std::fs::File::lock` (stable 1.89, this host runs 1.98) provides it with no dependency; the bootstrap mutex remains the shell `mkdir`+pid one.
- [x] 4.2 Emit all 31 hook entries in exec form with an argument vector and an absolute executable path — all 31 Claude and 30 Codex entries are `{type:"command", command:"node", args:[...]}`. `command` is `node` rather than the compiled dispatcher because `hooks.json` is one file shared by every host, the harness substitutes only `CLAUDE_PROJECT_DIR`/`CLAUDE_PLUGIN_ROOT`/`CLAUDE_PLUGIN_DATA` with no platform placeholder, and a binary cannot bootstrap itself. `scripts/hook-entry.mjs` execs the compiled dispatcher the moment one exists, so the hot path is compiled once 4.4's artifacts ship.
- [x] 4.3 Remove `-x` gating from the runtime and launch by explicit interpreter argv resolved from the manifest entry type — the interpreter is a signed field of the hook-runtime receipt (`dispatcherInterpreter`), covered by the bundle identity and allowlisted by the runtime.
- [~] 4.4 Ship prebuilt dispatcher binaries for six targets in the payload and select the correct one at install — **the mechanism is done and proven; the artifacts are not.** `bin/` is a payload root, so a dispatcher is manifested, hashed, and signed like any other entry; the installer selects one by EXECUTING each candidate's `--version` rather than reading `process.platform` + `process.arch`, and places it at `runtime/v1/`. Proven end to end on this host: with the shell runtime moved aside, the compiled dispatcher alone served a real exec-form hook. Only the `x86_64-pc-windows-msvc` binary can be built here, so `bin/` is left unpopulated and gitignored rather than committing one target of six; **populating it is a release-matrix job (5.1)**. A payload with no binary is slower, not broken — verified: install, `--verify`, and hook dispatch all pass with `bin/` absent.
- [x] 4.5 Gate cold-path shell scripts on probed shell availability with an explicit message when absent — `probeShell()` reports the shell and the digest tool; the entry point reports `MISSING_SHELL` by name; the runtime names a missing digest tool; the bootstrap declares `awk` and `node`.

## 5. Cross-host verification and handoff

**Not a CI matrix, and deliberately so.** `CLAUDE.md`'s Local-Only Validation
section is MANDATORY, and `scripts/check-workflow-policy.mjs` enforces it
mechanically: only `docs-sync` and `docs-pages` may exist under
`.github/workflows`, and any workflow containing `npm test`, `cargo test`, or a
doctor invocation is rejected. A hosted matrix would fail the repository's own
gate before it ran. The matrix is instead assembled from per-host local legs,
which changes where the legs run and nothing about what they assert.

- [x] 5.1 Add a four-leg CI matrix: Linux, macOS, Windows with Developer Mode disabled, and Windows with it enabled — landed as a LOCAL matrix: `scripts/verify-host-leg.mjs` runs a host's leg and writes a receipt, `scripts/compare-host-legs.mjs` compares the collected legs, and `config/host-legs.json` declares the four required ones. A leg's id is DERIVED from the probed capability rather than declared, so a Windows host with Developer Mode enabled cannot be filed as the disabled leg — verified by a negative control.
- [~] 5.2 Assert byte-identical bundle identity across all four legs as the gating check — the gate exists, works, and is wired (`npm run verify:host-matrix:gate`); it compares `bundleId` and the golden payload digest and fails on divergence, on a mislabelled leg, and on a leg whose own checks failed. All three failure modes are exercised by negative controls. **Only the `windows-junction` leg has been collected**; the gate correctly refuses to pass until Linux, macOS, and `windows-symlink` are added.
- [x] 5.3 Prove the junction path satisfies `isSymbolicLink()` and `test -L`, and that a degraded copy leaves identity unchanged
- [x] 5.4 Prove the key predicate passes for an owner-restricted file and fails for an inherited or over-granted DACL
- [x] 5.5 Prove exec-form dispatch delivers an argument containing backslashes and `$` unmodified
- [x] 5.6 Record redacted command results and hashes in the phase evidence and complete the OpenSpec checklist — `evidence/legs/windows-junction.json` records every check, its exit code, and the SHA-256 of its raw output, with the home directory and temporary directory redacted and long digests elided. The hash is taken over the RAW bytes, so redaction cannot launder a failing result. `evidence/README.md` states what is collected, what is not, and why.

## Defects found and fixed outside the numbered tasks

Each of these blocked activation on Windows and had to be fixed for the numbered
work to be runnable at all.

- [x] `path.dirname(new URL(import.meta.url).pathname)` produced `C:\C:\...` in six scripts, so `generate-skill-system-distribution.js`, `check-release-version-matrix.mjs`, `build-codex-plugin.js`, and three fixtures could not run on Windows. Replaced with `fileURLToPath`.
- [x] `syncDirectory()` did not tolerate the EPERM `FlushFileBuffers` returns for a directory handle.
- [x] `syncPath()` opened its descriptor read-only. `FlushFileBuffers` requires WRITE access, so every payload file's durability flush failed with EPERM. Now opens `r+` and falls back to `r` only where it cannot.
- [x] `check-harness-adapters.js` gated three files on `stat().mode & 0o111`, which is zero for every file on a volume that cannot record a permission bit.
- [x] A JUNCTION cannot be used inside a staged payload. Its substitute name is absolute, so it keeps pointing at `.staging-<pid>-<rand>` after the rename into `generations/<id>`. The payload ladder is symlink → copy; the junction rung belongs only to activation links, which live at a stable path. Both halves are fixtured.
- [x] Shell-form hooks are handed to POWERSHELL on Windows whenever Git Bash is absent, per the Claude Code hooks reference. All 31 entries were `bash -c '<multi-line bash>'`, so the pack worked on Windows only where Git Bash happened to be installed. Exec form removes the shell entirely.
- [x] `.cargo/config.toml` requires sccache repo-wide, but sccache is not installed on this host, so every cargo invocation fails with `program not found`. Worked around with `--config "build.rustc-wrapper=''"`; the config itself is still a trap for anyone without sccache.
- [x] `CreateProcess` enforces MAX_PATH on the executable it is handed, even where the filesystem does not. A dispatcher probed inside a generation reached 326 characters here and failed with ENOENT despite being byte-identical to one that ran from a short path. The installer now probes the binary at its short, fixed installed path -- which is also where it actually runs.
- [x] `fs::canonicalize` returns a verbatim `\\?\` path, and msys2 cannot represent one at all (`cygpath -u` turns it into `/c/?/C:/...`). Handing it to `bash` as an argument produced a mangled filename and exit 126. Caught only by running the compiled dispatcher end to end.
- [x] The runtime called `shasum` unconditionally. It is a Perl script that ships with git-bash; `sha256sum` is coreutils, and a host can have either, both, or neither. It now selects whichever exists and names the missing dependency when neither does.
- [x] Directories created implicitly (`.agents` exists only because `.agents/plugins` was copied into it) had no manifest intent and no mode applied.
