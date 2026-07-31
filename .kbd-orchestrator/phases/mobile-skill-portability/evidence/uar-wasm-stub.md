# Cross-repo evidence, captured 2026-07-31
# Verifiable from THIS repo: excerpts + sha256 + repo SHA.

## universal-agent-runtime @ 563ecc23316177e8d7bece00e84de02574737a92
### src/uar/runtime/skills/wasm_runtime.rs (sha256 bf8e38a3b6ca3f7a21c466dc9144890e31729992cb9959ded215b645451acde9)
```rust
    pub async fn run(&self, skill_id: &str, input: &str) -> Result<String> {
        let components = self.components.lock().await;
        let _component = components
            .get(skill_id)
            .ok_or_else(|| anyhow::anyhow!("wasm skill not loaded: {skill_id}"))?
            .clone();
        drop(components);

        let _store = Store::new(&self.engine, WasmHostState {});
        let _linker: Linker<WasmHostState> = Linker::new(&self.engine);
        // Concrete component bindings will be added once wit-bindgen
        // integration lands (the WIT world is pinned; this is implementation
        // surface, not API surface). For now, return a stub so callers can
        // wire dispatch end-to-end without a fixture component.
        let _ = input;
        Ok(format!(
            "<wasm skill '{skill_id}' loaded but binding not yet generated; \
             implement wit-bindgen invocation here>"
        ))
    }
```
### wit/uar-skill.wit (sha256 fac96fcea9d82d449db05f735765f3b1adbf5c3aa4cde6d5a549a6257dd320f8)
```wit
package uar:skill@0.1.0;

world skill {
  export run: func(input: string) -> result<string, string>;
}
```
