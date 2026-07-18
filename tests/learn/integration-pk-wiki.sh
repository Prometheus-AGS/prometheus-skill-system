#!/usr/bin/env bash
# Real pk CLI round trip: source -> OKF wiki -> index/log -> search/focus/lint.

set -euo pipefail

PK_BIN="${PK_BIN:-pk}"
command -v "$PK_BIN" >/dev/null 2>&1 || {
  echo "[FAIL] pk binary not found: $PK_BIN" >&2
  exit 1
}

TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT
KB_DIR="$TMP_ROOT/kb"
SOURCE_FILE="$TMP_ROOT/source.md"

cat > "$SOURCE_FILE" <<'EOF'
# Tower middleware ordering

Tower layers wrap from the outside in. Request flow follows layer construction
order while response flow unwinds in reverse. Authentication must precede
handlers and middleware that require an established identity.
EOF

INGEST_OUTPUT="$($PK_BIN ingest --kb-dir "$KB_DIR" --source integration-proof "$SOURCE_FILE")"
grep -q 'compiled' <<< "$INGEST_OUTPUT"

PAGE="$(find "$KB_DIR/wiki" -maxdepth 1 -type f \
  ! -name index.md ! -name log.md -print -quit)"
[[ -n "$PAGE" && -f "$PAGE" ]]
grep -q '^type:' "$PAGE"
grep -q '^id:' "$PAGE"
grep -q '^title:' "$PAGE"
grep -q '^sources:' "$PAGE"
grep -q 'integration-proof' "$PAGE"

[[ -f "$KB_DIR/wiki/index.md" ]]
[[ -f "$KB_DIR/wiki/log.md" ]]
grep -q "$(basename "$PAGE")" "$KB_DIR/wiki/index.md"
grep -q 'Creation' "$KB_DIR/wiki/log.md"

LIST_OUTPUT="$($PK_BIN list --kb-dir "$KB_DIR")"
SEARCH_OUTPUT="$($PK_BIN search --kb-dir "$KB_DIR" tower)"
FOCUS_OUTPUT="$($PK_BIN focus --kb-dir "$KB_DIR" 'tower middleware authentication ordering' --k 3)"
LINT_OUTPUT="$($PK_BIN lint --kb-dir "$KB_DIR")"

grep -qi 'tower' <<< "$LIST_OUTPUT"
grep -qi 'tower' <<< "$SEARCH_OUTPUT"
grep -qi 'authentication' <<< "$FOCUS_OUTPUT"
grep -Eq 'issue\(s\)|no issues|clean|OK' <<< "$LINT_OUTPUT"

echo "[PASS] pk source ingest created an OKF wiki page plus index and log"
echo "[PASS] pk list/search/focus retrieved the compiled knowledge"
echo "[PASS] pk lint completed against the generated wiki"
