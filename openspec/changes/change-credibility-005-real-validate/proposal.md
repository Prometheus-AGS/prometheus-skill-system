---
id: change-credibility-005-real-validate
title: Wire forge validate to call ConstitutionChecker
phase: phase-credibility-closure
priority: P1
effort: M
wave: 2
agent: claude
status: done
gap_id: P1-A
verdict: BUILD
scope:
  - tools/forge-rs/crates/forge-cli/src/main.rs
  - tools/forge-rs/crates/forge-enricher/src/lib.rs
  - tools/forge-rs/crates/forge-mcp/src/lib.rs
---

# change-credibility-005 — Wire forge validate to call ConstitutionChecker

## Context

`forge-cli/src/main.rs:210-219` — the `Validate` command reads a file, prints its character count, and prints "Validation complete (constitution checks applied)." without calling any checker. The `ConstitutionChecker` already exists in `forge-enricher` but is not exported or called from the CLI.

This is the highest-return credibility fix: the headline validate feature is completely fake.

## Scope

1. Make `forge_enricher::load_constitutions` and `check_constitution` `pub`
2. Wire them into the `Commands::Validate` arm in `forge-cli/src/main.rs`
3. Exit with code 1 on `Error`-severity violations
4. Fix the same stub in `forge_validate` MCP tool in `forge-mcp/src/lib.rs`

## Implementation Notes

`forge-enricher/src/lib.rs` — change visibility:
```rust
pub fn load_constitutions(dir: &Path) -> anyhow::Result<HashMap<Language, Constitution>> { ... }
pub fn check_constitution(constitution: &Constitution, content: &str) -> Vec<ConstitutionWarning> { ... }
```

`forge-cli/src/main.rs` Validate arm:
```rust
Commands::Validate { file, language } => {
    let content = std::fs::read_to_string(&file)
        .with_context(|| format!("cannot read {}", file.display()))?;
    let constitution_dir = cli.project_root.join(".forge").join("constitution");
    let lang = Language::from_str(&language)
        .map_err(|_| anyhow::anyhow!("unknown language: {}", language))?;
    
    let warnings = if constitution_dir.exists() {
        let constitutions = forge_enricher::load_constitutions(&constitution_dir)?;
        constitutions.get(&lang)
            .map(|c| forge_enricher::check_constitution(c, &content))
            .unwrap_or_default()
    } else {
        println!("No constitution found at {}; skipping checks.", constitution_dir.display());
        vec![]
    };
    
    if warnings.is_empty() {
        println!("Validation complete — no constitution violations.");
    } else {
        for w in &warnings {
            println!("[{:?}] {}: {}", w.severity, w.rule, w.occurrence);
        }
        if warnings.iter().any(|w| matches!(w.severity, Severity::Error)) {
            std::process::exit(1);
        }
    }
}
```

`forge-mcp/src/lib.rs` `forge_validate` tool: call `check_constitution` and return violations in the response.

## Verification

- `cargo build --workspace` clean
- `forge validate <file> --language rust` with no constitution → reports "No constitution found"
- `forge validate <file> --language rust` with constitution + violation → prints warning, exits 1 on Error severity
- `forge validate <file> --language rust` with clean file → "Validation complete"
