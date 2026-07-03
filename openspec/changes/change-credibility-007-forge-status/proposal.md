---
id: change-credibility-007-forge-status
title: Add forge status command and label stub commands as experimental
phase: phase-credibility-closure
priority: P1
effort: S
wave: 2
agent: claude
status: done
gap_id: P1-C
verdict: BUILD
scope:
  - tools/forge-rs/crates/forge-cli/src/main.rs
---

# change-credibility-007 — Add forge status command and label stub commands as experimental

## Context

`forge optimize`, `forge evolve`, and `forge generate` are print-only stubs with no implementation. They print a success-looking message without doing anything, making it hard for users to tell which capabilities are active vs gated. There is also no `forge status` command to get a diagnostic view of the system.

## Scope

1. Add `forge status` subcommand showing: constitution files, skill count, drift reports, pk_mcp_url status, active vs gated features
2. Prefix all stub commands with `[EXPERIMENTAL]` in their output and help text
3. Make stubs explicitly warn that the feature requires a feature flag or external service

## Implementation Notes

`forge-cli/src/main.rs` — add `Status` variant to `Commands` enum:
```rust
/// Show forge configuration and service status
Status,
```

Status handler:
```rust
Commands::Status => {
    let project_root = &cli.project_root;
    let forge_dir = project_root.join(".forge");
    
    println!("forge status\n");
    
    // Constitution files
    let constitution_dir = forge_dir.join("constitution");
    let constitution_count = if constitution_dir.exists() {
        std::fs::read_dir(&constitution_dir)
            .map(|d| d.flatten().count())
            .unwrap_or(0)
    } else { 0 };
    println!("Constitutions:  {} file(s) in {}", constitution_count, constitution_dir.display());
    
    // Skills
    let skills_count = /* count skill manifests */ 0;
    println!("Skills:         {} loaded", skills_count);
    
    // Drift reports
    let drift_dir = forge_dir.join("memory").join("drift");
    let drift_count = if drift_dir.exists() {
        std::fs::read_dir(&drift_dir).map(|d| d.flatten().count()).unwrap_or(0)
    } else { 0 };
    println!("Drift reports:  {} file(s)", drift_count);
    
    // PK MCP URL
    match &cli.pk_mcp_url {
        Some(url) => println!("PK MCP URL:     {} [configured]", url),
        None => println!("PK MCP URL:     [not configured] — `optimize`, `generate`, `evolve` gated"),
    }
    
    println!("\nActive features:");
    println!("  enrich      YES — forge enrich <task>");
    println!("  reflect     YES — forge reflect <iteration-id>");
    println!("  validate    YES — forge validate <file> --language <lang>");
    println!("  drift       YES — forge drift");
    println!("  optimize    [EXPERIMENTAL] — requires pk_mcp_url");
    println!("  generate    [EXPERIMENTAL] — requires pk_mcp_url");
    println!("  evolve      [EXPERIMENTAL] — requires pk_mcp_url");
}
```

Update stub commands to prepend `[EXPERIMENTAL - requires pk_mcp_url]`:
```rust
Commands::Optimize { .. } => {
    eprintln!("[EXPERIMENTAL] forge optimize requires a running prometheus-knowledge MCP server.");
    eprintln!("Set --pk-mcp-url to enable. See forge status for current configuration.");
}
```

## Verification

- `forge status` prints constitution count, skill count, drift reports, pk_mcp_url status
- `forge optimize` prints [EXPERIMENTAL] prefix, not a fake success message
- `cargo build --workspace` clean
