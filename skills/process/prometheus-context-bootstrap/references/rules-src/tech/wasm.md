---
paths: ['**/*.wit', '**/wasm32*/**', '**/wit/**']
---

# WASM (Component Model)

| Tier | Commands |
|---|---|
| T0 every edit | `cargo check --target wasm32-<target>` — faster than a build, catches most interface errors |
| T1 unit complete | `wasm-pack test --node` |
| T2 phase complete | `wasm-pack build` or `cargo component build`; WIT validation with `wasm-tools` |
| T3 milestone only | headless browser tests |

Pin `wasm-bindgen` to the CLI version exactly. The Rust rules and the 500-line limit apply to guest code.
