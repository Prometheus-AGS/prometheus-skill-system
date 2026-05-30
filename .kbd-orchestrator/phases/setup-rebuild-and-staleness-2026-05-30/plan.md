# Plan: setup-rebuild-and-staleness-2026-05-30

**Date**: 2026-05-30
**Backend**: OpenSpec (`openspec/` present, `change_backend: openspec`)
**Assessment**: `.kbd-orchestrator/phases/setup-rebuild-and-staleness-2026-05-30/assessment.md`
**Scope**: `tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs` + `main.rs` clap wiring + unit tests
**Estimated effort**: ~3.5 hours

## Locked decisions (from user, 2026-05-30)

1. **prometheus binary source time** → path-scoped: `git log -1 --format=%ct -- tools/prometheus-cli/`
2. **`--rebuild` implies `--non-interactive`** → no prompts
3. **`launchctl kickstart -k` failure** → warn and continue, record in `setup-state.json`
4. **`--rebuild` granularity** → all-or-nothing (rebuilds all 4 binaries unconditionally)

## Design summary

### New `ComponentStatus::Stale` variant
- `icon()` → ⚠️ yellow
- `label()` → `"stale (source newer than binary)"`
- `needs_action()` → `true`
- Serde-serialized automatically (snake_case "stale")

### Staleness comparator (pure)
```rust
fn is_stale(binary_mtime: SystemTime, source_commit_time: SystemTime) -> bool {
    source_commit_time > binary_mtime
}
```

### Source-of-truth dispatch (per binary)
```rust
// repo_root() returns the git repo root (already needed by install_template_forge_binaries
// — reuse that pattern via PROMETHEUS_SKILL_PACK_ROOT env or walking up from current exe).

fn source_commit_time_for(binary_id: &str) -> Option<SystemTime> {
    let root = repo_root()?;
    let (cwd, args) = match binary_id {
        "prometheus" => (root.clone(), vec!["log", "-1", "--format=%ct", "--", "tools/prometheus-cli/"]),
        "forge"      => (root.join("tools/forge-rs"), vec!["log", "-1", "--format=%ct", "HEAD"]),
        "pk-cherry"  => (root.join("tools/prometheus-knowledge"), vec!["log", "-1", "--format=%ct", "HEAD"]),
        "liter-llm"  => (root.join("tools/liter-llm"), vec!["log", "-1", "--format=%ct", "HEAD"]),
        _ => return None,
    };
    let out = Command::new("git").current_dir(cwd).args(args).output().ok()?;
    let secs: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(secs))
}

fn binary_mtime(name: &str) -> Option<SystemTime> {
    let path = which::which(name).ok()?; // already shelling `which`; cheaper: PathBuf from `which` output
    fs::metadata(path).ok()?.modified().ok()
}
```
> Note: we already shell `which` in `detect_binary`; we'll keep that pattern (`Command::new("which")`) rather than add a `which` crate dep, to honor "no new deps".

### Install functions (4 new, modeled on `install_template_forge_binaries`)
```rust
fn install_prometheus_cli() -> Result<()> {
    let root = repo_root()?;
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "prometheus-cli"])
        .current_dir(root.join("tools/prometheus-cli"))
        .status()?;
    anyhow::ensure!(status.success(), "cargo build failed for prometheus-cli");
    let src = root.join("tools/prometheus-cli/target/release/prometheus");
    let dst = bin_dir().join("prometheus");
    fs::copy(&src, &dst)?;
    Ok(())
}
// ... install_forge (forge-rs / forge-cli), install_pk_cherry, install_liter_llm (liter-llm-cli)
// ... with post-install kickstart for forge → forge-mcp, pk-cherry → pk-mcp
```

### `kickstart_or_warn` helper
```rust
fn kickstart_or_warn(label: &str) {
    let domain = format!("gui/{}", users::get_current_uid());
    let target = format!("{domain}/{label}");
    let status = Command::new("launchctl").args(["kickstart","-k",&target]).status();
    match status {
        Ok(s) if s.success() => println!("  kickstart {label}: ok"),
        Ok(s) => eprintln!("  ⚠ kickstart {label} exited {s}"),
        Err(e) => eprintln!("  ⚠ kickstart {label} failed: {e}"),
    }
}
```
> We need the current uid. Cheapest source: `nix` crate is overkill; just `Command::new("id").args(["-u"]).output()` or read `$UID`. Or even simpler: use `gui/$(id -u)` via shell — but we're avoiding shell. Final pick: shell out to `id -u` once (matches existing detection-by-shell pattern, no new dep).

### CLI change (`main.rs:184`)
```rust
Setup {
    #[arg(long)] non_interactive: bool,
    #[arg(long)] dry_run: bool,
    #[arg(long)] check: bool,
    /// Force rebuild of all binary components from source (implies --non-interactive)
    #[arg(long)] rebuild: bool,
},
```
And in the match arm at `main.rs:349`: `setup::run(non_interactive || rebuild, dry_run, check, rebuild)`.

### Run signature
```rust
pub fn run(non_interactive: bool, dry_run: bool, check: bool, rebuild: bool) -> Result<()>
```

### `--check` output grouping
```
Component Status
  ✅ forge-mcp ... ok
  ⚠️  liter-llm ... stale (source newer than binary)
  ❌ pk-cherry ... not installed
  ...

  2 gap(s): 1 missing, 1 stale.
```

---

## Ordered change list

| # | Change ID | Closes | Files | Effort | Depends on |
|---|-----------|--------|-------|--------|------------|
| 1 | `change-staleness-001-add-stale-component-status` | G-STALE-1, G-TEST-2, G-TEST-3 | setup.rs | 30 min | — |
| 2 | `change-staleness-002-add-staleness-comparator` | G-STALE-2, G-STALE-3, G-STALE-4, G-TEST-1 | setup.rs | 45 min | 001 |
| 3 | `change-staleness-003-wire-staleness-into-detection` | G-STALE-5 | setup.rs | 30 min | 002 |
| 4 | `change-staleness-004-add-rebuild-flag-and-installers` | G-REBUILD-1..5 | setup.rs + main.rs | 60–90 min | 003 |
| 5 | `change-staleness-005-improve-check-output-grouping` | G-CHECK-1, G-CHECK-2 | setup.rs | 15 min | 001 (variant must exist) |
| 6 | `change-staleness-006-verify-and-integration-test` | G-VERIFY (final) | (verification only) | 30 min | 001-005 |

Strict order: **1 → 2 → 3 → 4 → 5 → 6.** (5 could parallelize with 4 but the file overlap makes sequential merge cleaner.)

---

### change-staleness-001 — add Stale ComponentStatus variant
**Closes**: G-STALE-1, G-TEST-2, G-TEST-3
**Effort**: 30 min · **Files**: `setup.rs`

Tasks:
- [ ] Add `Stale` to the enum (after `NotInstalled`)
- [ ] Add `Stale` arm to `icon()` (yellow ⚠️ — `colored` already imported)
- [ ] Add `Stale` arm to `label()` → `"stale (source newer than binary)"`
- [ ] Add `Stale` to `needs_action()` → returns `true`
- [ ] Extend existing tests:
  - [ ] `component_status_needs_action_only_for_gap_states` adds `Stale`
  - [ ] `component_status_labels_are_non_empty` adds `Stale`
- [ ] `cargo test -p prometheus-cli` green

Acceptance:
- `Stale` variant exists, all three impl arms updated, 3 tests still pass + Stale assertions added.
- Serde round-trip: `serde_json::to_string(&Stale) == "\"stale\""` (snake_case).

---

### change-staleness-002 — staleness comparator + IO
**Closes**: G-STALE-2, G-STALE-3, G-STALE-4, G-TEST-1
**Effort**: 45 min · **Files**: `setup.rs`
**Depends on**: 001

Tasks:
- [ ] Add `fn is_stale(binary_mtime: SystemTime, source_commit_time: SystemTime) -> bool`
- [ ] Add `fn binary_mtime(name: &str) -> Option<SystemTime>` — `which` (reuse Command pattern) + `fs::metadata().modified()`
- [ ] Add `fn repo_root() -> Option<PathBuf>` — reuse the walk-up logic from `install_template_forge_binaries` (lines 184-204); extract to shared helper
- [ ] Add `fn source_commit_time_for(binary_id: &str) -> Option<SystemTime>` — dispatch on id, run `git log -1 --format=%ct ...` per-binary mapping (path-scoped for `prometheus`, `HEAD` for the 3 submodules)
- [ ] Unit tests for `is_stale`:
  - [ ] `is_stale_returns_true_when_source_newer`
  - [ ] `is_stale_returns_false_when_source_older`
  - [ ] `is_stale_returns_false_when_equal`
- [ ] `cargo test -p prometheus-cli` green
- [ ] `cargo clippy -- -D warnings` clean

Acceptance:
- 4 new functions exist, comparator has 3 pure-function tests, IO functions return Option (no panics on missing binary / unmapped id).

---

### change-staleness-003 — wire staleness into the 4 detect fns
**Closes**: G-STALE-5
**Effort**: 30 min · **Files**: `setup.rs`
**Depends on**: 002

Tasks:
- [ ] Update `detect_liter_llm`, `detect_prometheus_cli`, `detect_forge_bin`, `detect_pk_cherry`:
  - If binary missing → `NotInstalled` (existing behavior)
  - If binary present AND `source_commit_time_for(id)` returns Some(t) AND `is_stale(mtime, t)` → `Stale`
  - Else → `Installed`
- [ ] If `binary_mtime` or `source_commit_time_for` returns None (git not available, source dir missing), fall back to current behavior: report `Installed` (do NOT block on missing source — staleness is best-effort)
- [ ] `cargo test -p prometheus-cli` green
- [ ] Manual smoke: `cargo run -p prometheus-cli -- setup --check` — current machine should report all 4 as `installed` (no stale), since today's binaries are 06:27–06:38 mtimes and submodules haven't moved since

Acceptance:
- Running `setup --check` after this change reports 0 stale on a clean machine.
- Manually touching a submodule source file (e.g., `touch tools/liter-llm/crates/liter-llm-cli/src/main.rs` won't work — mtime needs to be in the *git* commit. Test by checking the binary mtime is newer than the submodule HEAD commit time → reports installed.

---

### change-staleness-004 — `--rebuild` flag + 4 install fns + kickstart chain
**Closes**: G-REBUILD-1, G-REBUILD-2, G-REBUILD-3, G-REBUILD-4, G-REBUILD-5
**Effort**: 60–90 min · **Files**: `setup.rs` + `main.rs`
**Depends on**: 003

Tasks:
- [ ] `main.rs:184`: add `#[arg(long)] rebuild: bool,` to `Setup` clap variant
- [ ] `main.rs:349`: thread `rebuild` into `setup::run(...)` (and OR with `non_interactive` per locked decision #2)
- [ ] `setup.rs`: update `pub fn run` signature to take `rebuild: bool` (4th param)
- [ ] Add 4 install functions in setup.rs:
  - `install_prometheus_cli()` — cargo build -p prometheus-cli + cp
  - `install_forge_cli()` — cargo build -p forge-cli + cp (mind the rename caught in machine-refresh!)
  - `install_pk_cherry()` — cargo build -p pk-cherry + cp
  - `install_liter_llm()` — cargo build -p liter-llm-cli + cp
- [ ] Wire them into the `Component { install: Some(...) }` slots for the 4 binary entries
- [ ] Add `kickstart_or_warn(label: &str)` helper — warn-and-continue on failure (locked decision #3)
- [ ] After `install_forge_cli`, call `kickstart_or_warn("dev.prometheusags.forge-mcp")`
- [ ] After `install_pk_cherry`, call `kickstart_or_warn("dev.prometheusags.pk-mcp")`
- [ ] In `run()`: when `rebuild` is true, skip the `gap_count == 0` early-return (lines 385-390) so the install loop runs even if all detect green
- [ ] In `run()`: when `rebuild` is true, treat ALL 4 binary components as needing-action (force their statuses to be installable in the install loop)
- [ ] `cargo build --release -p prometheus-cli` succeeds; `cargo test -p prometheus-cli` green
- [ ] Smoke: `prometheus setup --rebuild --dry-run` lists 4 rebuild actions; `prometheus setup --rebuild` actually rebuilds + kickstarts

Acceptance:
- `prometheus setup --rebuild --dry-run` shows what would rebuild without doing it
- `prometheus setup --rebuild` produces fresh mtimes on all 4 binaries + restarted services
- Kickstart failure produces a yellow warning, exits 0, state file records the partial state
- `--rebuild` implies non-interactive (no prompts even without `--non-interactive`)

---

### change-staleness-005 — group `--check` output by missing vs stale
**Closes**: G-CHECK-1, G-CHECK-2
**Effort**: 15 min · **Files**: `setup.rs`
**Depends on**: 001 (`Stale` variant must exist)

Tasks:
- [ ] In `run()`, split the gap counting:
  ```rust
  let missing_count = statuses.iter().filter(|(_,s)| matches!(s, Missing|NotInstalled)).count();
  let stale_count   = statuses.iter().filter(|(_,s)| matches!(s, Stale)).count();
  ```
- [ ] Update the gap-count print to:
  ```
  X gap(s): Y missing, Z stale.
  ```
  Color: missing red, stale yellow.
- [ ] G-CHECK-2 is automatically closed by serde — `Stale` will serialize/deserialize in `setup-state.json` without code change. Add an assertion in a test that round-trip works.

Acceptance:
- Output groups gaps. `setup-state.json` correctly persists Stale entries.

---

### change-staleness-006 — verify and integration test
**Closes**: G-VERIFY (final gate)
**Effort**: 30 min · **Files**: none (verification)
**Depends on**: 001-005

Tasks:
- [ ] Build & install the new `prometheus`: `bash scripts/install-binaries.sh` (which now contains the package-rename fix from prior phase)
- [ ] `prometheus setup --check` on current machine — should report 0 stale (binaries are fresh)
- [ ] **Create a synthetic stale**: `touch -t 202001010000 ~/.local/bin/liter-llm` (force ancient mtime). Re-run `setup --check` → should now report 1 stale.
- [ ] `prometheus setup --rebuild` → rebuilds liter-llm (and the other 3), kickstarts both MCP services
- [ ] `setup --check` again → 0 stale
- [ ] Inspect `~/.prometheus/setup-state.json` → 4 binary components all in `Installed`, last_checked is fresh
- [ ] `cargo test -p prometheus-cli` → all tests pass (original 3 + new ones)
- [ ] `cargo clippy -p prometheus-cli -- -D warnings` clean
- [ ] Write a 1-paragraph verification report in change progress

Acceptance:
- Synthetic stale detected, force-rebuild fixes it, both MCP services restarted, all tests green.

---

## OpenSpec emission

```
/opsx:new change-staleness-001-add-stale-component-status
/opsx:new change-staleness-002-add-staleness-comparator
/opsx:new change-staleness-003-wire-staleness-into-detection
/opsx:new change-staleness-004-add-rebuild-flag-and-installers
/opsx:new change-staleness-005-improve-check-output-grouping
/opsx:new change-staleness-006-verify-and-integration-test
```

Changes 001-004 touch source code (>3 files for 004) → **subject to artifact-refiner QA per
the KBD QA gate**. Changes 005 and 006 are smaller / verification-only and likely QA-skipped
by the file-count rule, but per the prior reflection that rule is on the wrong axis — we'll
opt-in 005 to QA anyway because it touches setup.rs UX.

## Next step

Run `/kbd-execute setup-rebuild-and-staleness-2026-05-30` to dispatch change-staleness-001.
Execute strictly in order 001 → 002 → 003 → 004 → 005 → 006.
