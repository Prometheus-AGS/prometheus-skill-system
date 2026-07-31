# Component-target inventory — captured 2026-07-31
Three INCOMPATIBLE wasm guest targets exist. This is repo-verifiable.

## 1. LibreFang core-wasm ABI (in THIS repo)
  skills/rust/librefang-wasm-skill/templates/
  crate-type: crate-type=["cdylib"]
  exports: alloc execute 
  imports: host_call (JSON-over-pointer), NOT Component Model
  .wit files: 0

## 2. UAR component world (external)
  world skill { export run: func(input: string) -> result<string, string>; }
  loader: wasmtime::component::Component — requires COMPONENT MODEL binary

## 3. KnowMe knowme:plugin (external) — lifecycle/hook/contributor worlds

## Consequence
A guest built from the librefang template CANNOT load in UAR's component
runtime: core-wasm + extern "C" is not a component. The templates are not
reusable for goal 1 as-is.

## Real skill components shipped today: 0
```console
$ find skills -name "skill.wasm" -not -path "*/node_modules/*"
(none)
```
