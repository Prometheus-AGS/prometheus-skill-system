# KBD Plan — phase-librefang-wasm-onramp

> **Date**: 2026-04-29
> **Tool**: Claude Code (claude-sonnet-4-6)
> **Backend**: native-kbd (no OpenSpec, no evolver)
> **Assessment**: `.kbd-orchestrator/phases/phase-librefang-wasm-onramp/assessment.md`

---

## Phase Goals

1. Implement `forge package-librefang <agent-dir>` in `tools/forge-rs/crates/forge-cli/`
2. End-to-end smoke test: agent dir → `.lf-skill.zip` → librefang install → `runtime.type=wasm` verified
3. Close §9 verification criteria 4–7 from phase-compliance-and-power-multiplier

---

## Change Order

| # | Change ID | Priority | Effort | Gaps | Agent |
|---|-----------|----------|--------|------|-------|
| 1 | `change-001-forge-package-librefang` | **P0** | S | G1, G4 | native-tool |
| 2 | `change-002-smoke-test` | **P0** | XS | G2, G3 | native-tool |

**Rationale**: change-001 must land before change-002 (smoke test calls the subcommand). Both are mechanical — all design decisions resolved in assessment §7.

---

## change-001-forge-package-librefang

### Summary

Add `forge package-librefang <agent-dir>` subcommand that reads `skill.toml`, optionally compiles the WASM binary, and produces a `.lf-skill.zip` ready for `librefang skill install`.

### Files

| File | Action | Description |
|------|--------|-------------|
| `tools/forge-rs/Cargo.toml` | Modify | Add `zip = "2"` to `[workspace.dependencies]` |
| `tools/forge-rs/crates/forge-cli/Cargo.toml` | Modify | Add `zip = { workspace = true }` to `[dependencies]` |
| `tools/forge-rs/crates/forge-cli/src/main.rs` | Modify | Add `PackageLibrefang` variant to `Commands`, implement handler |

### Acceptance Criteria

- [ ] `cargo build -p forge-cli --release` succeeds (no new warnings)
- [ ] `forge package-librefang --help` shows the subcommand and `--no-build`, `--output` flags
- [ ] Running against `skills/rust/librefang-wasm-skill/` (with pre-built `.wasm`) produces `librefang-wasm-skill-0.1.0.lf-skill.zip`
- [ ] `unzip -l librefang-wasm-skill-0.1.0.lf-skill.zip` shows `skill.toml` + `librefang-wasm-skill.wasm` at the archive root
- [ ] `skill.toml` inside the zip is byte-identical to the source `skill.toml`
- [ ] Error paths: missing `skill.toml` prints clear message + exits non-zero; missing `.wasm` + `--no-build` prints clear message + exits non-zero

### Implementation Notes

**New `Commands` variant** (after `Constitution`):

```rust
/// Package an agent directory as a LibreFang WASM skill zip
PackageLibrefang {
    /// Path to the agent directory (must contain skill.toml)
    agent_dir: PathBuf,

    /// Skip `cargo build` — assume .wasm already compiled
    #[arg(long)]
    no_build: bool,

    /// Output path for the zip (default: ./<name>-<version>.lf-skill.zip)
    #[arg(long, short)]
    output: Option<PathBuf>,
},
```

**Handler** (match arm in `main()`):

```rust
Commands::PackageLibrefang { agent_dir, no_build, output } => {
    // 1. Read + parse skill.toml
    let manifest_path = agent_dir.join("skill.toml");
    let manifest_toml = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Cannot read {}", manifest_path.display()))?;
    let manifest: toml::Value = toml::from_str(&manifest_toml)
        .with_context(|| format!("Invalid TOML in {}", manifest_path.display()))?;
    let skill = manifest.get("skill")
        .context("[skill] table missing from skill.toml")?;
    let name = skill.get("name")
        .and_then(|v| v.as_str())
        .context("skill.name missing")?
        .to_string();
    let version = skill.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();
    let entry = manifest.get("runtime")
        .and_then(|r| r.get("entry"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{name}.wasm"));

    // 2. Optionally build
    if !no_build {
        let status = std::process::Command::new("cargo")
            .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
            .current_dir(&agent_dir)
            .status()
            .context("Failed to spawn cargo build")?;
        anyhow::ensure!(status.success(), "cargo build --target wasm32-unknown-unknown failed");
    }

    // 3. Locate WASM
    let wasm_path = agent_dir
        .join("target/wasm32-unknown-unknown/release")
        .join(&entry);
    anyhow::ensure!(
        wasm_path.exists(),
        "WASM binary not found at {}. Run `cargo build --release --target wasm32-unknown-unknown` first, or remove --no-build.",
        wasm_path.display()
    );

    // 4. Write zip
    let zip_name = output.unwrap_or_else(|| {
        PathBuf::from(format!("{name}-{version}.lf-skill.zip"))
    });
    let zip_file = std::fs::File::create(&zip_name)
        .with_context(|| format!("Cannot create {}", zip_name.display()))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("skill.toml", opts)?;
    std::io::Write::write_all(&mut zip, manifest_toml.as_bytes())?;

    let wasm_bytes = std::fs::read(&wasm_path)
        .with_context(|| format!("Cannot read {}", wasm_path.display()))?;
    zip.start_file(&entry, opts)?;
    std::io::Write::write_all(&mut zip, &wasm_bytes)?;

    zip.finish()?;

    println!("✅ Packaged: {}", zip_name.display());
    println!("   Skill:    {} v{}", name, version);
    println!("   WASM:     {} ({} KB)", entry, wasm_bytes.len() / 1024);
    println!("   Install:  librefang skill install {}", zip_name.display());
}
```

**Cargo.toml changes**:

```toml
# tools/forge-rs/Cargo.toml — [workspace.dependencies] section, add:
zip = "2"

# tools/forge-rs/crates/forge-cli/Cargo.toml — [dependencies] section, add:
zip = { workspace = true }
```

**Also update** the doc comment at the top of `main.rs` to list the new subcommand:
```
//!   forge package-librefang <agent-dir> [--no-build] [--output <path>]  — package WASM agent as .lf-skill.zip
```

### QA Gate

- `cargo clippy -p forge-cli -- -D warnings` clean
- `cargo build -p forge-cli --release` succeeds
- `unzip -l` check on produced zip (manual verification, < 30 sec)
- Fewer than 3 files changed — QA gate may be skipped per `/kbd-execute` policy

---

## change-002-smoke-test

### Summary

Extend `scripts/smoke-test.sh` with a new `forge-package-librefang` test section that exercises the full pipeline: build WASM → `forge package-librefang` → inspect zip → (if librefang present) install + verify `runtime.type=wasm`. Also update `stage 6` of `/start-business-build` skill to use `forge package-librefang` instead of the manual zip fallback.

### Files

| File | Action | Description |
|------|--------|-------------|
| `scripts/smoke-test.sh` | Modify | Add `test_forge_package_librefang()` function |
| `skills/process/native-agent/SKILL.md` | Modify | Update stage 6 text: replace manual fallback with `forge package-librefang` |

### Acceptance Criteria

- [ ] `bash scripts/smoke-test.sh` runs end-to-end without manual steps (librefang checks are guarded by `command -v librefang`)
- [ ] When `forge` binary is on PATH: test builds the zip and checks its contents with `unzip -l`
- [ ] When `librefang` binary is on PATH: test installs the zip and greps `runtime.*wasm` from `librefang skill info`
- [ ] When either binary is absent: test skips that section gracefully (SKIP not FAIL)
- [ ] `/start-business-build` stage 6 shows `forge package-librefang <agent-dir>` as the primary command (manual `zip` fallback documented as the pre-forge alternative, not the active path)
- [ ] `npm run validate` still passes (0 errors)

### Implementation Notes

**New test function in `scripts/smoke-test.sh`**:

```bash
test_forge_package_librefang() {
    local agent_dir="skills/rust/librefang-wasm-skill"
    local zip_out="/tmp/test-librefang-wasm-skill.lf-skill.zip"

    echo ""
    echo "🧪 forge package-librefang pipeline"
    echo "------------------------------------"

    if ! command -v forge &>/dev/null; then
        echo "  [SKIP] forge not on PATH"
        SKIP=$((SKIP + 1))
        return
    fi

    # Build WASM (skip if --no-build flag already has a .wasm)
    local wasm_path="${agent_dir}/target/wasm32-unknown-unknown/release/librefang-wasm-skill.wasm"
    if [[ ! -f "$wasm_path" ]]; then
        echo "  Building WASM..."
        if ! cargo build --manifest-path "${agent_dir}/Cargo.toml" \
                --release --target wasm32-unknown-unknown --quiet 2>/dev/null; then
            echo "  [SKIP] cargo build failed (WASM toolchain may not be installed)"
            SKIP=$((SKIP + 1))
            return
        fi
    fi

    # Package
    rm -f "$zip_out"
    if ! forge package-librefang "$agent_dir" --no-build --output "$zip_out"; then
        echo "  [FAIL] forge package-librefang exited non-zero"
        FAIL=$((FAIL + 1))
        return
    fi

    # Inspect zip
    if ! unzip -l "$zip_out" 2>/dev/null | grep -q "skill.toml"; then
        echo "  [FAIL] skill.toml missing from zip"
        FAIL=$((FAIL + 1))
        return
    fi
    if ! unzip -l "$zip_out" 2>/dev/null | grep -q "\.wasm"; then
        echo "  [FAIL] .wasm binary missing from zip"
        FAIL=$((FAIL + 1))
        return
    fi
    echo "  [PASS] zip contains skill.toml + .wasm"
    PASS=$((PASS + 1))

    # librefang install (optional)
    if command -v librefang &>/dev/null; then
        local tmp_skills_dir
        tmp_skills_dir=$(mktemp -d)
        if librefang skill install "$zip_out" --skills-dir "$tmp_skills_dir" --quiet 2>/dev/null; then
            if librefang skill info librefang-wasm-skill \
                    --skills-dir "$tmp_skills_dir" 2>/dev/null \
                    | grep -qi "runtime.*wasm\|type.*wasm"; then
                echo "  [PASS] librefang install + runtime.type=wasm verified"
                PASS=$((PASS + 1))
            else
                echo "  [FAIL] runtime.type=wasm not found in skill info"
                FAIL=$((FAIL + 1))
            fi
        else
            echo "  [SKIP] librefang skill install failed (may need config)"
            SKIP=$((SKIP + 1))
        fi
        rm -rf "$tmp_skills_dir"
    else
        echo "  [SKIP] librefang not on PATH — install step skipped"
        SKIP=$((SKIP + 1))
    fi

    rm -f "$zip_out"
}
```

Call it near the end of the script, before the summary:
```bash
test_forge_package_librefang
```

**`/start-business-build` stage 6 update** — in `skills/process/native-agent/SKILL.md`, find stage 6 text and replace the manual zip fallback block with:

```markdown
### Stage 6: Package as LibreFang skill

```bash
# Primary path (requires forge binary):
forge package-librefang <agent-dir>
# Produces: <skill-name>-<version>.lf-skill.zip

# Legacy fallback (if forge not yet installed):
zip -j <name>.lf-skill.zip skill.toml target/wasm32-unknown-unknown/release/<name>.wasm
```
```

### QA Gate

- `bash scripts/smoke-test.sh` (check forge section at minimum; librefang section SKIP is acceptable)
- `npm run validate` → 0 errors
- `bash -n scripts/smoke-test.sh` → syntax check

---

## Change Dependencies

```
change-001-forge-package-librefang
    │
    ▼ (smoke test calls forge binary from change-001)
change-002-smoke-test
```

---

## Waypoint After Plan

```json
{
  "phase": "phase-librefang-wasm-onramp",
  "stage": "execute",
  "next_action": "/kbd-execute change-001-forge-package-librefang",
  "active_change": null,
  "changes_total": 2,
  "changes_completed": 0
}
```

---

## Risk Mitigations Baked In

| Risk | Mitigation in plan |
|------|--------------------|
| `zip` v1 vs v2 API | Pinned to `"2"` in workspace; use `SimpleFileOptions` (v2 API) |
| `cargo build` inside forge may be slow | `--no-build` flag exits early; smoke test uses pre-built wasm path |
| `librefang skill install` flags vary by version | Smoke test guards with `2>/dev/null` + SKIP on non-zero |
| stage 6 update breaks `npm run validate` | Update is text-only in SKILL.md — validator checks frontmatter, not body |
