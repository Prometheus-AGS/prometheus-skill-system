#!/bin/bash
# scratch-surreal.sh — shared scratch surreal-memory-server for integration tests.
#
# Source it (do not execute) BEFORE overriding HOME so the real model cache can
# be located:
#
#   . "$HERE/lib/scratch-surreal.sh"
#   PORT="$(scratch_surreal_pick_port)"          # honours TLI_SM_PORT
#   ...
#   scratch_surreal_start "$S/sm" <namespace>    # exports SURREAL_MEMORY_URL
#   ...
#   scratch_surreal_stop                         # kills it, verifies, removes the dir
#
# Never the live service: the port is a free ephemeral one (or an explicit
# TLI_SM_PORT that must not already answer), and a data dir is always scratch.
# Missing/old binary or executor calls `blocked` (exit 2). bash 3.2 compatible.
#
# Environment: TLI_SM_BIN (binary, default PATH surreal-memory-server >= 1.10.0),
# TLI_SM_PORT (fixed port), TLI_SM_REUSE (tolerate an answering explicit port).

_SS_REAL_HOME="${_SS_REAL_HOME:-$HOME}"
_SS_EXECUTOR="${TLI_SM_EXECUTOR:-/usr/local/bin/surreal-memory-mlx-executor}"
SCRATCH_SURREAL_PID=""
SCRATCH_SURREAL_DIR=""
SCRATCH_SURREAL_PORT=""

if ! type blocked >/dev/null 2>&1; then
  blocked() { echo "BLOCKED: $*" >&2; exit 2; }
fi

_ss_version_ok() { "$1" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.'; }

# Prints the port to use: TLI_SM_PORT when set (blocked if something answers on
# it), otherwise a free ephemeral port.
scratch_surreal_pick_port() {
  if [ -n "${TLI_SM_PORT:-}" ]; then
    if curl -fsS -m 1 "http://127.0.0.1:$TLI_SM_PORT/health" >/dev/null 2>&1 && [ -z "${TLI_SM_REUSE:-}" ]; then
      blocked "port $TLI_SM_PORT is already in use (set TLI_SM_PORT)"
    fi
    echo "$TLI_SM_PORT"; return 0
  fi
  python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()' \
    || blocked "could not pick a free port (python3 missing?)"
}

# scratch_surreal_start <data-dir> <namespace> [port]
scratch_surreal_start() {
  local dir="$1" ns="${2:-scratch}" port="${3:-${SCRATCH_SURREAL_PORT:-}}"
  local bin="${TLI_SM_BIN:-$(command -v surreal-memory-server || true)}"
  [ -n "$bin" ] && [ -x "$bin" ] || blocked "surreal-memory-server not found (set TLI_SM_BIN)"
  _ss_version_ok "$bin" || blocked "surreal-memory-server < 1.10.0"
  [ -x "$_SS_EXECUTOR" ] || blocked "MLX embedding executor missing"
  command -v curl >/dev/null 2>&1 || blocked "curl missing"
  [ -n "$port" ] || port="$(scratch_surreal_pick_port)"
  mkdir -p "$dir" || blocked "cannot create scratch data dir $dir"
  SCRATCH_SURREAL_DIR="$(cd "$dir" && pwd -P)"
  SCRATCH_SURREAL_PORT="$port"
  # argv[0] carries the data dir so a leaked process is findable with pgrep -f.
  ( cd "$SCRATCH_SURREAL_DIR" \
    && export SURREAL_MODE=embedded SURREAL_PATH="$SCRATCH_SURREAL_DIR/db" SURREAL_NAMESPACE="$ns" SURREAL_DATABASE=gate \
       API_HOST=127.0.0.1 API_PORT="$port" MCP_STDIO=false EMBEDDING_PROVIDER=local LOCAL_EMBEDDING_BACKEND=mlx \
       LOCAL_EMBEDDING_EXECUTOR="$_SS_EXECUTOR" LOCAL_EMBEDDING_MODEL=BAAI/bge-small-en-v1.5 \
       LOCAL_EMBEDDING_MODEL_REVISION=5c38ec7c405ec4b44b94cc5a9bb96e735b38267a LOCAL_EMBEDDING_DIMENSIONS=384 \
       HF_HUB_CACHE="$_SS_REAL_HOME/.cache/huggingface/hub" MODEL_CACHE_DIR="$_SS_REAL_HOME/.cache/huggingface" \
       SURREAL_EXECUTOR_STARTUP_MS=300000 RUST_LOG=warn \
    && exec -a "surreal-memory-server[$SCRATCH_SURREAL_DIR]" "$bin" > "$SCRATCH_SURREAL_DIR/server.log" 2>&1 ) &
  SCRATCH_SURREAL_PID=$!
  export SURREAL_MEMORY_URL="http://127.0.0.1:$port"
  local _
  for _ in $(seq 1 180); do
    curl -fsS -m 1 "$SURREAL_MEMORY_URL/ready" 2>/dev/null | grep -q '"ledger":true' && return 0
    sleep 1
  done
  tail -20 "$SCRATCH_SURREAL_DIR/server.log" >&2
  scratch_surreal_stop >/dev/null 2>&1 || true
  blocked "scratch surreal-memory never became ready"
}

# Stops the scratch server, removes its data dir (unless TLI_KEEP) and returns 1
# when any process still holds the data dir or the port still answers.
scratch_surreal_stop() {
  local dir="$SCRATCH_SURREAL_DIR" pid="$SCRATCH_SURREAL_PID" rc=0 _
  if [ -n "$pid" ]; then
    pkill -TERM -P "$pid" 2>/dev/null
    kill "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
  fi
  if [ -n "$dir" ]; then
    for _ in 1 2 3 4 5 6 7 8 9 10; do pgrep -f "$dir" >/dev/null 2>&1 || break; sleep 1; done
    if pgrep -f "$dir" >/dev/null 2>&1; then
      echo "FAIL: process still holds scratch surreal dir $dir:" >&2
      pgrep -fl "$dir" >&2
      rc=1
    fi
  fi
  if [ -n "$SCRATCH_SURREAL_PORT" ] && curl -fsS -m 1 "http://127.0.0.1:$SCRATCH_SURREAL_PORT/health" >/dev/null 2>&1; then
    echo "FAIL: scratch surreal-memory still answering on :$SCRATCH_SURREAL_PORT" >&2
    rc=1
  fi
  [ -z "${TLI_KEEP:-}" ] && [ -n "$dir" ] && rm -rf "$dir"
  SCRATCH_SURREAL_PID=""
  return $rc
}
