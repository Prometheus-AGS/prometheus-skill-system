#!/usr/bin/env bash
# sycophancy.sh — shared library for sycophancy detection over PMPO artifacts.
# Source this file; it does not run anything on import.
#
#   source "$(dirname "$0")/lib/sycophancy.sh"
#
# Exposes:
#   syco_find_bin                       → prints MCP binary path, or returns 1
#   syco_map_strictness <in>            → prints permissive|standard|strict
#   syco_analyze <text> <strictness>    → prints raw MCP JSON-RPC response
#   syco_score <response>               → prints sycophancy_score, or empty
#   syco_critical <response>            → prints "id:sev ..." for high/critical
#   syco_counter_path <key>             → per-artifact rejection counter path
#
# All functions degrade gracefully (empty output / non-zero) when the binary or
# python3 is unavailable; callers decide how to handle that.

syco_find_bin() {
  if command -v sycophancy-correction >/dev/null 2>&1; then
    command -v sycophancy-correction
    return 0
  fi
  local root="${CLAUDE_PLUGIN_ROOT:-}"
  [ -n "$root" ] || return 1
  local cand="${root}/skills/imported/sycophancy-correction/target/release/sycophancy-correction"
  [ -x "$cand" ] && { printf '%s' "$cand"; return 0; }
  return 1
}

syco_map_strictness() {
  case "${1:-strict}" in
    adversarial) printf 'strict' ;;
    loose)       printf 'permissive' ;;
    *)           printf '%s' "${1:-strict}" ;;
  esac
}

# syco_analyze <text> <mcp-strictness>
syco_analyze() {
  local artifact="$1" strictness="$2"
  local bin; bin="$(syco_find_bin 2>/dev/null)" || return 1
  command -v python3 >/dev/null 2>&1 || return 1

  local escaped
  escaped="$(printf '%s' "$artifact" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))' 2>/dev/null)"
  [ -n "$escaped" ] || return 1

  local fifo; fifo="$(mktemp -u /tmp/sycophancy_lib_XXXXX)"
  mkfifo "$fifo" || return 1
  # shellcheck disable=SC2064
  trap "rm -f '$fifo'" RETURN

  local skill_toml="" root="${CLAUDE_PLUGIN_ROOT:-}"
  if [ -n "$root" ] && [ -f "${root}/skills/imported/sycophancy-correction/skill.toml" ]; then
    skill_toml="${root}/skills/imported/sycophancy-correction/skill.toml"
  fi

  (
    printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"sycophancy-lib","version":"0.1.0"}}}\n'
    sleep 0.2
    printf '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}\n'
    sleep 0.1
    printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"detect_sycophancy","arguments":{"content":%s,"target":"completion","strictness":"%s"}}}\n' \
      "$escaped" "$strictness"
    sleep 3
  ) > "$fifo" &

  local response=""
  if command -v timeout >/dev/null 2>&1; then
    if [ -n "$skill_toml" ]; then
      response="$(timeout 30 "$bin" --config "$skill_toml" < "$fifo" 2>/dev/null)" || true
    else
      response="$(timeout 30 "$bin" < "$fifo" 2>/dev/null)" || true
    fi
  else
    if [ -n "$skill_toml" ]; then
      response="$("$bin" --config "$skill_toml" < "$fifo" 2>/dev/null)" || true
    else
      response="$("$bin" < "$fifo" 2>/dev/null)" || true
    fi
  fi
  printf '%s' "$response"
}

syco_score() {
  printf '%s' "$1" | python3 -c '
import sys, json
for line in sys.stdin.read().splitlines():
    try:
        obj = json.loads(line)
        r = obj.get("result", {})
        if isinstance(r, dict):
            content = r.get("content", [])
            if content and isinstance(content, list):
                data = json.loads(content[0].get("text", "{}"))
                score = data.get("sycophancy_score", -1)
                if score >= 0:
                    print(score); break
    except Exception:
        pass
' 2>/dev/null || true
}

syco_critical() {
  printf '%s' "$1" | python3 -c '
import sys, json
for line in sys.stdin.read().splitlines():
    try:
        obj = json.loads(line)
        r = obj.get("result", {})
        if isinstance(r, dict):
            content = r.get("content", [])
            if content and isinstance(content, list):
                data = json.loads(content[0].get("text", "{}"))
                classes = data.get("classifications", [])
                hc = [c.get("pattern_id","?")+":"+c.get("severity","?")
                      for c in classes if c.get("severity","") in ("high","critical")]
                if hc:
                    print(" ".join(hc))
    except Exception:
        pass
' 2>/dev/null || true
}

# Per-artifact rejection counter so reflection and artifact gates do not share
# one global counter. Key is typically a sha1 of the artifact path.
syco_counter_path() {
  printf '%s/.prometheus/reflect-rejections/%s.txt' "$HOME" "${1:-default}"
}
