---
license: MIT
name: tauri-react-vite
version: '1.0.0'
description: >
  Tauri 2 + React 19 + Vite 8 desktop application pattern for Prometheus AGS. Covers
  the Tauri command API, IPC invoke() pattern, Rust sidecar or plugin integration,
  Flutter WebView embedding within Tauri, secure window configuration, and the
  gen_ui_core Rust crate sharing between Tauri backend and flutter_rust_bridge.
  Use when building any Prometheus desktop application.
language: tauri
---

# Tauri 2 + React 19 + Vite 8

## Architecture

```
my-desktop-app/
├── src-tauri/                  ← Rust backend (Tauri 2)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs             ← Tauri app entry
│       ├── commands/           ← #[tauri::command] handlers
│       │   ├── mod.rs
│       │   ├── inference.rs
│       │   └── storage.rs
│       └── lib.rs
├── src/                        ← React 19 + Vite 8 frontend
│   ├── main.tsx
│   ├── routes/                 ← TanStack Router
│   ├── features/
│   └── lib/
│       └── tauri.ts            ← typed invoke() wrappers
└── vite.config.ts
```

## Tauri Commands (Rust Backend)

Define commands in `src-tauri/src/commands/`. All commands are async, return `Result`,
and use `tauri::State` for dependency injection.

```rust
// src-tauri/src/commands/inference.rs
use tauri::State;
use crate::InferenceState;

#[tauri::command]
pub async fn run_inference(
    state: State<'_, InferenceState>,
    prompt: String,
    model: String,
) -> Result<String, String> {
    state
        .client
        .complete(&prompt, &model)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_models(
    state: State<'_, InferenceState>,
) -> Result<Vec<String>, String> {
    state.client.list_models().await.map_err(|e| e.to_string())
}
```

Register in `main.rs`:
```rust
tauri::Builder::default()
    .manage(InferenceState::new())
    .invoke_handler(tauri::generate_handler![
        commands::inference::run_inference,
        commands::inference::list_models,
        commands::storage::read_file,
    ])
    .run(tauri::generate_context!())
    .expect("error running tauri app");
```

## Typed IPC Wrappers (TypeScript)

Never use raw `invoke()` in component code. Wrap all commands in typed functions
in `src/lib/tauri.ts` and import from there.

```ts
// src/lib/tauri.ts
import { invoke } from '@tauri-apps/api/core'

export async function runInference(prompt: string, model: string): Promise<string> {
  return invoke<string>('run_inference', { prompt, model })
}

export async function listModels(): Promise<string[]> {
  return invoke<string[]>('list_models')
}
```

In React components:
```tsx
import { runInference, listModels } from '@/lib/tauri'
import { useQuery, useMutation } from '@tanstack/react-query'

export function useModelList() {
  return useQuery({ queryKey: ['models'], queryFn: listModels })
}

export function useInferenceMutation() {
  return useMutation({
    mutationFn: ({ prompt, model }: { prompt: string; model: string }) =>
      runInference(prompt, model),
  })
}
```

## Sharing gen_ui_core with Flutter

When the same app needs both a Tauri desktop shell and a Flutter mobile target,
`gen_ui_core` is a shared Rust workspace crate used by both:

```toml
# src-tauri/Cargo.toml
[dependencies]
gen-ui-core = { path = "../../gen_ui_core" }  # shared with flutter_rust_bridge
```

In Tauri commands, call `gen_ui_core` directly (no FFI needed — same process):
```rust
use gen_ui_core::inference::InferenceClient;

#[tauri::command]
pub async fn run_inference(prompt: String, model: String) -> Result<String, String> {
    InferenceClient::global()
        .map_err(|e| e.to_string())?
        .complete(&prompt, &model)
        .await
        .map_err(|e| e.to_string())
}
```

## Flutter WebView Embedding in Tauri

For apps that combine a Tauri shell with Flutter UI panels:

```rust
// src-tauri/src/commands/flutter.rs
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub async fn open_flutter_panel(app: tauri::AppHandle, url: String) {
    WebviewWindowBuilder::new(&app, "flutter-panel", WebviewUrl::External(url.parse().unwrap()))
        .title("Flutter Panel")
        .inner_size(800.0, 600.0)
        .build()
        .expect("failed to build flutter webview");
}
```

## Secure Window Configuration (`tauri.conf.json`)

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; connect-src 'self' http://localhost:*"
    }
  },
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173"
  }
}
```

Never use `dangerousDisableAssetCspModification` in production. Never enable
`allowlist.all` — allowlist only the capabilities actually needed.

## Vite Config for Tauri

```ts
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  // Tauri expects a fixed port; mobile can't use ephemeral ports
  server: { port: 5173, strictPort: true },
  // Optimize asset handling for Tauri
  envPrefix: ['VITE_', 'TAURI_'],
  build: { target: 'esnext', minify: !process.env.TAURI_DEBUG ? 'esbuild' : false },
})
```

## Forbidden Patterns

- `eval()` in any JS/TS code running inside Tauri — use `invoke()`
- `window.location.href` for navigation — use TanStack Router
- Raw `invoke('command_name')` without type parameter — always `invoke<ReturnType>(...)`
- Accessing `window.__TAURI__` directly — use `@tauri-apps/api` imports
- `allowlist.all: true` in `tauri.conf.json` — allowlist precisely
