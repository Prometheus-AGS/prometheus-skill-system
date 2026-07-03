---
id: change-credibility-008-unit-tests
title: Fix Unicode doctest and add ≥15 unit tests to forge-rs
phase: phase-credibility-closure
priority: P2
effort: M
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-A
verdict: BUILD
scope:
  - tools/forge-rs/crates/forge-enricher/src/lib.rs
  - tools/forge-rs/crates/forge-core/src/lib.rs
  - tools/forge-rs/crates/forge-reflect/src/lib.rs
  - tools/forge-rs/crates/forge-skills/src/lib.rs
---

# change-credibility-008 — Fix Unicode doctest and add ≥15 unit tests

## Context

The `forge-enricher/src/lib.rs:16` module-level doc comment uses the Unicode arrow `→` which Rust's doctest runner cannot parse, causing `cargo test` to fail on the doctests. Beyond this, the entire forge-rs workspace has zero unit tests — no `#[test]` functions anywhere.

This change fixes the doctest and adds at least 15 unit tests covering the core logic in forge-enricher and forge-core.

## Scope

1. Replace `→` with `->` in `forge-enricher/src/lib.rs:16` doc comment
2. Add ≥15 `#[cfg(test)]` unit test functions covering:
   - `forge-enricher`: skill loading, constitution checking, drift path parsing
   - `forge-core`: task model construction, language detection
   - `forge-reflect`: metric extraction
   - `forge-skills`: skill registry operations

## Implementation Notes

```rust
// forge-enricher/src/lib.rs — fix Unicode:
// Change: //! task enrichment pipeline: task -> skills -> constitution check
// Was:    //! task enrichment pipeline: task → skills → constitution check

// Example unit tests to add:
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_detection_rust() {
        let lang = Language::from_extension("rs").unwrap();
        assert_eq!(lang, Language::Rust);
    }
    
    #[test]
    fn test_language_detection_typescript() {
        let lang = Language::from_extension("ts").unwrap();
        assert_eq!(lang, Language::TypeScript);
    }
    
    #[test]
    fn test_language_detection_unknown_returns_err() {
        let result = Language::from_extension("xyz");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_constitution_check_no_violations_on_empty_content() {
        let constitution = Constitution::default();
        let warnings = check_constitution(&constitution, "");
        assert!(warnings.is_empty());
    }
    
    #[test]
    fn test_constitution_check_returns_warning_for_violation() {
        // use a constitution with a known rule and content that triggers it
    }
    
    #[test]
    fn test_skill_registry_empty_by_default() {
        let registry = SkillRegistry::new();
        assert_eq!(registry.len(), 0);
    }
    
    #[test]
    fn test_task_construction_sets_language() {
        let task = Task::new("do something").with_language(Language::Rust);
        assert_eq!(task.language(), Some(&Language::Rust));
    }
    
    // ... (8+ more tests covering drift path parsing, skill loading,
    //      reflect metric extraction, edge cases)
}
```

Exact test content is adapted to the real API discovered during implementation; minimum 15 tests across the workspace with all passing.

## Verification

- `cargo test --workspace` passes (all tests green, no doctest failures)
- Test count ≥ 15 confirmed with `cargo test --workspace 2>&1 | grep "test result"`
