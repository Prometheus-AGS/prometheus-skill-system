# gen_ui_core Rust Core Template
> Use `scripts/scaffold-rust-core.mjs` to generate this structure

## Complete module implementation checklist

After scaffolding, implement these modules in order:

### Phase 1 — Foundation (required before any UI can run)

1. **`runtime.rs`** — Already complete from scaffold
2. **`config.rs`** — Already complete from scaffold
3. **`api_http.rs`** — Anthropic HTTP/2 client

```rust
// src/api_http.rs
use anyhow::{Context, Result};
use reqwest::{header, Client};
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const BASE: &str  = "https://api.anthropic.com/v1";
const VER:  &str  = "2023-06-01";
const BETA: &str  = "interleaved-thinking-2025-05-14,prompt-caching-2024-07-31";

#[derive(Debug, Clone, Serialize)]
pub struct AnthropicRequest {
    pub model:       String,
    pub max_tokens:  u32,
    pub messages:    Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system:      Option<Vec<SystemBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools:       Option<Vec<ToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking:    Option<ThinkingConfig>,
    pub stream:      bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage { pub role: String, pub content: Vec<ApiContent> }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiContent {
    Text       { text: String },
    ToolUse    { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: Vec<ApiContent>, is_error: Option<bool> },
    Thinking   { thinking: String, signature: Option<String> },
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemBlock {
    #[serde(rename = "type")] pub kind: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheCtrl>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheCtrl { #[serde(rename = "type")] pub kind: String }
impl CacheCtrl { pub fn ephemeral() -> Self { Self { kind: "ephemeral".into() } } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String, pub description: String, pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheCtrl>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")] pub kind: String,
    pub budget_tokens: u32,
}
impl ThinkingConfig {
    pub fn with_budget(n: u32) -> Self { Self { kind: "enabled".into(), budget_tokens: n } }
}

#[derive(Clone)]
pub struct AnthropicClient { pub(crate) http: Client, pub(crate) api_key: String }

impl AnthropicClient {
    pub fn new(api_key: impl Into<String>) -> anyhow::Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(10))
            .use_rustls_tls()
            .http2_prior_knowledge()
            .build()
            .context("HTTP client")?;
        Ok(Self { http, api_key: api_key.into() })
    }

    pub fn stream_messages(&self, req: AnthropicRequest) -> anyhow::Result<EventSource> {
        let body = serde_json::to_string(&req)?;
        let r = self.http
            .post(format!("{BASE}/messages"))
            .header("x-api-key",         &self.api_key)
            .header("anthropic-version", VER)
            .header("anthropic-beta",    BETA)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT,       "text/event-stream")
            .body(body).build()?;
        Ok(EventSource::new(r)?)
    }
}
```

### Phase 2 — Protocol pipeline

4. **`streaming.rs`** — SSE event parser (see references/rust/patterns.md)
5. **`protocol/a2ui.rs`** — A2UI adapter + 27 event variants
6. **`protocol/agui.rs`** — AG-UI translation adapter
7. **`protocol/mod.rs`** — ProtocolPipeline (dual broadcast channels)

### Phase 3 — Infrastructure

8. **`db/mod.rs`** — SurrealDB: MemoryStore, ToolCache, EntityGraph
9. **`mcp/mod.rs`** — internal UAR transports behind the governance boundary
10. **`mcp/sse_transport.rs`** — HTTP SSE MCP transport

### Phase 4 — Agent loop

11. **`inference/mod.rs`** — InferenceEngine, ModelId, ChatTemplate
12. **`inference/sampler.rs`** — Token sampling
13. **`uar/mod.rs`** — pinned embedded/service/test `UarRuntimeFacade` adapters

### Phase 5 — FFI surface

14. **`api.rs`** — flutter_rust_bridge codegen target (Flutter)
    OR update **`src-tauri/src/commands.rs`** (Tauri)

## Cargo features for optional capabilities

```toml
# Cargo.toml — optional features to control binary size
[features]
default     = ["embedded-uar", "local-inference", "surreal-db"]
embedded-uar    = []             # Include the pinned UAR facade implementation
local-inference = ["candle-core", "candle-nn", "candle-transformers", "hf-hub", "tokenizers"]
surreal-db      = ["surrealdb"]
mcp-support     = []             # Internal UAR transport support; never app-facing
flutter-ffi     = ["flutter_rust_bridge"]   # Flutter FFI surface
tauri-plugin    = ["tauri"]                 # Tauri plugin surface
```

## Build profiles

```toml
[profile.release]
opt-level     = 3
lto           = "thin"
codegen-units = 1
panic         = "abort"
strip         = "symbols"

[profile.dev]
opt-level = 1    # Faster incremental builds
debug     = true
```

## Android NDK setup (required for cargo-ndk)

```bash
# .cargo/config.toml (at workspace root)
[target.aarch64-linux-android]
linker = "aarch64-linux-android-clang"
ar     = "llvm-ar"

[target.armv7-linux-androideabi]
linker = "armv7a-linux-androideabi-clang"
ar     = "llvm-ar"

[target.x86_64-linux-android]
linker = "x86_64-linux-android-clang"
ar     = "llvm-ar"

[env]
ANDROID_NDK_HOME = { value = "path/to/ndk", force = false }
```
