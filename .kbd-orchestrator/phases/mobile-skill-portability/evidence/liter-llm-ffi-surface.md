# liter-llm FFI/JNI/WASM crates — in-tree submodule evidence, 2026-07-31
submodule HEAD: 3545cf6a2a69d77358d1658183f30b1a9b893d51

- liter-llm-ffi: crate-type=["cdylib","staticlib","rlib"]
- liter-llm-jni: crate-type=["cdylib"]
- liter-llm-wasm: crate-type=["cdylib"]
- liter-llm-py: crate-type=["cdylib"]
- liter-llm-node: crate-type=["cdylib"]
- liter-llm-php: crate-type=["cdylib"]

JNI entry points: 46
C header: tools/liter-llm/crates/liter-llm-ffi/include/liter_llm.h (1022 symbol refs)
Java package files: 150
cbindgen: 1 ref(s) in liter-llm-ffi/Cargo.toml

## Correction to the first count

An initial grep matched only 4 symbols and led to the claim that the C ABI is
"error-inspection only". That pattern was too narrow. Recounted:

- declared functions in the header: **767**
- unique `literllm_*` symbols: **800**
- header size: **6767 lines**

Reproduce:

```bash
grep -cE '^[A-Za-z_].*literllm_[a-z_]+\(' tools/liter-llm/crates/liter-llm-ffi/include/liter_llm.h
```

This is a **complete client ABI**, not a proof of shape.
