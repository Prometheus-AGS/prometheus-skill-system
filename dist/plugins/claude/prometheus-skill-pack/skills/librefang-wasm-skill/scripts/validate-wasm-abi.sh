#!/usr/bin/env bash
# Validate that a built .wasm conforms to the LibreFang Guest ABI.
#
# Required exports: memory, alloc, execute
# Required imports under "librefang": host_call (and optionally host_log)
# Forbidden: any import from "wasi_snapshot_preview1" / "wasi:" — LibreFang
#            uses core wasmtime Module+Linker, NOT the Component Model, so any
#            WASI imports go unsatisfied at sandbox load time.
#
# Usage: validate-wasm-abi.sh <path/to/skill.wasm>

set -euo pipefail

WASM="${1:-}"
if [ -z "$WASM" ] || [ ! -f "$WASM" ]; then
    echo "Usage: $0 <path/to/skill.wasm>" >&2
    exit 2
fi

if ! command -v wasm-tools >/dev/null 2>&1; then
    echo "❌ wasm-tools not found on PATH." >&2
    echo "   Install with: cargo install wasm-tools  (or: brew install wasm-tools)" >&2
    exit 2
fi

echo "🔍 Validating $WASM"
echo "================================================"

PASS=0
FAIL=0

# The wat output contains lines like:
#   (export "memory" (memory 0))
#   (export "alloc" (func 17))
#   (export "execute" (func 18))
#   (import "librefang" "host_call" (func ...))
WAT="$(wasm-tools print "$WASM" 2>/dev/null || true)"
if [ -z "$WAT" ]; then
    echo "❌ wasm-tools print failed — not a valid WASM module"
    exit 1
fi

# --- Required exports ---
for name in memory alloc execute; do
    if echo "$WAT" | grep -q "(export \"$name\" "; then
        echo "  ✅ export $name"
        PASS=$((PASS + 1))
    else
        echo "  ❌ MISSING export $name"
        FAIL=$((FAIL + 1))
    fi
done

# --- Required imports ---
if echo "$WAT" | grep -q "(import \"librefang\" \"host_call\" "; then
    echo "  ✅ import librefang.host_call"
    PASS=$((PASS + 1))
else
    # host_call is "required" only for skills that actually call the host.
    # Most do; skills that don't are still valid but unusual.
    echo "  ℹ️  import librefang.host_call NOT found (skill makes no host calls)"
fi

if echo "$WAT" | grep -q "(import \"librefang\" \"host_log\" "; then
    echo "  ✅ import librefang.host_log"
fi

# --- Forbidden imports ---
if echo "$WAT" | grep -q "(import \"wasi_snapshot_preview1\""; then
    echo "  ❌ FORBIDDEN: wasi_snapshot_preview1 imports detected"
    echo "     LibreFang uses core wasmtime (no Component Model). Rebuild with --target wasm32-unknown-unknown."
    FAIL=$((FAIL + 1))
fi

# Direct net/file syscalls outside the librefang module are a sign someone
# bypassed the host bridge. These bypass capability checks and SHOULD fail
# at sandbox load, but it's better to catch them here.
for forbidden in "wasi:filesystem" "wasi:sockets" "wasi:cli/exit"; do
    if echo "$WAT" | grep -q "(import \"$forbidden"; then
        echo "  ⚠️  Suspicious import: $forbidden"
        echo "     Skills should route I/O through host_call, not direct WASI."
    fi
done

# --- Size sanity ---
SIZE=$(stat -f%z "$WASM" 2>/dev/null || stat -c%s "$WASM" 2>/dev/null || echo "?")
if [ "$SIZE" != "?" ]; then
    if [ "$SIZE" -gt $((4 * 1024 * 1024)) ]; then
        echo "  ⚠️  Size: ${SIZE} bytes (>4 MB — consider opt-level=\"z\" + lto + strip)"
    else
        echo "  ✅ Size: ${SIZE} bytes"
    fi
fi

echo ""
echo "================================================"
if [ "$FAIL" -gt 0 ]; then
    echo "❌ $FAIL ABI check(s) failed"
    exit 1
fi
echo "✨ ABI validation passed ($PASS checks)"
