# KBD Assessment — phase-librefang-wasm-onramp

> **Date**: 2026-04-29
> **Tool**: Claude Code (claude-sonnet-4-6)
> **Phase goals**: Implement `forge package-librefang`, smoke test end-to-end, close §9 criteria 4–7

---

## §1 Phase Goal Recap

| # | Goal |
|---|------|
| 1 | Implement `forge package-librefang <agent-dir>` in `tools/forge-rs/crates/forge-cli/` |
| 2 | End-to-end smoke test: agent dir → `.lf-skill.zip` → librefang install → `runtime.type=wasm` |
| 3 | Close assessment §9 verification criteria 4–7 (from phase-compliance-and-power-multiplier) |

---

## §2 Codebase State

### forge-rs workspace

| Crate | Path | Relevant state |
|-------|------|----------------|
| `forge-cli` | `tools/forge-rs/crates/forge-cli/src/main.rs` | 9 subcommands; **no** `PackageLibrefang` |
| `forge-core` | `tools/forge-rs/crates/forge-core/src/lib.rs` | Domain types: `SkillManifest`, `Language`, `SkillTrigger` |
| `forge-skills` | `tools/forge-rs/crates/forge-skills/src/lib.rs` | `SkillRegistry::load()`, `resolve()` — no packaging logic |
| workspace `Cargo.toml` | `tools/forge-rs/Cargo.toml` | `toml`, `tera`, `walkdir`, `serde_json` present; **no `zip` crate** |

### LibreFang skill format (from `references/librefang/`)

`librefang-skills` expects a zip archive containing:
- `skill.toml` — TOML manifest deserialized into `SkillManifest` struct
- `<skill_name>.wasm` — the WASM binary at the archive root

`SkillManifest` key fields for a WASM skill:
```toml
[skill]
name = "my-skill"
version = "0.1.0"

[runtime]
type = "wasm"
entry = "my-skill.wasm"

[[tools]]
name = "run"
description = "..."
input_schema = { ... }
```

ClawHub's `install_from_bytes()` detects zip by magic bytes and extracts all entries. No special `.lf-skill.zip` envelope — it is a standard zip.

### Native-agent skill template

`skills/process/native-agent/SKILL.md` + `skills/rust/librefang-wasm-skill/` provide the source-side templates (Cargo, `src/lib.rs`, `skill.toml.tera`). These are **complete** and **validated** (82 KB echo.wasm proven in change-003).

### `/start-business-build` stage 6

Stage 6 prints a manual fallback when `forge package-librefang` is absent:
```bash
# Manual fallback (used until forge package-librefang is implemented):
zip -j <name>.lf-skill.zip skill.toml target/wasm32-unknown-unknown/release/<name>.wasm
librefang skill install <name>.lf-skill.zip
```
This fallback is the *exact* operation the subcommand must automate.

---

## §3 Gap Analysis

### G1 — `forge package-librefang` subcommand (CRITICAL, P0)

**What's missing**: `Commands::PackageLibrefang { agent_dir: PathBuf }` in `forge-cli/src/main.rs`.

**What it must do**:
1. Read `<agent_dir>/skill.toml` → deserialize `SkillManifest` (name, version, runtime.entry)
2. Locate WASM binary: `<agent_dir>/target/wasm32-unknown-unknown/release/<name>.wasm`
   - If not present, run `cargo build --release --target wasm32-unknown-unknown` in `<agent_dir>` (or accept `--no-build` flag)
3. Create `<name>-<version>.lf-skill.zip` in cwd (or `--output <path>`) containing:
   - `skill.toml` (manifest, at archive root)
   - `<name>.wasm` (binary, at archive root — matches `runtime.entry`)
4. Print path to the produced zip

**Dependencies to add**:
- `zip = "2"` (workspace dep — currently absent; librefang itself uses `zip` 2.x)

**Implementation location**: `forge-cli/src/main.rs` (new `Commands` variant + handler) + `forge-skills` (optional: new `packaging.rs` module for reuse)

**Effort**: S (≤ 150 lines Rust + Cargo.toml change)

---

### G2 — End-to-end smoke test script (P0)

**What's missing**: A runnable smoke test that verifies the full pipeline without manual steps.

**What it must do**:
1. `cd skills/rust/librefang-wasm-skill/` → render templates → `cargo build --target wasm32-unknown-unknown --release`
2. `forge package-librefang .` → produces `librefang-wasm-skill-0.1.0.lf-skill.zip`
3. `librefang skill install librefang-wasm-skill-0.1.0.lf-skill.zip`
4. `librefang skill list | grep librefang-wasm-skill` → verify installed
5. `librefang skill info librefang-wasm-skill | grep 'runtime.*wasm'` → confirm `runtime.type=wasm`

**Implementation location**: `scripts/smoke-test.sh` (untracked file already exists in repo root — extend or replace)

**Effort**: XS (≤ 50 lines bash)

---

### G3 — Assessment §9 criteria 4–7 closure (P1, depends on G1+G2)

| Criterion | Current | After G1+G2 |
|-----------|---------|-------------|
| 4. `forge package-librefang` → `.lf-skill.zip` | NOT MET | MET |
| 5. librefang install succeeds | Pending (manual fallback) | MET |
| 6. manifest check: `runtime.type=wasm` | Pending | MET |
| 7. `/start-business-build` full chain < 10 min | Partial (stage 6 manual) | MET — stage 6 automated |

---

### G4 — `zip` workspace dep documentation (P2)

The `zip` crate needs to be added to `tools/forge-rs/Cargo.toml` workspace deps and listed in `forge-cli/Cargo.toml`. This is trivial but must be correct to avoid build failures.

---

## §4 What Is Already Complete

| Item | Status |
|------|--------|
| LibreFang WASM skill templates (Cargo, src/lib.rs, skill.toml.tera) | ✅ Complete (change-003) |
| echo.wasm proven build (82 KB, all ABI exports) | ✅ Verified (change-003) |
| native-agent `/package-as-librefang-skill` skill | ✅ Complete (change-004) |
| `/start-business-build` orchestrator (stages 1–6) | ✅ Complete (change-005) |
| Upload script + marketplace packaging | ✅ Complete (change-005) |
| forge-rs workspace, crates, build tooling | ✅ Buildable |
| wasm32-unknown-unknown target registration | ✅ Complete (change-002) |
| `scripts/smoke-test.sh` stub | ✅ Exists (untracked) |

---

## §5 Risk Register

| Risk | Severity | Mitigation |
|------|----------|-----------|
| `zip` crate API differs between v1 and v2 | Medium | Pin to `"2"` (matches librefang's own dep); use `zip::write::ZipWriter` |
| `cargo build` step inside `forge` may be slow | Low | Make build optional via `--no-build` flag; document that templates require pre-built WASM |
| librefang CLI not in PATH on test machine | Medium | Smoke test guards with `command -v librefang` check + clear error message |
| `skill.toml` TOML schema mismatch vs SkillManifest | Low | Template proven in change-003; deserialize with `toml::from_str::<SkillManifest>` in the subcommand to validate before zipping |

---

## §6 Prioritized Change Plan

| Change | Priority | Effort | Gaps closed |
|--------|----------|--------|-------------|
| `change-001-forge-package-librefang` | **P0** | S | G1, G4 |
| `change-002-smoke-test` | **P0** | XS | G2 |
| `change-003-assessment-9-close` | P1 | XS | G3 (verification run + §9 update) |

**Total**: 2–3 changes, all mechanical. No architectural decisions deferred.

---

## §7 Implementation Blueprint for change-001

### Files to touch

```
tools/forge-rs/Cargo.toml                          # add zip = "2" to [workspace.dependencies]
tools/forge-rs/crates/forge-cli/Cargo.toml          # add zip = { workspace = true }
tools/forge-rs/crates/forge-cli/src/main.rs          # new Commands variant + handler
```

### New `Commands` variant

```rust
/// Package an agent directory as a LibreFang WASM skill zip
PackageLibrefang {
    /// Path to agent directory (must contain skill.toml and a compiled .wasm)
    agent_dir: PathBuf,

    /// Skip cargo build — assume .wasm already exists
    #[arg(long)]
    no_build: bool,

    /// Output path for the zip (default: ./<name>-<version>.lf-skill.zip)
    #[arg(long, short)]
    output: Option<PathBuf>,
},
```

### Handler logic (≈ 80 lines)

```rust
Commands::PackageLibrefang { agent_dir, no_build, output } => {
    // 1. Read skill.toml
    let manifest_path = agent_dir.join("skill.toml");
    let manifest_toml = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Cannot read {}", manifest_path.display()))?;
    // Parse enough to get name/version — use a local thin struct or serde_json::Value
    let manifest: toml::Value = toml::from_str(&manifest_toml)?;
    let skill = manifest.get("skill").context("[skill] table missing")?;
    let name = skill["name"].as_str().context("skill.name missing")?.to_string();
    let version = skill.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();
    let entry = manifest
        .get("runtime")
        .and_then(|r| r.get("entry"))
        .and_then(|e| e.as_str())
        .unwrap_or_else(|| Box::leak(format!("{name}.wasm").into_boxed_str()));

    // 2. Optionally build
    if !no_build {
        let status = std::process::Command::new("cargo")
            .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
            .current_dir(&agent_dir)
            .status()
            .context("Failed to run cargo build")?;
        anyhow::ensure!(status.success(), "cargo build failed");
    }

    // 3. Find the WASM binary
    let wasm_path = agent_dir
        .join("target/wasm32-unknown-unknown/release")
        .join(entry);
    anyhow::ensure!(wasm_path.exists(), "WASM not found: {}", wasm_path.display());

    // 4. Write zip
    let zip_name = output.unwrap_or_else(|| {
        PathBuf::from(format!("{name}-{version}.lf-skill.zip"))
    });
    let zip_file = std::fs::File::create(&zip_name)?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let opts = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("skill.toml", opts)?;
    std::io::Write::write_all(&mut zip, manifest_toml.as_bytes())?;

    zip.start_file(entry, opts)?;
    let wasm_bytes = std::fs::read(&wasm_path)?;
    std::io::Write::write_all(&mut zip, &wasm_bytes)?;

    zip.finish()?;
    println!("✅ Packaged: {}", zip_name.display());
    println!("   Skill: {name} v{version}");
    println!("   WASM:  {} ({} KB)", entry, wasm_bytes.len() / 1024);
    println!("   Install: librefang skill install {}", zip_name.display());
}
```

---

## §8 Verification Criteria (new phase)

| # | Check | Method |
|---|-------|--------|
| 1 | `cargo build -p forge-cli --release` succeeds | CI / local |
| 2 | `forge package-librefang skills/rust/librefang-wasm-skill` produces `librefang-wasm-skill-0.1.0.lf-skill.zip` | `ls *.lf-skill.zip` |
| 3 | Zip contains `skill.toml` + `librefang-wasm-skill.wasm` | `unzip -l *.lf-skill.zip` |
| 4 | `librefang skill install` exits 0 | `echo $?` |
| 5 | `librefang skill info librefang-wasm-skill` shows `runtime.type: wasm` | grep |
| 6 | `/start-business-build` stage 6 uses `forge package-librefang` (not manual fallback) | script diff |
| 7 | `scripts/smoke-test.sh` passes end-to-end | `bash scripts/smoke-test.sh` |

---

## §9 Outstanding §9 Verification Criteria (from prior phase)

| Criterion | Status after this phase |
|-----------|------------------------|
| 1. `npm run validate` green | ✅ Already met |
| 2. `check-prerequisites.sh --install --build-tools` exits 0 | Pending remote env |
| 3. `/create-native-agent --target librefang-wasm` builds | Pending end-to-end (templates proven) |
| **4. `forge package-librefang` → `.lf-skill.zip`** | **Closed by change-001** |
| **5. librefang install succeeds** | **Closed by change-002 smoke test** |
| **6. manifest check: `runtime.type=wasm`** | **Closed by change-002 smoke test** |
| **7. `/start-business-build` full chain < 10 min** | **Closed by change-001 + change-002** |

---

## §10 Assessment Verdict

**This phase is narrow and achievable in 2 changes** (≈ 230 lines total). All design decisions are resolved. No blockers.

**Proceed to plan.**
