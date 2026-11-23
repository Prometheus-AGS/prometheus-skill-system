#!/usr/bin/env bash
# Keep the binary's loopback-only listener. Publish the relay only on host loopback.
set -eu
outer="$1"
inner="$2"
shift 2
socat "TCP4-LISTEN:${outer},bind=0.0.0.0,reuseaddr,fork" "TCP4:127.0.0.1:${inner}" &
relay=$!
"$@" &
service=$!
trap 'kill "$service" "$relay" 2>/dev/null || true' EXIT
wait -n "$service" "$relay"
