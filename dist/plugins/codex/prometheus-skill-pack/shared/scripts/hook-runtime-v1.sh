#!/usr/bin/env bash
# Stable hook runtime ABI. Hook commands bind to an immutable bundle id and
# never resolve through the mutable current generation.
#
# THE POINTER FILE IS AUTHORITATIVE; THE LINK IS A CONVENIENCE
#
# This used to resolve a bundle through `bundles/<id>`, a directory symlink,
# gated on `[[ -L ]]`. Windows cannot swap a directory link atomically --
# MoveFileExW with MOVEFILE_REPLACE_EXISTING fails when the destination is a
# directory, and both directory symlinks and junctions carry the directory
# attribute -- so activation could not be made atomic while a link was the
# record of truth.
#
# The record of truth is now `pointers/bundles/<id>`, a small file holding
# `generations/<sha256>`. A file is swapped by rename, which is atomic over an
# existing file on every supported host, and a byte string can also be hashed
# and signed. The link is still created where a link primitive exists, and
# `[[ -L ]]` on it is now an ADVISORY observation rather than a gate: its
# absence is reported on stderr and resolution continues.
#
# A store written before the pointer file existed has only the link, so the
# link path is retained as a fallback and such a store is not invalidated.
set -euo pipefail

BUNDLE_ID=""
HOOK_ID=""
HARNESS=""
RESOLVE_ONLY=false

fail() {
  local code="$1"
  local message="$2"
  printf '{"status":"HOOK_RUNTIME_ERROR","code":"%s","message":"%s","bundle":"%s"}\n' \
    "$code" "$message" "$BUNDLE_ID" >&2
  exit 78
}

advise() {
  local code="$1"
  local message="$2"
  printf '{"status":"HOOK_RUNTIME_ADVISORY","code":"%s","message":"%s","bundle":"%s"}\n' \
    "$code" "$message" "$BUNDLE_ID" >&2
}

# Reduce a Windows verbatim path to its ordinary spelling.
#
# `\\?\C:\x` reaches a POSIX shell as `//?/c/x`. The two spellings of one path
# do not compare equal, so a containment check that saw one on each side would
# reject every valid bundle -- reporting an escape where there is none, which is
# worse than no check at all because it is indistinguishable from a real one in
# a log. The `?` is quoted so it is a literal, not a single-character glob.
strip_verbatim() {
  case "$1" in
    '//?/'*) printf '%s\n' "/${1#'//?/'}" ;;
    *) printf '%s\n' "$1" ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle) BUNDLE_ID="${2:-}"; shift 2 ;;
    --hook) HOOK_ID="${2:-}"; shift 2 ;;
    --harness) HARNESS="${2:-}"; shift 2 ;;
    --resolve-only) RESOLVE_ONLY=true; shift ;;
    *) fail "INVALID_ARGUMENT" "unknown argument" ;;
  esac
done

[[ "$BUNDLE_ID" =~ ^[a-f0-9]{64}$ ]] || fail "INVALID_BUNDLE" "bundle id is not sha256"

PLUGIN_ROOT="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}"
POINTER="$PLUGIN_ROOT/pointers/bundles/$BUNDLE_ID"
BUNDLE_LINK="$PLUGIN_ROOT/bundles/$BUNDLE_ID"

GENERATIONS_ROOT="$(cd "$PLUGIN_ROOT/generations" 2>/dev/null && pwd -P)" || \
  fail "BROKEN_STORE" "generation store is missing"
GENERATIONS_ROOT="$(strip_verbatim "$GENERATIONS_ROOT")"

if [[ -f "$POINTER" ]]; then
  IFS= read -r POINTER_TARGET <"$POINTER" || POINTER_TARGET=""
  POINTER_TARGET="${POINTER_TARGET%$'\r'}"
  # The pointer may name a generation and nothing else. This rejects an absolute
  # path, a traversal, and a name that is not a generation identity before any
  # filesystem call is made with it.
  [[ "$POINTER_TARGET" =~ ^generations/[a-f0-9]{64}$ ]] || \
    fail "INVALID_POINTER" "activation pointer does not name a generation"
  GENERATION_ROOT="$(cd "$PLUGIN_ROOT/$POINTER_TARGET" 2>/dev/null && pwd -P)" || \
    fail "BROKEN_BUNDLE" "activation pointer cannot be resolved"
  if [[ -e "$BUNDLE_LINK" && ! -L "$BUNDLE_LINK" ]]; then
    advise "POINTER_LINK_DEGRADED" "bundle index is present but is not a link"
  fi
elif [[ -L "$BUNDLE_LINK" ]]; then
  # A store from an installer that predates the pointer file. Resolving it keeps
  # an existing installation working across the upgrade.
  GENERATION_ROOT="$(cd "$BUNDLE_LINK" 2>/dev/null && pwd -P)" || \
    fail "BROKEN_BUNDLE" "bundle index cannot be resolved"
else
  fail "NOT_ACTIVATED" "bundle index is missing"
fi
GENERATION_ROOT="$(strip_verbatim "$GENERATION_ROOT")"

# The containment check is unchanged in strength. Both operands are fully
# resolved by `pwd -P` and reduced to one spelling, so a generation reached
# through a link, a junction, or a pointer file that points outside the store is
# still rejected here.
case "$GENERATION_ROOT" in
  "$GENERATIONS_ROOT"/*) ;;
  *) fail "ESCAPING_BUNDLE" "bundle index escapes generation store" ;;
esac

GENERATION_NAME="${GENERATION_ROOT##*/}"
[[ "$GENERATION_NAME" =~ ^[a-f0-9]{64}$ ]] || \
  fail "INVALID_GENERATION" "generation directory is not sha256"
MANIFEST="$GENERATION_ROOT/manifest.json"
[[ -f "$MANIFEST" ]] || fail "MISSING_MANIFEST" "generation manifest is missing"

manifest_value() {
  local key="$1"
  awk -F'"' -v wanted="\"$key\"" '$0 ~ wanted "[[:space:]]*:" { print $4; exit }' "$MANIFEST"
}

MANIFEST_BUNDLE="$(manifest_value bundleId)"
MANIFEST_GENERATION="$(manifest_value generation)"
MANIFEST_ABI="$(manifest_value abi)"
DISPATCHER_PATH="$(manifest_value dispatcherPath)"
DISPATCHER_SHA="$(manifest_value dispatcherSha256)"
DISPATCHER_INTERPRETER="$(manifest_value dispatcherInterpreter)"

[[ "$MANIFEST_BUNDLE" == "$BUNDLE_ID" ]] || fail "BUNDLE_MISMATCH" "manifest bundle differs"
[[ "$MANIFEST_GENERATION" == "$GENERATION_NAME" ]] || \
  fail "GENERATION_MISMATCH" "manifest generation differs"
[[ "$MANIFEST_ABI" == "hook-runtime-v1" ]] || fail "ABI_MISMATCH" "unsupported dispatcher ABI"
[[ "$DISPATCHER_PATH" == "shared/scripts/generated/hook-dispatch-v1.sh" ]] || \
  fail "DISPATCHER_PATH" "dispatcher path is not allowlisted"

# EXECUTION ELIGIBILITY COMES FROM THE MANIFEST, NOT THE FILESYSTEM
#
# This used to be `[[ -x "$DISPATCHER" ]]`. On a volume that cannot record a
# permission bit, `stat().st_mode & 0111` is zero for every file, so that gate
# rejected a dispatcher whose own signed manifest records it as executable.
# msys2 papers over it with a heuristic -- it reports `-x` for anything starting
# `#!` -- but a heuristic is not the manifest, and it does not exist off msys2.
#
# What actually makes the dispatcher safe to run is unchanged and is checked
# below: it is the file the signed receipt names, and its bytes hash to the
# digest that receipt records. It is then launched by an EXPLICIT interpreter,
# also taken from the receipt, rather than by relying on a shebang the kernel
# may or may not honour.
DISPATCHER="$GENERATION_ROOT/$DISPATCHER_PATH"
[[ -f "$DISPATCHER" ]] || fail "MISSING_DISPATCHER" "dispatcher is missing"
[[ "$DISPATCHER_INTERPRETER" == "bash" ]] || \
  fail "DISPATCHER_INTERPRETER" "dispatcher interpreter is not allowlisted"

# `shasum` is a Perl script that ships with git-bash; `sha256sum` is coreutils.
# A host can have either, both, or neither, and the previous unconditional call
# to `shasum` died with a bare "command not found" where it was absent.
# Neither tool needs its output unescaped here. GNU coreutils escapes a filename
# containing a backslash or a newline and prefixes the line with one, but
# $DISPATCHER is `pwd -P` output joined to the allowlisted literal checked just
# above, so it can contain neither. Widening that allowlist would need this
# revisited.
if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_DISPATCHER_SHA="$(sha256sum "$DISPATCHER" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL_DISPATCHER_SHA="$(shasum -a 256 "$DISPATCHER" | awk '{print $1}')"
else
  fail "MISSING_DIGEST_TOOL" "no sha256sum or shasum is available to verify the dispatcher"
fi
[[ "$ACTUAL_DISPATCHER_SHA" == "$DISPATCHER_SHA" ]] || \
  fail "DISPATCHER_HASH" "dispatcher hash differs"

if $RESOLVE_ONLY; then
  printf '{"status":"ok","bundle":"%s","generation":"%s","abi":"hook-runtime-v1"}\n' \
    "$BUNDLE_ID" "$GENERATION_NAME"
  exit 0
fi

[[ -n "$HOOK_ID" ]] || fail "MISSING_HOOK" "hook id is required"
[[ -n "$HARNESS" ]] || fail "MISSING_HARNESS" "harness is required"
command -v "$DISPATCHER_INTERPRETER" >/dev/null 2>&1 || \
  fail "MISSING_INTERPRETER" "the dispatcher interpreter is not installed on this host"
exec "$DISPATCHER_INTERPRETER" "$DISPATCHER" --hook "$HOOK_ID" --harness "$HARNESS"
