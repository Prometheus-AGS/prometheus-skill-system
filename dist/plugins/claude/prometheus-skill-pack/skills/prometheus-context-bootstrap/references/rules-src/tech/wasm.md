---
paths: ['**/*.wit', '**/wasm32*/**', '**/wit/**']
---

# WASM (Component Model)

Use static inspection during implementation. At a completed change boundary, run the smallest component
build plus WIT validation that exercises the changed public interface. At the final phase boundary, run
the applicable headless browser or host/guest integration flow. Do not use module-local or per-edit tests
as completion evidence. The Rust phase-gate and single-writer rules apply to every Cargo command.

Pin `wasm-bindgen` to the CLI version exactly. The Rust rules and the 500-line limit apply to guest code.
