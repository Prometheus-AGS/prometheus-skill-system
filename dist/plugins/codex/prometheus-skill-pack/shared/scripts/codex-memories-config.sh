#!/usr/bin/env bash
# codex-memories-config.sh — keep Codex from regenerating memory_summary.md.
#
# Codex injects ${CODEX_HOME}/memories/memory_summary.md into every thread. This
# script idempotently sets `generate_memories = false` under a `[memories]` table
# in ${CODEX_HOME:-$HOME/.codex}/config.toml with a line-level edit (every other
# line and comment is preserved), writes a timestamped backup before changing the
# file, re-parses the result and restores the backup on a parse failure, and
# archives an existing memories/memory_summary.md to memories-archive/. MEMORY.md,
# raw_memories.md and every other file are left in place.
#
# Usage: codex-memories-config.sh [--check]
#   --check   print state as JSON; write nothing.
# Exit: 0 ok (or Codex absent: silent), 1 failure (original config restored).
# Bash 3.2 compatible.
set -uo pipefail

CHECK=0
for arg in "$@"; do
  case "$arg" in
    --check) CHECK=1 ;;
    *) echo "codex-memories-config: unknown argument: $arg" >&2; exit 2 ;;
  esac
done

CODEX_DIR="${CODEX_HOME:-$HOME/.codex}"
CONFIG="$CODEX_DIR/config.toml"
SUMMARY="$CODEX_DIR/memories/memory_summary.md"

json_escape() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }

# Prints true|false|unset|invalid for [memories].generate_memories.
read_state() {
  if [ ! -f "$CONFIG" ]; then echo unset; return; fi
  if command -v python3 >/dev/null 2>&1 && python3 -c 'import tomllib' >/dev/null 2>&1; then
    python3 - "$CONFIG" <<'PY'
import sys, tomllib
try:
    with open(sys.argv[1], "rb") as f:
        data = tomllib.load(f)
except Exception:
    print("invalid"); raise SystemExit
v = data.get("memories", {})
v = v.get("generate_memories") if isinstance(v, dict) else None
print("unset" if v is None else ("true" if v is True else "false" if v is False else "invalid"))
PY
    return
  fi
  # Fallback line scan when tomllib is unavailable.
  awk '
    /^[ \t]*\[/ { t = ($0 ~ /^[ \t]*\[memories\][ \t]*(#.*)?$/); next }
    t && /^[ \t]*generate_memories[ \t]*=/ {
      v = $0; sub(/^[^=]*=[ \t]*/, "", v); sub(/[ \t]*#.*$/, "", v); sub(/[ \t]+$/, "", v)
      r = (v == "false" || v == "true") ? v : "invalid"
    }
    END { print (r == "" ? "unset" : r) }' "$CONFIG"
}

if [ ! -d "$CODEX_DIR" ]; then
  # Codex not installed: silent no-op (check mode still reports it).
  if [ "$CHECK" -eq 1 ]; then
    printf '{"codex_home":"%s","installed":false,"generate_memories":"unset","summary_present":false,"ok":true}\n' \
      "$(json_escape "$CODEX_DIR")"
  fi
  exit 0
fi

if [ "$CHECK" -eq 1 ]; then
  state="$(read_state)"
  present=false; [ -f "$SUMMARY" ] && present=true
  ok=false; if [ "$state" = false ] && [ "$present" = false ]; then ok=true; fi
  printf '{"codex_home":"%s","installed":true,"generate_memories":"%s","summary_present":%s,"ok":%s}\n' \
    "$(json_escape "$CODEX_DIR")" "$state" "$present" "$ok"
  exit 0
fi

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
state="$(read_state)"

if [ "$state" != false ]; then
  tmp="$(mktemp "${TMPDIR:-/tmp}/codex-memories.XXXXXX")" || { echo "codex-memories-config: mktemp failed" >&2; exit 1; }
  trap 'rm -f "$tmp"' EXIT
  backup=""
  if [ -f "$CONFIG" ]; then
    backup="$CONFIG.bak-$stamp"
    n=0
    while [ -e "$backup" ]; do n=$((n + 1)); backup="$CONFIG.bak-$stamp-$n"; done
    cp -p "$CONFIG" "$backup" || { echo "codex-memories-config: backup failed" >&2; exit 1; }
    awk '
      function is_header(l) { return l ~ /^[ \t]*\[/ }
      function is_mem(l)    { return l ~ /^[ \t]*\[memories\][ \t]*(#.*)?$/ }
      {
        if (is_header($0)) {
          if (in_mem && !done) { print "generate_memories = false"; done = 1 }
          in_mem = is_mem($0); seen = seen || in_mem
          print; next
        }
        if (in_mem && !done && $0 ~ /^[ \t]*generate_memories[ \t]*=/) {
          line = $0; ind = line; sub(/[^ \t].*$/, "", ind)
          cm = ""
          if (match(line, /[ \t]+#.*$/)) cm = substr(line, RSTART)
          print ind "generate_memories = false" cm; done = 1; next
        }
        print
      }
      END {
        if (in_mem && !done) { print "generate_memories = false"; done = 1 }
        if (!seen) {
          if (NR > 0) print ""
          print "[memories]"; print "generate_memories = false"
        }
      }' "$CONFIG" > "$tmp"
    # awk drops a missing trailing newline only on the last line it prints; fine.
  else
    printf '[memories]\ngenerate_memories = false\n' > "$tmp"
  fi
  cat "$tmp" > "$CONFIG" || { echo "codex-memories-config: write failed" >&2; [ -n "$backup" ] && cat "$backup" > "$CONFIG"; exit 1; }

  new_state="$(read_state)"
  if [ "$new_state" != false ]; then
    if [ -n "$backup" ]; then cat "$backup" > "$CONFIG"; else rm -f "$CONFIG"; fi
    echo "codex-memories-config: edited $CONFIG did not parse to generate_memories=false (got: $new_state); original restored" >&2
    exit 1
  fi
fi

if [ -f "$SUMMARY" ]; then
  archive_dir="$CODEX_DIR/memories-archive"
  mkdir -p "$archive_dir" || { echo "codex-memories-config: cannot create $archive_dir" >&2; exit 1; }
  dest="$archive_dir/memory_summary-$stamp.md"
  n=0
  while [ -e "$dest" ]; do n=$((n + 1)); dest="$archive_dir/memory_summary-$stamp-$n.md"; done
  mv "$SUMMARY" "$dest" || { echo "codex-memories-config: archive failed" >&2; exit 1; }
fi
exit 0
