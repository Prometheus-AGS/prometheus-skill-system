#!/usr/bin/env bash
# liter-llm-mcp-launcher.integration.test.sh — the liter-llm MCP launcher, the
# installer that moves old harness entries onto it, and every writer of the entry.
#
# Everything runs through the production entry points against scratch homes; the
# real home is never written. Handshake scenarios start the REAL liter-llm binary
# and complete an MCP `initialize` over stdio with LITER_LLM_MASTER_KEY absent
# from the caller's environment. Without a binary they are NOT_EXERCISED, never
# passed.
#
#   LITER_LLM_TEST_BIN      liter-llm binary (default: ~/.local/bin/liter-llm, then PATH)
#   LITER_LLM_TEST_CONFIG   config to link into the scratch home (default: the pack template)
#   LITER_LLM_TEST_SECRETS  secrets file to link (default: a generated test value)
#
# bash 3.2 compatible. Exit 0 when no scenario FAILED.

set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER="$REPO_ROOT/scripts/install-liter-llm-mcp-launcher.mjs"
LAUNCHER_SOURCE="$REPO_ROOT/shared/scripts/liter-llm-mcp-launch.sh"
HANDSHAKE="$REPO_ROOT/scripts/tests/lib/mcp-initialize.mjs"
NODE="$(command -v node)"
MINIMAL_PATH="/usr/bin:/bin:/usr/sbin:/sbin"
REAL_HOME="$HOME"

TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

PASSED=0
FAILED=0
SKIPPED=0
pass() { PASSED=$((PASSED + 1)); printf 'PASS           %s\n' "$1"; }
fail() { FAILED=$((FAILED + 1)); printf 'FAIL           %s\n' "$1"; [ -z "${2:-}" ] || printf '               %s\n' "$2"; }
skip() { SKIPPED=$((SKIPPED + 1)); printf 'NOT_EXERCISED  %s (%s)\n' "$1" "$2"; }
check() { # check <name> <command...>
    _name="$1"; shift
    if _out="$("$@" 2>&1)"; then pass "$_name"; else fail "$_name" "$(printf '%s' "$_out" | tail -n 3 | tr '\n' ' ')"; fi
}

BIN="${LITER_LLM_TEST_BIN:-}"
if [ -z "$BIN" ]; then
    if [ -x "$REAL_HOME/.local/bin/liter-llm" ]; then BIN="$REAL_HOME/.local/bin/liter-llm"
    else BIN="$(command -v liter-llm 2>/dev/null || true)"; fi
fi
[ -n "$BIN" ] && [ -x "$BIN" ] || BIN=""

digest() { shasum -a 256 "$1" | awk '{print $1}'; }
mode_of() { stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"; }
json_get() { # json_get <file> <js expression over `c`>
    "$NODE" -e 'const c=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8")); const v=eval(process.argv[2]); process.stdout.write(typeof v==="string"?v:JSON.stringify(v));' "$1" "$2"
}
toml_command() { # the command of the liter-llm table in a TOML config
    awk '/^\[(mcp_servers|mcp\.servers)\.liter-llm\]/{t=1;next} /^\[/{t=0} t && /^command[[:space:]]*=/{sub(/^command[[:space:]]*=[[:space:]]*"/,""); sub(/".*$/,""); print; exit}' "$1"
}

# provision <home> — the runtime a launch needs: binary, config, secrets file.
provision() {
    mkdir -p "$1/.local/bin" "$1/.config/liter-llm" "$1/.prometheus/kbd"
    [ -z "$BIN" ] || ln -sf "$BIN" "$1/.local/bin/liter-llm"
    if [ -n "${LITER_LLM_TEST_CONFIG:-}" ]; then ln -sf "$LITER_LLM_TEST_CONFIG" "$1/.config/liter-llm/liter-llm-proxy.toml"
    else cp "$REPO_ROOT/shared/config/liter-llm-proxy.toml" "$1/.config/liter-llm/liter-llm-proxy.toml"; fi
    if [ -n "${LITER_LLM_TEST_SECRETS:-}" ]; then ln -sf "$LITER_LLM_TEST_SECRETS" "$1/.prometheus/kbd/secrets.env"
    else
        printf 'export LITER_LLM_MASTER_KEY="test-only-%s"\n' "$$" > "$1/.prometheus/kbd/secrets.env"
        chmod 600 "$1/.prometheus/kbd/secrets.env"
    fi
}

# in_home <home> <command...> — run as a harness started from the Dock would:
# an empty environment, a minimal PATH, and no LITER_LLM_MASTER_KEY.
in_home() { _h="$1"; shift; env -i HOME="$_h" PATH="$MINIMAL_PATH" "$@"; }
handshake() { _h="$1"; shift; in_home "$_h" "$NODE" "$HANDSHAKE" -- "$@"; }
run_installer() { _h="$1"; shift; env -u CODEX_HOME HOME="$_h" "$NODE" "$INSTALLER" --home "$_h" "$@"; }

# ── 1. Migration ─────────────────────────────────────────────────────────────
H="$TMP_ROOT/migrate"
provision "$H"
CFG="$H/.config/liter-llm/liter-llm-proxy.toml"
mkdir -p "$H/.claude" "$H/.opencode" "$H/.minimax/mcp" "$H/.cursor" "$H/.codeium/windsurf/mcp" \
    "$H/.codex" "$H/.kimi-code" "$H/.config/codex"

# Old pack-written forms: ${HOME} literal, absolute path, no --config, `~`.
cat > "$H/.claude/mcp.json" <<'JSON'
{ "mcpServers": { "other": { "command": "x" }, "liter-llm": { "type": "stdio", "command": "liter-llm",
  "args": ["mcp", "--transport", "stdio", "--config", "${HOME}/.config/liter-llm/liter-llm-proxy.toml"] } } }
JSON
cat > "$H/.opencode/opencode.json" <<JSON
{ "mcp": { "liter-llm": { "type": "local", "enabled": true,
  "command": ["liter-llm", "mcp", "--transport", "stdio", "--config", "$CFG"] } } }
JSON
cat > "$H/.minimax/mcp/mcp.json" <<'JSON'
{ "mcpServers": { "liter-llm": { "command": "liter-llm", "args": ["mcp", "--transport", "stdio"] } } }
JSON
cat > "$H/.claude/mcp_servers.json" <<'JSON'
{ "mcpServers": { "liter-llm": { "command": "liter-llm",
  "args": ["mcp", "--transport", "stdio", "--config", "~/.config/liter-llm/liter-llm-proxy.toml"] } } }
JSON
cat > "$H/.codex/config.toml" <<TOML
model = "x"

[mcp_servers.before]
command = "before"

[mcp_servers.liter-llm]
command = "liter-llm"
# --config is REQUIRED.
args = ["mcp", "--transport", "stdio", "--config", "$CFG"]

[mcp_servers.after]
command = "after"
args = ["keep"]
TOML
chmod 600 "$H/.codex/config.toml"
cat > "$H/.kimi-code/config.toml" <<TOML
[mcp_servers.liter-llm]
command = "liter-llm"
args = [
    "mcp",
    "--transport",
    "stdio",
    "--config",
    "$CFG",
]
TOML
# A user's own entries: another binary, an env block, another config path.
cat > "$H/.cursor/mcp.json" <<JSON
{ "mcpServers": { "liter-llm": { "command": "/opt/custom/liter-llm",
  "args": ["mcp", "--transport", "stdio", "--config", "$CFG"] } } }
JSON
cat > "$H/.codeium/windsurf/mcp/mcp.json" <<'JSON'
{ "mcpServers": { "liter-llm": { "command": "liter-llm", "args": ["mcp", "--transport", "stdio"],
  "env": { "LITER_LLM_MASTER_KEY": "user-supplied" } } } }
JSON
cat > "$H/.config/codex/config.toml" <<'TOML'
[mcp.servers.liter-llm]
command = "liter-llm"
args = ["mcp", "--transport", "stdio", "--config", "/somewhere/else/liter-llm-proxy.toml"]
TOML

ALL_CONFIGS=".claude/mcp.json .opencode/opencode.json .minimax/mcp/mcp.json .claude/mcp_servers.json .codex/config.toml .kimi-code/config.toml .cursor/mcp.json .codeium/windsurf/mcp/mcp.json .config/codex/config.toml"
CUSTOM_CONFIGS=".cursor/mcp.json .codeium/windsurf/mcp/mcp.json .config/codex/config.toml"
snapshot() { for _f in $ALL_CONFIGS; do printf '%s %s\n' "$(digest "$H/$_f")" "$_f"; done; }
BEFORE="$(snapshot)"
CUSTOM_BEFORE="$(for _f in $CUSTOM_CONFIGS; do digest "$H/$_f"; done)"
LAUNCHER="$H/.local/bin/liter-llm-mcp"

run_installer "$H" --check >/dev/null 2>&1; _rc=$?
if [ "$_rc" -eq 1 ] && [ "$(snapshot)" = "$BEFORE" ] && [ ! -e "$LAUNCHER" ]; then pass "migration: --check reports pending work (exit 1) and writes nothing"
else fail "migration: --check reports pending work (exit 1) and writes nothing" "exit $_rc"; fi

run_installer "$H" --dry-run >/dev/null 2>&1; _rc=$?
if [ "$_rc" -eq 0 ] && [ "$(snapshot)" = "$BEFORE" ] && [ ! -e "$LAUNCHER" ]; then pass "migration: --dry-run exits 0 and writes nothing"
else fail "migration: --dry-run exits 0 and writes nothing" "exit $_rc"; fi

check "migration: apply exits 0" run_installer "$H"

if [ -x "$LAUNCHER" ] && cmp -s "$LAUNCHER" "$LAUNCHER_SOURCE"; then pass "launcher installed, executable, identical to its source"
else fail "launcher installed, executable, identical to its source"; fi

[ "$(json_get "$H/.claude/mcp.json" 'c.mcpServers["liter-llm"]')" = "{\"type\":\"stdio\",\"command\":\"$LAUNCHER\"}" ] \
    && [ "$(json_get "$H/.claude/mcp.json" 'c.mcpServers.other.command')" = "x" ] \
    && pass "claude-style entry (\${HOME} form) moved; sibling entry kept" \
    || fail "claude-style entry (\${HOME} form) moved; sibling entry kept" "$(json_get "$H/.claude/mcp.json" 'c.mcpServers')"
[ "$(json_get "$H/.opencode/opencode.json" 'c.mcp["liter-llm"]')" = "{\"type\":\"local\",\"enabled\":true,\"command\":[\"$LAUNCHER\"]}" ] \
    && pass "opencode-style entry moved" || fail "opencode-style entry moved" "$(json_get "$H/.opencode/opencode.json" 'c.mcp')"
[ "$(json_get "$H/.minimax/mcp/mcp.json" 'c.mcpServers["liter-llm"]')" = "{\"command\":\"$LAUNCHER\"}" ] \
    && pass "entry without --config moved" || fail "entry without --config moved"
[ "$(json_get "$H/.claude/mcp_servers.json" 'c.mcpServers["liter-llm"].command')" = "$LAUNCHER" ] \
    && pass "entry with ~ config path moved" || fail "entry with ~ config path moved"

if [ "$(toml_command "$H/.codex/config.toml")" = "$LAUNCHER" ] \
    && ! awk '/^\[mcp_servers\.liter-llm\]/{t=1;next} /^\[/{t=0} t' "$H/.codex/config.toml" | grep -q '^args' \
    && grep -q '^model = "x"$' "$H/.codex/config.toml" \
    && [ "$(grep -c -E '^command = "(before|after)"$' "$H/.codex/config.toml")" -eq 2 ] \
    && grep -q '^args = \["keep"\]$' "$H/.codex/config.toml" \
    && [ "$(mode_of "$H/.codex/config.toml")" = "600" ]; then pass "TOML table moved; neighbouring tables and mode 0600 kept"
else fail "TOML table moved; neighbouring tables and mode 0600 kept" "$(tr '\n' '|' < "$H/.codex/config.toml")"; fi
if [ "$(toml_command "$H/.kimi-code/config.toml")" = "$LAUNCHER" ] && ! grep -q -E '^[[:space:]]*(args|"|\])' "$H/.kimi-code/config.toml"; then
    pass "TOML table with a multi-line args array moved"
else fail "TOML table with a multi-line args array moved" "$(tr '\n' '|' < "$H/.kimi-code/config.toml")"; fi

if [ "$(for _f in $CUSTOM_CONFIGS; do digest "$H/$_f"; done)" = "$CUSTOM_BEFORE" ]; then pass "customised entries (other binary, env block, other config path) left byte-identical"
else fail "customised entries (other binary, env block, other config path) left byte-identical"; fi

_backups="$(find "$H" -name '*.bak.*' | wc -l | tr -d ' ')"
_codex_backup="$(find "$H/.codex" -name 'config.toml.bak.*' | head -n 1)"
if [ "$_backups" -eq 6 ] && [ -n "$_codex_backup" ] && [ "$(mode_of "$_codex_backup")" = "600" ]; then pass "one backup per rewritten file (6), keeping the original mode"
else fail "one backup per rewritten file (6), keeping the original mode" "found $_backups"; fi

AFTER="$(snapshot)"
run_installer "$H" >/dev/null 2>&1; _rc=$?
run_installer "$H" --check >/dev/null 2>&1; _rc_check=$?
if [ "$_rc" -eq 0 ] && [ "$_rc_check" -eq 0 ] && [ "$(snapshot)" = "$AFTER" ] \
    && [ "$(find "$H" -name '*.bak.*' | wc -l | tr -d ' ')" -eq 6 ]; then pass "second run changes nothing; --check then exits 0"
else fail "second run changes nothing; --check then exits 0" "apply $_rc, check $_rc_check"; fi

H2="$TMP_ROOT/subtable"
mkdir -p "$H2/.codex"
printf '[mcp_servers.liter-llm]\ncommand = "liter-llm"\nargs = ["mcp", "--transport", "stdio"]\n\n[mcp_servers.liter-llm.env]\nFOO = "bar"\n' > "$H2/.codex/config.toml"
_sub_before="$(digest "$H2/.codex/config.toml")"
run_installer "$H2" >/dev/null 2>&1
[ "$(digest "$H2/.codex/config.toml")" = "$_sub_before" ] && pass "TOML entry with an env sub-table left alone" \
    || fail "TOML entry with an env sub-table left alone"

H3="$TMP_ROOT/foreign"
mkdir -p "$H3/.local/bin" "$H3/.minimax/mcp"
printf '#!/bin/sh\necho not ours\n' > "$H3/.local/bin/liter-llm-mcp"
cp "$H/.minimax/mcp/mcp.json.bak."* "$H3/.minimax/mcp/mcp.json"
_foreign_before="$(digest "$H3/.local/bin/liter-llm-mcp") $(digest "$H3/.minimax/mcp/mcp.json")"
run_installer "$H3" >/dev/null 2>&1; _rc=$?
if [ "$_rc" -eq 1 ] && [ "$(digest "$H3/.local/bin/liter-llm-mcp") $(digest "$H3/.minimax/mcp/mcp.json")" = "$_foreign_before" ]; then
    pass "an unrelated file named liter-llm-mcp is not overwritten (exit 1); no entry is moved onto it"
else fail "an unrelated file named liter-llm-mcp is not overwritten (exit 1); no entry is moved onto it" "exit $_rc"; fi

# ── 2. Launcher failures: a message on stderr, a non-zero exit, stdout empty ──
expect_failure() { # expect_failure <name> <exit> <stderr pattern> <home> [VAR=value...]
    _name="$1"; _want="$2"; _pattern="$3"; _h="$4"; shift 4
    _stdout="$(env -i HOME="$_h" PATH="$MINIMAL_PATH" "$@" "$LAUNCHER_SOURCE" 2>"$TMP_ROOT/stderr" </dev/null)"; _rc=$?
    if [ "$_rc" -eq "$_want" ] && [ -z "$_stdout" ] && grep -q -- "$_pattern" "$TMP_ROOT/stderr"; then pass "$_name"
    else fail "$_name" "exit $_rc, stdout '$_stdout', stderr: $(tr '\n' ' ' < "$TMP_ROOT/stderr")"; fi
}
F="$TMP_ROOT/failures"
provision "$F"
[ -n "$BIN" ] || { printf '#!/bin/sh\nexit 0\n' > "$F/.local/bin/liter-llm"; chmod +x "$F/.local/bin/liter-llm"; }
FAKE_SECRETS="$TMP_ROOT/no-such-secrets.env"
expect_failure "secrets file absent: exit 78, message names the file" 78 "$FAKE_SECRETS does not exist" "$F" \
    PROMETHEUS_KBD_SECRETS="$FAKE_SECRETS" LITER_LLM_CONFIG="$REPO_ROOT/shared/config/liter-llm-proxy.toml"
printf 'export UNRELATED=1\n' > "$TMP_ROOT/empty-secrets.env"
expect_failure "secrets file lacks the master key: exit 78, message names the file" 78 "$TMP_ROOT/empty-secrets.env does not define it" "$F" \
    PROMETHEUS_KBD_SECRETS="$TMP_ROOT/empty-secrets.env" LITER_LLM_CONFIG="$REPO_ROOT/shared/config/liter-llm-proxy.toml"
expect_failure "liter-llm binary missing: exit 127" 127 "liter-llm binary not found" "$F" LITER_LLM_BIN="$TMP_ROOT/no-such-binary"
expect_failure "config file missing: exit 66, message names it" 66 "$TMP_ROOT/no-such-config.toml" "$F" LITER_LLM_CONFIG="$TMP_ROOT/no-such-config.toml"

printf 'echo NOISE-ON-STDOUT\nexport LITER_LLM_MASTER_KEY="test-only-noisy"\n' > "$TMP_ROOT/noisy-secrets.env"
_stdout="$(env -i HOME="$F" PATH="$MINIMAL_PATH" PROMETHEUS_KBD_SECRETS="$TMP_ROOT/noisy-secrets.env" \
    LITER_LLM_CONFIG="$REPO_ROOT/shared/config/liter-llm-proxy.toml" /bin/bash "$LAUNCHER_SOURCE" --check 2>"$TMP_ROOT/stderr")"; _rc=$?
if [ "$_rc" -eq 0 ] && [ -z "$_stdout" ] && grep -q 'NOISE-ON-STDOUT' "$TMP_ROOT/stderr"; then pass "--check under /bin/bash: exit 0, stdout empty even when the secrets file prints"
else fail "--check under /bin/bash: exit 0, stdout empty even when the secrets file prints" "exit $_rc, stdout '$_stdout'"; fi

# ── 3. Writers of the entry ──────────────────────────────────────────────────
W="$TMP_ROOT/writers"
mkdir -p "$W/.claude" "$W/.opencode" "$W/.minimax/mcp" "$W/.cursor" "$W/.codex" "$W/.kimi-code"
for _f in .claude/mcp.json .opencode/opencode.json .minimax/mcp/mcp.json .cursor/mcp.json; do printf '{}\n' > "$W/$_f"; done
: > "$W/.codex/config.toml"; : > "$W/.kimi-code/config.toml"
if env -u CODEX_HOME HOME="$W" TAVILY_API_KEY="test-only" bash "$REPO_ROOT/scripts/configure-mcp-all-tools.sh" >"$TMP_ROOT/writers.out" 2>&1; then
    WL="$W/.local/bin/liter-llm-mcp"
    if [ -x "$WL" ] \
        && [ "$(json_get "$W/.claude/mcp.json" 'c.mcpServers["liter-llm"]')" = "{\"type\":\"stdio\",\"command\":\"$WL\"}" ] \
        && [ "$(json_get "$W/.opencode/opencode.json" 'c.mcp["liter-llm"].command')" = "[\"$WL\"]" ] \
        && [ "$(json_get "$W/.minimax/mcp/mcp.json" 'c.mcpServers["liter-llm"].command')" = "$WL" ] \
        && [ "$(json_get "$W/.cursor/mcp.json" 'c.mcpServers["liter-llm"].command')" = "$WL" ] \
        && [ "$(toml_command "$W/.codex/config.toml")" = "$WL" ] \
        && [ "$(toml_command "$W/.kimi-code/config.toml")" = "$WL" ]; then pass "configure-mcp-all-tools.sh installs the launcher and writes it into all six configs"
    else fail "configure-mcp-all-tools.sh installs the launcher and writes it into all six configs" "$(tr '\n' '|' < "$W/.codex/config.toml")"; fi
else fail "configure-mcp-all-tools.sh installs the launcher and writes it into all six configs" "$(tail -n 3 "$TMP_ROOT/writers.out" | tr '\n' ' ')"; fi

B="$TMP_ROOT/bridge"
mkdir -p "$B"
if env -i HOME="$B" PATH="$(dirname "$NODE"):$MINIMAL_PATH" bash "$REPO_ROOT/skills/process/liter-llm-bridge/scripts/configure-mcp.sh" \
        register --harness cursor >"$TMP_ROOT/bridge.out" 2>&1 \
    && cmp -s "$B/.local/bin/liter-llm-mcp" "$LAUNCHER_SOURCE" \
    && [ "$(json_get "$B/.cursor/mcp.json" 'c.mcpServers["liter-llm"]')" = "{\"command\":\"$B/.local/bin/liter-llm-mcp\"}" ]; then
    pass "/liter-llm-bridge register installs the launcher and writes it"
else fail "/liter-llm-bridge register installs the launcher and writes it" "$(tail -n 3 "$TMP_ROOT/bridge.out" | tr '\n' ' ')"; fi

# ── 4. MCP initialize handshakes against the real binary ─────────────────────
PLUGIN_ENTRY() { json_get "$REPO_ROOT/dist/plugins/$1/.mcp.json" 'c.mcpServers["liter-llm"].args[1]'; }
if [ -z "$BIN" ]; then
    for _name in "migrated claude-style entry" "migrated opencode-style entry" "migrated Codex TOML entry" \
        "plugin .mcp.json entry (launcher in ~/.local/bin)" "plugin .mcp.json entry (copy inside the plugin)" \
        "repository .codex/config.toml entry"; do skip "handshake: $_name" "no liter-llm binary; set LITER_LLM_TEST_BIN"; done
else
    check "handshake: migrated claude-style entry" handshake "$H" "$(json_get "$H/.claude/mcp.json" 'c.mcpServers["liter-llm"].command')"
    check "handshake: migrated opencode-style entry" handshake "$H" "$(json_get "$H/.opencode/opencode.json" 'c.mcp["liter-llm"].command[0]')"
    check "handshake: migrated Codex TOML entry" handshake "$H" "$(toml_command "$H/.codex/config.toml")"
    # A harness that does not substitute ${CLAUDE_PLUGIN_ROOT} runs the string as it is.
    check "handshake: plugin .mcp.json entry (launcher in ~/.local/bin)" handshake "$H" /bin/sh -c "$(PLUGIN_ENTRY codex/prometheus-skill-pack)"
    # Claude Code substitutes ${CLAUDE_PLUGIN_ROOT}; this home has no ~/.local/bin launcher.
    P="$TMP_ROOT/plugin-only"
    provision "$P"
    _entry="$(PLUGIN_ENTRY claude/prometheus-runtime)"
    _root="$REPO_ROOT/dist/plugins/claude/prometheus-runtime"
    check "handshake: plugin .mcp.json entry (copy inside the plugin)" handshake "$P" /bin/sh -c "$(printf '%s' "$_entry" | sed "s|\${CLAUDE_PLUGIN_ROOT}|$_root|g")"
    _codex_script="$(awk '/^\[mcp_servers\.liter-llm\]/{t=1;next} /^\[/{t=0} t && /^args/' "$REPO_ROOT/.codex/config.toml" \
        | "$NODE" -e 'const l=require("fs").readFileSync(0,"utf8"); process.stdout.write(JSON.parse(l.slice(l.indexOf("[")))[1]);')"
    check "handshake: repository .codex/config.toml entry" handshake "$H" /bin/sh -c "$_codex_script"
fi

printf '\n%d passed, %d failed, %d not exercised\n' "$PASSED" "$FAILED" "$SKIPPED"
[ "$FAILED" -eq 0 ]
