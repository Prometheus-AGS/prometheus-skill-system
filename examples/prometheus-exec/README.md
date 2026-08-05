# Prometheus Exec runnable examples

These examples demonstrate the boundary between code generation and evidence-producing execution. The source is ordinary Python, Node, and Bash. Prometheus Exec supplies declared inputs, a private output directory, limits, durable events, content-addressed artifacts, and the signed receipt.

## Prerequisites

- installed `prometheus-exec 1.7.0`;
- a supported local Tier P backend (the release Mac uses Seatbelt); and
- an active signed plugin generation for the Tier W example.

Use disposable state:

```bash
prometheus-exec init --identity ./exec-identity.json
prometheus-exec daemon \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack"
```

## Python

```bash
prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --runtime python3 \
  --code ./examples/prometheus-exec/tier-p/transform.py \
  --input records=./examples/prometheus-exec/tier-p/records.json \
  --timeout-ms 5000 \
  --output-mb 2 \
  --format json
```

The `summary.json` artifact must equal `tier-p/expected-summary.json`.

## Node

Use the same command with `--runtime node` and `--code ./examples/prometheus-exec/tier-p/transform.mjs`. It produces the same business output through a different attested interpreter.

## Bash

```bash
prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --runtime bash \
  --code ./examples/prometheus-exec/tier-p/transform.sh \
  --input numbers=./examples/prometheus-exec/tier-p/numbers.txt \
  --timeout-ms 5000 \
  --output-mb 2 \
  --format json
```

The `total.txt` artifact must equal `tier-p/expected-total.txt`.

## Tier W reference component

```bash
prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack" \
  --runtime wasm-component \
  --code ./skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm \
  --format json
```

The checked component implements `prometheus:component@0.1.0` and is authorized by the active signed generation. A LibreFang `wasm32-unknown-unknown` agent skill is not a substitute for this component.

## Local documentation commands

```bash
npm run check:docs-exec-examples
npm run docs:examples
```

The first command validates source syntax, expected output, paths, and documentation contracts without a daemon. The second runs the repository's disposable Prometheus Exec certification driver using the installed binary and active plugin generation. Neither command belongs in hosted product validation.
