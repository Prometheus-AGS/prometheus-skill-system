---
id: change-credibility-004-path-confinement
title: Canonicalize and confine task_path in forge_enrich handler
phase: phase-credibility-closure
priority: P0
effort: S
wave: 1
agent: claude
status: done
gap_id: P0-D
verdict: BUILD
scope:
  - tools/forge-rs/crates/forge-mcp/src/lib.rs
---

# change-credibility-004 — Canonicalize and confine task_path in forge_enrich handler

## Context

The `forge_enrich` MCP tool handler at `forge-mcp/src/lib.rs:171-184` accepts `task_path` as a string and passes it directly to `Path::new(task_path)` with no canonicalization or prefix confinement:

```rust
let enricher = forge_enricher::Enricher::new(&state.skills_root, &state.project_root, ...)?;
let ctx = enricher.enrich(std::path::Path::new(task_path)).await?;
```

This allows path traversal: `../../etc/passwd` would be passed to the enricher and read from disk. This is a file-read primitive when combined with the (now fixed) unauthenticated 0.0.0.0 binding.

## Scope

1. Add `Path::canonicalize()` call on `task_path` before passing to enricher
2. Add prefix check: reject paths outside `state.project_root`
3. Return a clear JSON-RPC error for invalid paths

## Implementation Notes

In the `forge_enrich` match arm:
```rust
"forge_enrich" => {
    let task_path_str = args
        .and_then(|a| a["task_path"].as_str())
        .ok_or_else(|| anyhow::anyhow!("task_path required"))?;

    let raw_path = std::path::Path::new(task_path_str);
    let canonical = raw_path.canonicalize()
        .map_err(|e| anyhow::anyhow!("invalid task_path '{}': {}", task_path_str, e))?;
    let project_root_canonical = state.project_root.canonicalize()?;
    if !canonical.starts_with(&project_root_canonical) {
        return Err(anyhow::anyhow!(
            "task_path '{}' is outside the project root '{}'",
            canonical.display(),
            project_root_canonical.display()
        ));
    }

    let enricher = forge_enricher::Enricher::new(
        &state.skills_root,
        &state.project_root,
        state.pk_mcp_url.clone(),
    )?;
    let ctx = enricher.enrich(&canonical).await?;
    // ...
}
```

Note: `canonicalize()` requires the path to exist. This is correct: `forge_enrich` requires the task folder to exist. A missing path returns a clear error.

## Verification

- `cargo build --workspace` clean
- `task_path = "../../etc/passwd"` → returns JSON-RPC error, not file content
- `task_path = "/tmp/outside"` → returns JSON-RPC error
- Valid task path within project root → works normally
