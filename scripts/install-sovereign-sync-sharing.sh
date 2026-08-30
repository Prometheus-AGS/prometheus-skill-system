#!/usr/bin/env bash
# Install the optional sovereign-sync binary only for an explicit sharing setup.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --help|-h)
            echo "usage: $0 [--dry-run]"
            exit 0
            ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

MANIFEST="$REPO_ROOT/substrate/sovereign-sync/Cargo.toml"
DEST_DIR="$HOME/.local/bin"
DEST="$DEST_DIR/sovereign-sync"

if $DRY_RUN; then
    echo "[dry-run] cargo build --release --manifest-path $MANIFEST"
    echo "[dry-run] install sovereign-sync to $DEST"
    exit 0
fi

if pgrep -x cargo >/dev/null 2>&1 || pgrep -x rustc >/dev/null 2>&1; then
    echo "ERROR: another Cargo/rustc process is active; sharing binary installation is serialized" >&2
    exit 1
fi

cargo build --release --manifest-path "$MANIFEST"
TARGET_DIR="$(cargo metadata --no-deps --format-version 1 --manifest-path "$MANIFEST" \
    | node -e 'let s=""; process.stdin.on("data",d=>s+=d).on("end",()=>process.stdout.write(JSON.parse(s).target_directory))')"
SOURCE="$TARGET_DIR/release/sovereign-sync"
[ -x "$SOURCE" ] || { echo "ERROR: built sovereign-sync artifact missing at $SOURCE" >&2; exit 1; }

mkdir -p "$DEST_DIR"
STAGED="$(mktemp "$DEST_DIR/.sovereign-sync.install.XXXXXX")"
install -m 755 "$SOURCE" "$STAGED"
if [ "$(uname -s)" = "Darwin" ]; then
    codesign --force --sign - "$STAGED" >/dev/null
    codesign --verify "$STAGED"
fi
mv -f "$STAGED" "$DEST"
echo "Installed optional sharing binary: $DEST"
