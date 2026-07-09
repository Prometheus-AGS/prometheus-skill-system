---
name: bdd-cucumber-rs
version: '1.0.0'
license: MIT
description: >
  Author, run, and maintain BDD integration tests in Rust using the
  cucumber 0.23 crate with async World, tokio + reqwest for HTTP scenarios,
  and thirtyfour for browser-driven scenarios. Use when writing behavior
  tests, integration tests, feature files, or step definitions for any
  Rust crate or workspace.
metadata:
  author: prometheus-skill-pack
  category: testing
  tags: [testing, bdd, cucumber, cucumber-rs, rust, gherkin, e2e, thirtyfour]
---

# BDD Cucumber-rs Skill

Author and run BDD integration tests for **any Rust project**. This skill is
crate-agnostic. Pair it with `bdd-lifecycle-loop` for the author → run →
triage → maintain workflow, and with `bdd-video-proof` for certification
bundles.

## When to invoke

- "Write a BDD test for the auth service"
- "Add a cucumber-rs scenario for the CRDT merge"
- "Create integration tests using Gherkin in this Rust workspace"
- "Set up cucumber-rs + thirtyfour for browser testing"

## Stack (2026-07)

- `cucumber` **0.23.0** (crates.io, ~230k downloads/month, MSRV 1.88)
- `tokio` 1.x — async runtime
- `reqwest` 0.12+ — HTTP client for API scenarios
- `thirtyfour` **0.37.2** — primary WebDriver-based browser driver
- `ffmpeg` (system) — optional MP4 remux for video-proof bundles

Alternative browser drivers documented in
[references/browser-drivers.md](references/browser-drivers.md):
`fantoccini` (older, more stars) and `headless_chrome` (Chrome-only, CDP).

## Directory layout

Cucumber-rs conventionally puts feature files under `tests/features/` at the
crate root, and step definitions in a binary at `tests/features.rs`:

```
crate-root/
├── Cargo.toml
├── src/
│   └── ...
└── tests/
    ├── features/            ← Gherkin .feature files
    │   ├── api/
    │   └── ui/
    ├── features.rs          ← Cucumber entrypoint (main)
    ├── steps/               ← Step modules
    │   ├── api.rs
    │   ├── ui.rs
    │   └── common.rs
    └── support/
        └── world.rs         ← World type
```

`Cargo.toml`:

```toml
[dev-dependencies]
cucumber = "0.23"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thirtyfour = { version = "0.37", optional = true }

[[test]]
name = "features"
harness = false                     # cucumber owns the runner

[features]
ui = ["thirtyfour"]                 # gate browser tests behind a feature
```

## Choose your scenario style

| Style | When to pick | Runtime |
|-------|--------------|---------|
| **HTTP-only** (@api) | Testing REST/gRPC/CLI, no browser needed | `cargo test --test features -- --tags @api` — fast (~ms) |
| **Browser** (@ui) | Rendering, forms, navigation, visual proof | `cargo test --features ui --test features -- --tags @ui` — slower (seconds), requires running WebDriver (`chromedriver` / `geckodriver`) |

## Async World in cucumber 0.23

**No more `#[async_trait]`.** Since 0.21 the `World` trait uses native
`async fn` in traits (MSRV bumped to 1.88 in 0.22). Steps are plain
`async fn`; `#[given]` / `#[when]` / `#[then]` macros wire them up.

```rust
// tests/features.rs
use cucumber::{given, then, when, World};

#[derive(Debug, Default, World)]
pub struct AuthWorld {
    pub base_url: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub response: Option<reqwest::Response>,
    pub body: Option<serde_json::Value>,
}

#[given(regex = r"^the auth service is reachable at \"([^\"]+)\"$")]
async fn service_up(w: &mut AuthWorld, url: String) {
    w.base_url = url;
}

#[given(regex = r"^a registered user \"([^\"]+)\" with password \"([^\"]+)\"$")]
async fn user(w: &mut AuthWorld, email: String, password: String) {
    w.email = Some(email);
    w.password = Some(password);
}

#[when(regex = r"^they POST to \"([^\"]+)\" with those credentials$")]
async fn post(w: &mut AuthWorld, path: String) {
    let url = format!("{}{}", w.base_url, path);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "email": w.email,
            "password": w.password,
        }))
        .send()
        .await
        .expect("request failed");
    w.body = Some(resp.json().await.expect("bad json"));
}

#[then(regex = r"^the response body contains a non-empty \"([^\"]+)\" field$")]
async fn field_present(w: &mut AuthWorld, field: String) {
    let val = w.body.as_ref().and_then(|b| b.get(&field));
    assert!(val.is_some() && !val.unwrap().as_str().unwrap_or("").is_empty());
}

#[tokio::main]
async fn main() {
    AuthWorld::run("tests/features").await;
}
```

## Feature file

`tests/features/api/sign-in.feature`:

```gherkin
@api
Feature: Auth service returns a token on valid credentials

  Scenario: Happy path
    Given the auth service is reachable at "http://localhost:3000"
    And a registered user "alice@example.com" with password "hunter2"
    When they POST to "/api/auth/sign-in" with those credentials
    Then the response body contains a non-empty "token" field
```

Run: `cargo test --test features`

## Browser-driven scenarios (thirtyfour)

For `@ui` scenarios, extend `World` with a `WebDriver` handle. Requires
`chromedriver` or `geckodriver` running on `localhost:4444` (default port
of the W3C WebDriver spec).

```rust
use cucumber::{given, then, when, World};
use thirtyfour::{DesiredCapabilities, WebDriver};

#[derive(Debug, Default, World)]
pub struct UiWorld {
    pub driver: Option<WebDriver>,
    pub base_url: String,
}

#[given(regex = r"^the app is running at \"([^\"]+)\"$")]
async fn app_up(w: &mut UiWorld, url: String) {
    w.base_url = url;
    let caps = DesiredCapabilities::chrome();
    w.driver = Some(
        WebDriver::new("http://localhost:4444", caps)
            .await
            .expect("could not start WebDriver"),
    );
}

#[when(regex = r"^they navigate to the sign-in page$")]
async fn goto_signin(w: &mut UiWorld) {
    let driver = w.driver.as_ref().expect("driver not initialized");
    driver
        .goto(format!("{}/sign-in", w.base_url))
        .await
        .expect("navigation failed");
}

#[then(regex = r"^they land on the dashboard$")]
async fn on_dashboard(w: &mut UiWorld) {
    let driver = w.driver.as_ref().unwrap();
    let url = driver.current_url().await.unwrap();
    assert!(url.as_str().contains("/dashboard"));
}
```

Teardown the WebDriver in an `#[after]` hook so it disconnects cleanly:

```rust
use cucumber::gherkin::Scenario;

// In main:
UiWorld::cucumber()
    .after(|_, _, _, _, w| {
        Box::pin(async move {
            if let Some(driver) = w.and_then(|w| w.driver.take()) {
                let _ = driver.quit().await;
            }
        })
    })
    .run("tests/features")
    .await;
```

## Running

```bash
# HTTP-only scenarios (@api)
cargo test --test features -- --tags @api

# Browser scenarios (@ui) — requires chromedriver / geckodriver
cargo test --features ui --test features -- --tags @ui

# All (excluding known flakes)
cargo test --test features -- --tags "not @flaky"

# Verbose Gherkin output
cargo test --test features -- --format=libtest --show-output
```

## Video capture

`thirtyfour` (WebDriver) does not natively record video. Two workable paths:

1. **ffmpeg screen capture** — pipe X11/Wayland/CoreGraphics into
   `ffmpeg`; wrap `cargo test` in a script that starts and stops the
   recorder around each scenario. See `bdd-video-proof` for the pattern.
2. **selenium-video sidecar** — run `selenium/video` Docker image alongside
   `chromedriver` for automatic per-session MP4s.

For CDP-based video (Chrome only) see `references/browser-drivers.md`.

## See also

- [bdd-cucumber-js](../bdd-cucumber-js/SKILL.md) — TypeScript equivalent
- [bdd-lifecycle-loop](../bdd-lifecycle-loop/SKILL.md) — author → run → triage → maintain
- [bdd-video-proof](../bdd-video-proof/SKILL.md) — certification bundles
- [references/browser-drivers.md](references/browser-drivers.md) — thirtyfour vs fantoccini vs headless_chrome
- [references/migration-from-0.20.md](references/migration-from-0.20.md) — upgrading from cucumber 0.20
- `docs/future-work/02-bdd-testing-evolution/BDD-005-testid-drift-detection.md`
- `docs/future-work/02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md`
