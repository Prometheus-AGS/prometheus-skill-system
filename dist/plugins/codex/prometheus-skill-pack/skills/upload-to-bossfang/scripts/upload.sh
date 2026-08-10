#!/usr/bin/env bash
# upload-to-bossfang — POST a .lf-skill.zip to a LibreFang instance.
#
# Defense-in-depth SSRF guard:
#   1. Scheme allowlist (http(s) only)
#   2. No embedded credentials in URL
#   3. DNS resolved to an IP that is NOT private/loopback/metadata
#   4. Host:port appears in ~/.config/prometheus-skill-pack/bossfang-allowlist.toml
#      (skipped under --insecure, but only the allowlist — IP check still runs)
#   5. curl --resolve pins the validated IP (defeats DNS rebinding)
#   6. curl --max-redirs 0 (redirects could bypass IP validation)
#   7. BOSSFANG_TOKEN never reaches stdout/stderr
#
# Reviewed under change-005 of phase-compliance-and-power-multiplier.

# `set -e` enabled but `set -x` deliberately not used — would echo the
# Authorization header. If a developer wants a trace, they should add it
# locally and remember to rotate the token.
set -euo pipefail

# Disable bash xtrace if accidentally enabled by parent shell — protect the token.
set +x

# ── arg parsing ──────────────────────────────────────────────────────────────
URL=""
ZIP=""
INSECURE=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --zip)        ZIP="$2"; shift 2 ;;
        --insecure)   INSECURE=true; shift ;;
        --help|-h)
            sed -n '2,15p' "$0" | sed 's/^# //;s/^#//'
            exit 0
            ;;
        -*)           echo "Unknown flag: $1" >&2; exit 2 ;;
        *)            if [ -z "$URL" ]; then URL="$1"; shift; else echo "Extra positional arg: $1" >&2; exit 2; fi ;;
    esac
done

if [ -z "$URL" ]; then
    echo "Usage: upload.sh <url> [--zip <path>] [--insecure]" >&2
    exit 2
fi

# ── Step 1: Resolve zip ──────────────────────────────────────────────────────
if [ -z "$ZIP" ]; then
    # Glob *.lf-skill.zip in cwd. Use mapfile to avoid word-splitting hazards.
    matches=()
    while IFS= read -r -d '' f; do matches+=("$f"); done < <(find . -maxdepth 1 -name "*.lf-skill.zip" -print0 2>/dev/null)
    case ${#matches[@]} in
        0) echo "❌ No *.lf-skill.zip found in cwd. Pass --zip <path> or run forge package-librefang first." >&2; exit 4 ;;
        1) ZIP="${matches[0]}" ;;
        *) echo "❌ Multiple *.lf-skill.zip in cwd. Pass --zip explicitly:" >&2; printf '   %s\n' "${matches[@]}" >&2; exit 4 ;;
    esac
fi
if [ ! -f "$ZIP" ]; then
    echo "❌ Zip not found: $ZIP" >&2
    exit 4
fi
# Argument-injection guard: zip paths starting with '-' could become curl flags.
case "$ZIP" in -*) echo "❌ Zip path must not start with '-': $ZIP" >&2; exit 4 ;; esac

# ── Step 2: Extract skill name (zip-bomb-safe) ───────────────────────────────
# Limit unzip output to 64 KB — skill.toml is always tiny; if it isn't, the
# zip is malicious. `dd bs=65536 count=1` caps the read regardless of compressed size.
SKILL_TOML="$(unzip -p "$ZIP" skill.toml 2>/dev/null | dd bs=65536 count=1 2>/dev/null || true)"
if [ -z "$SKILL_TOML" ]; then
    echo "❌ Zip $ZIP has no skill.toml at root" >&2
    exit 5
fi
SKILL_NAME="$(printf '%s' "$SKILL_TOML" | grep -m1 -E '^[[:space:]]*name[[:space:]]*=' | sed -E 's/.*"([^"]+)".*/\1/')"
if ! [[ "$SKILL_NAME" =~ ^[a-z0-9][a-z0-9-]+$ ]]; then
    echo "❌ skill.toml has invalid name: ${SKILL_NAME:-<empty>}" >&2
    exit 5
fi

# ── Step 3: SSRF pipeline ────────────────────────────────────────────────────

# 3a. Scheme allowlist. Match exactly "http://" or "https://".
case "$URL" in
    http://*|https://*) ;;
    *) echo "❌ URL must be http:// or https://" >&2; exit 2 ;;
esac

# 3b. No embedded credentials.
case "$URL" in
    *@*) echo "❌ URL must not contain user:pass@ — pass auth via BOSSFANG_TOKEN env var" >&2; exit 2 ;;
esac

# 3c. Parse host:port. Strip scheme, then split on first '/'.
no_scheme="${URL#http://}"; no_scheme="${no_scheme#https://}"
host_port="${no_scheme%%/*}"
HOST="${host_port%%:*}"
if [ "$host_port" = "$HOST" ]; then
    case "$URL" in
        https://*) PORT=443 ;;
        http://*)  PORT=80  ;;
    esac
else
    PORT="${host_port##*:}"
fi
if ! [[ "$HOST" =~ ^[A-Za-z0-9.-]+$ ]] && ! [[ "$HOST" =~ ^\[[0-9a-fA-F:]+\]$ ]]; then
    echo "❌ Invalid hostname: $HOST" >&2; exit 2
fi
if ! [[ "$PORT" =~ ^[0-9]+$ ]] || [ "$PORT" -lt 1 ] || [ "$PORT" -gt 65535 ]; then
    echo "❌ Invalid port: $PORT" >&2; exit 2
fi

# 3d. DNS resolution + private-IP check. Resolve all A/AAAA records.
RESOLVED_IPS=""
if command -v getent >/dev/null 2>&1; then
    RESOLVED_IPS="$(getent ahosts "$HOST" 2>/dev/null | awk '{print $1}' | sort -u)"
elif command -v dig >/dev/null 2>&1; then
    RESOLVED_IPS="$(dig +short A "$HOST"; dig +short AAAA "$HOST")"
elif command -v host >/dev/null 2>&1; then
    RESOLVED_IPS="$(host "$HOST" | awk '/has address|has IPv6 address/{print $NF}')"
else
    echo "❌ Cannot resolve DNS — install getent (glibc), dig, or host" >&2
    exit 2
fi
if [ -z "$RESOLVED_IPS" ]; then
    echo "❌ DNS lookup failed for $HOST" >&2; exit 2
fi

is_private_ip() {
    local ip="$1"
    case "$ip" in
        # IPv4
        127.*|10.*|169.254.*|0.*) return 0 ;;
        172.1[6-9].*|172.2[0-9].*|172.3[0-1].*) return 0 ;;
        192.168.*) return 0 ;;
        100.6[4-9].*|100.[7-9][0-9].*|100.1[0-1][0-9].*|100.12[0-7].*) return 0 ;;
        22[4-9].*|23[0-9].*|24[0-9].*|25[0-5].*) return 0 ;;
        # IPv6
        ::1|::ffff:127.*|::ffff:10.*|::ffff:192.168.*|::ffff:172.*) return 0 ;;
        fc*|fd*|fe8*|fe9*|fea*|feb*) return 0 ;;
    esac
    return 1
}

# Locally-explicit exception: localhost:4545 (LibreFang dev port) requires --insecure.
LOCALHOST_DEV=false
if [ "$HOST" = "localhost" ] || [ "$HOST" = "127.0.0.1" ] || [ "$HOST" = "::1" ]; then
    if [ "$PORT" = "4545" ] && $INSECURE; then
        LOCALHOST_DEV=true
    fi
fi

# Pick the first non-private IP for --resolve pinning. Reject if all are private.
PINNED_IP=""
for ip in $RESOLVED_IPS; do
    if is_private_ip "$ip"; then
        if $LOCALHOST_DEV; then
            PINNED_IP="$ip"
            break
        fi
        echo "❌ DNS resolved to private/reserved IP: $ip — refusing to upload" >&2
        echo "   (use --insecure ONLY for http://localhost:4545 dev work)" >&2
        exit 2
    elif [ -z "$PINNED_IP" ]; then
        PINNED_IP="$ip"
    fi
done
if [ -z "$PINNED_IP" ]; then
    echo "❌ No usable IP after SSRF filter for $HOST" >&2; exit 2
fi

# 3e. Allowlist (skipped under --insecure).
if ! $INSECURE; then
    ALLOWLIST="${PROMETHEUS_BOSSFANG_ALLOWLIST:-$HOME/.config/prometheus-skill-pack/bossfang-allowlist.toml}"
    if [ ! -f "$ALLOWLIST" ]; then
        echo "❌ Allowlist not found: $ALLOWLIST" >&2
        echo "   Create it (see references/bossfang-allowlist.example.toml) or pass --insecure (after review)" >&2
        exit 3
    fi
    # Permission hygiene: refuse to read an allowlist that anyone but the user
    # can modify. A world-writable allowlist is a privilege-escalation vector
    # for any local process that wants to whitelist an arbitrary host.
    perms="$(stat -f '%A' "$ALLOWLIST" 2>/dev/null || stat -c '%a' "$ALLOWLIST" 2>/dev/null || echo "")"
    if [ -n "$perms" ]; then
        # Strip any sticky/setuid digits — keep just the trailing 3.
        last3="${perms: -3}"
        group_w="$(printf '%s' "$last3" | cut -c2)"
        other_w="$(printf '%s' "$last3" | cut -c3)"
        # Each octal digit's bit-2 (value 2,3,6,7) means write permission.
        case "$group_w$other_w" in
            *[2367]*) echo "❌ Allowlist $ALLOWLIST is group/world-writable (mode $perms)" >&2
                      echo "   Run: chmod 600 $ALLOWLIST" >&2
                      exit 3 ;;
        esac
    fi
    # Naive but effective TOML scan: look for a [[allowed]] table containing
    # this exact host AND port. Don't pull in a TOML parser to keep the script
    # dep-free; for production fleets, prefer the Rust CLI form (forge upload-bossfang).
    if ! awk -v h="$HOST" -v p="$PORT" '
        /^\[\[allowed\]\]/ { in_block=1; host=""; port=""; next }
        /^\[/ && !/^\[\[allowed\]\]/ { in_block=0 }
        in_block && /^[[:space:]]*host[[:space:]]*=/ { gsub(/.*"|".*/,""); host=$0 }
        in_block && /^[[:space:]]*port[[:space:]]*=/ { gsub(/.*=[[:space:]]*|[[:space:]]*$/,""); port=$0 }
        in_block && host==h && port==p { print "MATCH"; exit }
    ' "$ALLOWLIST" | grep -q MATCH; then
        echo "❌ $HOST:$PORT not in allowlist $ALLOWLIST" >&2
        echo "   Add an [[allowed]] entry, or pass --insecure (after review)" >&2
        exit 3
    fi
fi

# ── Step 4: Upload ──────────────────────────────────────────────────────────
# Token handling: pass via `--header @<file>` so it never appears in argv
# (where `ps aux` and bash xtrace can see it). The header file is created
# in a strict-mode mktemp, mode 600, and unlinked on EXIT via the same
# trap that cleans CURL_LOG.
AUTH_HEADER_FILE=""
AUTH_ARGS=()
if [ -n "${BOSSFANG_TOKEN:-}" ]; then
    AUTH_HEADER_FILE="$(mktemp -t bossfang-hdr-XXXXXX)"
    chmod 600 "$AUTH_HEADER_FILE"
    printf 'Authorization: Bearer %s\n' "$BOSSFANG_TOKEN" >"$AUTH_HEADER_FILE"
    AUTH_ARGS=(--header "@$AUTH_HEADER_FILE")
fi

# Capture curl output so we can scrub the auth header from any error display.
# `2>&1` then sed scrubs would be the wrong order — never let stdout grep see
# the token in the first place. Instead, log curl's stderr to a file and post-
# process it before showing.
CURL_LOG="$(mktemp -t bossfang-XXXXXX)"
# Cleanup BOTH the curl log and any auth header file. Token leak prevention
# matters even on script failure paths.
cleanup() {
    rm -f "$CURL_LOG" "$CURL_LOG.out" "$CURL_LOG.err" "${AUTH_HEADER_FILE:-/dev/null}"
}
trap cleanup EXIT

CURL_BASE_ARGS=(
    --proto "=http,https"
    --proto-redir "=http,https"   # if redirects ever get followed, still scheme-restricted
    --no-location                 # explicitly do not follow 30x redirects
    --max-redirs 0                # belt-and-suspenders for --no-location
    --connect-timeout 10
    --max-time 60
    --resolve "$HOST:$PORT:$PINNED_IP"
    --fail-with-body
    --silent
    --show-error
)

echo "📦 Uploading $ZIP → $URL/skills/install (skill: $SKILL_NAME)"
if ! curl "${CURL_BASE_ARGS[@]}" "${AUTH_ARGS[@]}" \
        -X POST \
        -H "Content-Type: application/zip" \
        --data-binary "@$ZIP" \
        "$URL/skills/install" >"$CURL_LOG.out" 2>"$CURL_LOG.err"; then
    sed -E 's/Authorization:[^ ]+ [^ ]+/Authorization: <REDACTED>/g' "$CURL_LOG.err" >&2
    echo "❌ Upload failed" >&2
    exit 6
fi

# ── Step 5: Reload ───────────────────────────────────────────────────────────
curl "${CURL_BASE_ARGS[@]}" "${AUTH_ARGS[@]}" \
    -X POST "$URL/skills/reload" >/dev/null 2>"$CURL_LOG.err" || {
        sed -E 's/Authorization:[^ ]+ [^ ]+/Authorization: <REDACTED>/g' "$CURL_LOG.err" >&2
        echo "⚠️  Skill installed but reload failed — call /skills/reload manually" >&2
    }

# ── Step 6: Verify ───────────────────────────────────────────────────────────
echo "🔎 Verifying $URL/skills/$SKILL_NAME"
if ! curl "${CURL_BASE_ARGS[@]}" "${AUTH_ARGS[@]}" \
        -X GET "$URL/skills/$SKILL_NAME" >"$CURL_LOG.out" 2>"$CURL_LOG.err"; then
    sed -E 's/Authorization:[^ ]+ [^ ]+/Authorization: <REDACTED>/g' "$CURL_LOG.err" >&2
    echo "❌ Verification GET failed" >&2
    exit 7
fi

# Pretty-print without ever including the auth header.
if command -v jq >/dev/null 2>&1; then
    jq . <"$CURL_LOG.out"
else
    cat "$CURL_LOG.out"; echo
fi
echo "✅ $SKILL_NAME installed and verified"
