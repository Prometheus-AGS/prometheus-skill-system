---
id: change-credibility-006-drift-readback
title: Wire drift data read-back in Enricher::enrich() (Phase A)
phase: phase-credibility-closure
priority: P1
effort: M
wave: 2
agent: claude
status: done
gap_id: P1-B
verdict: BUILD
scope:
  - tools/forge-rs/crates/forge-enricher/src/lib.rs
  - tools/forge-rs/crates/forge-core/src/lib.rs
---

# change-credibility-006 — Wire drift data read-back in Enricher::enrich() (Phase A)

## Context

The self-improving loop in forge-rs is architecturally open: `forge-reflect` writes drift data to `.forge/memory/drift/<lang>-YYYYMMDD.json`, but `forge-enricher` never reads these files. The enrichment pipeline proceeds without any awareness of which skills have been repeatedly overridden by users.

Phase A (this change): read drift data before `resolve()`, log stale skills (acceptance_rate < 0.5) as warnings to the user. The feedback circuit is closed at the logging level; Phase B (follow-on, not this change) will use the data to deprioritize stale skills in resolution.

## Scope

1. Add `load_stale_skills(forge_dir: &Path, language: &Language) -> HashSet<String>` function in `forge-enricher`
2. Call it in `Enricher::enrich()` before `skill_registry.resolve()`
3. Log stale skills with a `tracing::warn!` and user-facing message
4. No change to `SkillRegistry::resolve()` API (Phase B will do that)

## Implementation Notes

```rust
// In forge-enricher/src/lib.rs:

fn load_stale_skills(forge_dir: &Path, language: &Language) -> HashSet<String> {
    let drift_dir = forge_dir.join("memory").join("drift");
    let lang_str = language.to_string().to_lowercase();
    let mut stale = HashSet::new();
    
    if let Ok(entries) = std::fs::read_dir(&drift_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with(&lang_str) || !name.ends_with(".json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Ok(report) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(skills) = report["skills"].as_array() {
                        for skill in skills {
                            let rate = skill["acceptance_rate"].as_f64().unwrap_or(1.0);
                            if rate < 0.5 {
                                if let Some(name) = skill["skill_name"].as_str() {
                                    stale.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    stale
}

// In Enricher::enrich(), before resolve():
let forge_dir = self.project_root.join(".forge");
let stale = load_stale_skills(&forge_dir, &language);
if !stale.is_empty() {
    tracing::warn!(
        stale_count = stale.len(),
        "Drift data: {} skill(s) have acceptance_rate < 0.5 and may need updating: {:?}. \
         Run `forge evolve` to refresh stale skills.",
        stale.len(),
        stale
    );
}
```

## Verification

- `cargo build --workspace` clean
- When `.forge/memory/drift/` is absent → no error, no warning
- When drift files contain skills with acceptance_rate < 0.5 → warning logged with skill names
- When all skills have acceptance_rate >= 0.5 → no warning
