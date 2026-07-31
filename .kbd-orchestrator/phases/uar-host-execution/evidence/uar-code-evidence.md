# UAR code evidence — captured 2026-07-31
Cross-repo: NOT resolvable from this repo's review packet. Reproduction
commands given so each claim is checkable, not asserted.

universal-agent-runtime @ 563ecc23316177e8d7bece00e84de02574737a92

## R2 — Builtin skills are delete-protected (service.rs:374)
sha256 a30f605051bdf021c6849a17c6f125cd5c1c00acfd7adbbea0bfdfa834e8c36b
```rust
    ///
    /// Skills with `origin = Builtin` are immutable; this method returns
    /// `Err(SystemSkillImmutable)` for them so the API layer can map to 409.
    pub async fn delete_skill_permanent(&self, id: &str) -> anyhow::Result<bool> {
        // Block deletion of Builtin skills (system-shipped, immutable).
        {
            let registry = self.registry.read().await;
            if let Some(skill) = registry.get(id) {
                if matches!(
                    skill.origin,
                    crate::uar::domain::skills::SkillOrigin::Builtin
                ) {
                    anyhow::bail!("system_skill_immutable");
                }
            }
        }

        let removed = self.registry.write().await.remove(id).is_some();
```

## R2 gap — the DB schema has NO origin and NO enabled column
sha256 c650a71b02d894cca8fb0758b7617e77b9d36bf78ca31a6f3e0704e3223ab92d
```sql
CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    definition JSONB NOT NULL,
    embedding VECTOR(384),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

## R1 — builtin loader discovery root (builtin_loader.rs:129)
```rust
pub fn builtin_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UAR_BUILTIN_SKILLS_DIR") {
        return PathBuf::from(s);
    }
    PathBuf::from("crates/prometheus-skill-system/skills")
}
```

## R4 — lib.rs exports no skill-facing SDK
```console
$ grep -E '^pub (mod|use)' src/lib.rs
pub mod config;
pub mod config_manager;
pub mod llm;
pub mod mcp;
pub mod normalized;
pub mod sandbox;
pub mod server;
pub mod session;
pub mod uar;
pub use uar::error::{Result, UarError};
```

## R5 — no provenance recorded by the loader
```console
$ grep -rn 'rev-parse|git_sha|commit' src/uar/runtime/skills/builtin_loader.rs
(no matches — exit 1)
```

## Verified by running, not reading
`cargo test --lib builtin_loader` -> **9 passed, 0 failed** after the
359-commit submodule fast-forward. `cargo check --lib` clean.
