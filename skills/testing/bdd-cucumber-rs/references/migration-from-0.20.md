# Migrating from cucumber 0.20 → 0.23

Two breaking-change hops sit between 0.20 (the last widely-deployed
version before native async traits) and 0.23 (current stable as of
2026-07). This guide walks a legacy 0.20 project to 0.23.

## Version cadence

| Version | Released | Headline change |
|---------|----------|-----------------|
| 0.20 | 2023-07 | `#[async_trait]` on World/Writer; MSRV 1.65 |
| **0.21** | 2024-04 | **Removed `#[async_trait]`** — traits use native `async fn`; MSRV **1.75** |
| 0.22 | 2025-12 | MSRV bumped to **1.88** |
| **0.23** | **2026-04** | current stable |

## Migration steps

### 1. Bump MSRV to 1.88 (or newer)

`Cargo.toml`:

```toml
[package]
rust-version = "1.88"
```

CI matrix needs to install a matching toolchain (or use `stable`, which is
past 1.88 as of 2026-07).

### 2. Bump the `cucumber` dep

```toml
[dev-dependencies]
- cucumber = "0.20"
+ cucumber = "0.23"
```

### 3. Remove `#[async_trait]` from your World impl

Before (0.20):

```rust
use async_trait::async_trait;
use cucumber::World;

#[async_trait(?Send)]
impl World for MyWorld {
    type Error = std::convert::Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self::default())
    }
}
```

After (0.23) — use the derive macro:

```rust
use cucumber::World;

#[derive(Debug, Default, World)]
pub struct MyWorld {
    // fields
}
```

Or, if you need a custom constructor, implement without `#[async_trait]`:

```rust
impl World for MyWorld {
    type Error = std::convert::Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self { /* custom init */ ..Default::default() })
    }
}
```

Do NOT keep `async-trait` as a dependency for the cucumber World unless
another crate in your workspace still requires it.

### 4. Remove `#[async_trait]` from custom Writers

If you built a custom `Writer` for the report format, the same removal
applies:

```rust
- #[async_trait(?Send)]
  impl Writer<MyWorld> for MyReporter {
      type Cli = cli::Empty;

      async fn handle_event(&mut self, event: ...) { /* ... */ }
  }
```

### 5. Attribute macros — no changes needed

`#[given]`, `#[when]`, `#[then]` are unchanged from 0.20 → 0.23. The
regex and Cucumber Expression syntax are identical.

### 6. Re-check parameter parsers

`FromStr`-based parameter parsing continues to work. If you used
custom `Parameter` types via the `cucumber::Parameter` derive, check the
CHANGELOG — the derive syntax was refined in 0.22 (`regex` attribute is
now `regex(...)`).

### 7. Run the suite

```bash
cargo update -p cucumber
cargo test --test features
```

Any remaining errors are almost always MSRV-related (a downstream crate
that pinned an older Rust). Bump `rust-version` and re-run.

## If you hit a snag

- **"cannot find `#[async_trait]`"** — you missed step 3. Remove the
  attribute and rely on the derive macro.
- **"error[E0658]: use of unstable library feature `async_fn_in_trait`"** —
  your Rust toolchain is older than 1.75. Upgrade to 1.88+ per MSRV.
- **"trait `World` is not implemented"** — your struct is missing
  `#[derive(World)]` or a manual `impl World for T`.

## Reference

- Upstream CHANGELOG: <https://github.com/cucumber-rs/cucumber/blob/main/CHANGELOG.md>
- 0.21 release notes covered the `#[async_trait]` removal in detail
