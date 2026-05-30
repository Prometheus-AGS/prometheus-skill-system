# Assessment: setup-rebuild-and-staleness

**Date**: 2026-05-30
**Phase**: `setup-rebuild-and-staleness-2026-05-30`
**Goal**: Make `prometheus setup` capable of self-refreshing — `--rebuild` forces builds,
staleness detection surfaces drift between installed binary and source.

## 1. Current `setup.rs` Surface (observed)

`tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs` is **477 lines** with
**11 components**, 3 unit tests (all passing per prior phase).

### ComponentStatus enum (current)
```rust
pub enum ComponentStatus {
    Ok, Missing, SkippedDocker, SkippedLaunchd, Installed, NotInstalled,
}
```
6 variants. `needs_action()` returns true only for `Missing` / `NotInstalled`.
**No `Stale` variant exists.**

### `pub fn run` signature
```rust
pub fn run(non_interactive: bool, dry_run: bool, check: bool) -> Result<()>
```
3 flags wired in `main.rs:184-194` clap struct. **No `--rebuild` flag exists.**

### Component install-fn map (the load-bearing detail)

| id | detect | install | Notes |
|---|---|---|---|
| `surreal-memory` | docker/port | None | external — out of scope |
| `openai-proxy` | launchd/port | None | external |
| `forge-mcp` | launchd/port | **Some(load_launchd_forge_mcp)** | only loads plist; does NOT build the binary |
| `pk-mcp` | launchd/port | **Some(load_launchd_pk_mcp)** | only loads plist |
| **`liter-llm`** | binary | **None** | ❌ no install fn |
| **`prometheus`** | binary | **None** | ❌ no install fn |
| **`forge`** | binary | **None** | ❌ no install fn |
| **`pk-cherry`** | binary | **None** | ❌ no install fn |
| `sycophancy-correction` | binary | None | external (/usr/local/bin) |
| `template-forge` | binary | Some(install_template_forge_binaries) | ✅ already builds from submodule |
| `template-forge-mcp` | binary | None | piggybacks on template-forge install |

> **Implication for `--rebuild`**: This is not just adding a flag. Today, even when the
> 4 binaries show `NotInstalled`, `setup` has no installer to call (`comp.install.is_none()` →
> skipped at line 401). `--rebuild` requires adding 4 new install functions that wrap
> `cargo build --release -p <pkg> + cp` — essentially encoding what `scripts/install-binaries.sh`
> does. The good news: `install_template_forge_binaries` (line 184) is a working template;
> we model the 4 new ones on it.

### Detection (presence only — the root gap)

| Binary | Detect function | Returns |
|--------|----------------|---------|
| liter-llm | `detect_binary("liter-llm")` | `Installed` / `NotInstalled` |
| prometheus | `detect_binary("prometheus")` | `Installed` / `NotInstalled` |
| forge | `detect_binary("forge")` | `Installed` / `NotInstalled` |
| pk-cherry | `detect_binary("pk-cherry")` | `Installed` / `NotInstalled` |

`detect_binary` (line 88) shells out to `which`. **No mtime, no source-time, no version check.**
This is the gap the reflection identified.

## 2. Source-of-truth mapping for staleness (binary ↔ build source)

The locked decision is **mtime comparison: binary mtime vs. submodule HEAD commit time**.
The mapping is asymmetric because not all binaries come from submodules:

| Binary | Source dir | Submodule? | Stale comparator |
|--------|-----------|-----------|------------------|
| `prometheus` | `tools/prometheus-cli/` | ❌ **same repo** | binary mtime vs. `git log -1 --format=%ct -- tools/prometheus-cli/` |
| `forge` | `tools/forge-rs/` | ✅ submodule | binary mtime vs. `git -C tools/forge-rs log -1 --format=%ct HEAD` |
| `pk-cherry` | `tools/prometheus-knowledge/` | ✅ submodule | same pattern |
| `liter-llm` | `tools/liter-llm/` | ✅ submodule | same pattern |

> **Subtlety**: For `prometheus`, the binary will be stale whenever *any* commit touches
> `tools/prometheus-cli/`. Using the repo HEAD time would be too coarse (would mark stale
> on unrelated phase artifact commits). Using `git log -1 -- <path>` is correct and cheap.

## 3. Available crate dependencies (already in Cargo.toml)

```
anyhow, colored, serde, serde_json, chrono, dirs, walkdir, sha2
```

**No new dependencies needed.** Mtime read = `std::fs::metadata(p)?.modified()?` (returns
`SystemTime`). Git commit time = `Command::new("git")` shelled out (same pattern as
`detect_launchd`, `detect_binary`). Compare with `SystemTime` arithmetic.

## 4. Testability of the mtime comparator

The comparator function should be a **pure function over two `SystemTime` values**, so it
can be unit-tested without filesystem or git side effects. The IO functions that *fetch*
those times stay separate (and untested or covered by integration tests).

Recommended shape:
```rust
fn is_stale(binary_mtime: SystemTime, source_commit_time: SystemTime) -> bool {
    source_commit_time > binary_mtime
}
```
Trivial body, easy property tests (`is_stale(a, b) == !is_stale(b, a) || a == b`, etc.).

## 5. Gap Register

### G-STALE: Staleness detection
| ID | Gap | Action |
|----|-----|--------|
| G-STALE-1 | No `Stale` variant in `ComponentStatus` enum | Add `Stale` variant; update `icon()` (⚠️ yellow), `label()` ("stale — source newer than binary"), and `needs_action()` to return true |
| G-STALE-2 | No source-time lookup for the 4 binaries | Add `fn source_commit_time(path: &Path) -> Option<SystemTime>` shelling `git log -1 --format=%ct -- <path>` |
| G-STALE-3 | No binary mtime lookup | Add `fn binary_mtime(name: &str) -> Option<SystemTime>` via `which` + `fs::metadata` |
| G-STALE-4 | No comparator | Add `fn is_stale(binary: SystemTime, source: SystemTime) -> bool` — testable pure fn |
| G-STALE-5 | Existing `detect_*` for 4 binaries don't return `Stale` | Update each to return `Stale` when binary present AND source newer |

### G-REBUILD: Force-rebuild flag
| ID | Gap | Action |
|----|-----|--------|
| G-REBUILD-1 | `pub fn run` doesn't accept a `rebuild: bool` | Add 4th param; thread through |
| G-REBUILD-2 | Clap `Setup` variant lacks `--rebuild` flag | Add `#[arg(long)] rebuild: bool` in `main.rs:184` |
| G-REBUILD-3 | 4 binary components have `install: None` | Add 4 install fns wrapping `cargo build --release -p <pkg> + cp`. Use `install_template_forge_binaries` as the template |
| G-REBUILD-4 | When `--rebuild`, status-detection branch shouldn't short-circuit on "all healthy" | At line 385: skip the `gap_count == 0` early return when `rebuild` is set |
| G-REBUILD-5 | After binary rebuild, dependent launchd services need `kickstart -k` | After `forge` rebuild, `kickstart` `forge-mcp`; after `pk-cherry` rebuild, `kickstart` `pk-mcp` |

### G-CHECK: `--check` output separation
| ID | Gap | Action |
|----|-----|--------|
| G-CHECK-1 | `--check` lumps all gap-states together as "gap(s) detected" | Group output: "X missing, Y stale" so users can distinguish |
| G-CHECK-2 | `setup-state.json` records status but stale vs. missing not distinguishable post-hoc | Already fine — `ComponentStatus` is serialized; adding `Stale` variant is automatic |

### G-TEST: Unit tests for staleness
| ID | Gap | Action |
|----|-----|--------|
| G-TEST-1 | `is_stale` not tested | 3 cases: source newer → true; source older → false; equal → false |
| G-TEST-2 | `needs_action` doesn't cover `Stale` | Add assertion that `Stale.needs_action() == true` |
| G-TEST-3 | `label()` and `icon()` don't cover `Stale` | Add to existing iteration tests |

## 6. Recommended Phase Plan (preview for /kbd-plan)

| # | Change | Effort | Closes |
|---|--------|--------|--------|
| 1 | `add-stale-component-status` — enum variant, icon/label/needs_action, tests | 30 min | G-STALE-1, G-TEST-2,3 |
| 2 | `add-staleness-comparator` — `is_stale`, `binary_mtime`, `source_commit_time` + unit tests | 45 min | G-STALE-2,3,4, G-TEST-1 |
| 3 | `wire-staleness-into-detection` — update 4 detect fns to return `Stale` | 30 min | G-STALE-5 |
| 4 | `add-rebuild-flag-and-installers` — clap flag, run signature, 4 install fns, kickstart chain | 60–90 min | G-REBUILD-1..5 |
| 5 | `improve-check-output-grouping` — group missing vs stale in status table | 15 min | G-CHECK-1 |
| 6 | `verify-and-integration-test` — `setup --check` against current machine (should now show 0 stale), then touch a submodule + reprove `Stale`, then `--rebuild` + reprove `Ok` | 30 min | G-VERIFY (final gate) |

**Total**: ~3.5 hours, matches the 3-4 hour estimate.

Order is strict: 1 (enum) → 2 (comparator) → 3 (wire) → 4 (rebuild) → 5 (UX) → 6 (verify).
Changes 4 and 5 could parallelize but the sequential order keeps the code review simple.

## 7. Open Decisions (surface in plan, not assess)

1. **Source-time granularity for `prometheus`** — use `git log -1 -- tools/prometheus-cli/`
   (path-scoped) vs. repo HEAD time. Default: path-scoped (more accurate, avoids false-stale
   on unrelated commits).
2. **What about the `setup.rs` file itself?** When *only* `setup.rs` changes (no submodule
   advances), the `prometheus` binary is stale relative to itself. The path-scoped query
   catches this correctly. Confirm acceptable.
3. **Should `--rebuild` imply `--non-interactive`?** Or should the user still be prompted
   per-component? Default: `--rebuild` implies `--non-interactive` (the whole point is
   automation) but `--rebuild --interactive` (a possible future flag) could prompt.
4. **`kickstart` failure handling** — if a binary rebuilds but the kickstart fails (e.g.,
   launchd label not loaded), should the change error out or warn-and-continue? Default:
   warn (the install state file already records what succeeded; service restart is a soft step).

---

*Assessment written to: `.kbd-orchestrator/phases/setup-rebuild-and-staleness-2026-05-30/assessment.md`*
*Next: `/kbd-plan setup-rebuild-and-staleness-2026-05-30`*
