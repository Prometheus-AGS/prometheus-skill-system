# Echo Skill — LibreFang WASM Example

The canonical example for the `librefang-wasm-skill` skill. ~80 lines of
Rust, zero capabilities, demonstrates the full Guest ABI.

## Build

```bash
rustup target add wasm32-unknown-unknown   # if not already
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/echo.wasm`.

## Validate ABI

```bash
bash ../../scripts/validate-wasm-abi.sh \
  target/wasm32-unknown-unknown/release/echo.wasm
```

## Package

```bash
mkdir -p dist
cp target/wasm32-unknown-unknown/release/echo.wasm dist/
cp skill.toml dist/
cp README.md dist/
(cd dist && zip ../echo-skill.zip echo.wasm skill.toml README.md)
```

## Install into LibreFang

```bash
# Assumes `librefang start` is running on :4545
curl -X POST http://localhost:4545/skills/install \
  -H "Content-Type: application/zip" \
  --data-binary @echo-skill.zip

curl -X POST http://localhost:4545/skills/reload

curl http://localhost:4545/skills/echo | jq
```

## Invoke

```bash
# From an agent that has the skill installed:
#   tool: echo
#   input: { "message": "hello" }
# Response: { "echoed": { "message": "hello" } }
```
