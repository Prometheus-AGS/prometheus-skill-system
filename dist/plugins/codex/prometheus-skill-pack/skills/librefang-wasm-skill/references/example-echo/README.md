# Echo Skill — LibreFang WASM Example

The canonical example for the `librefang-wasm-skill` skill. ~80 lines of
Rust source demonstrating the LibreFang Guest ABI. This is a separate ABI from
the Prometheus Exec component world; packaging it does not establish Tier W
compatibility.

## Build after the complete production phase

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

## Install into a separately configured LibreFang host

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

These instructions assume the selected LibreFang host implements the shown
installation API. No host is started or installed by this example README.
Validate the actual guest/host invocation locally at the final phase boundary;
ABI inspection and a produced Wasm file are not an execution receipt.
