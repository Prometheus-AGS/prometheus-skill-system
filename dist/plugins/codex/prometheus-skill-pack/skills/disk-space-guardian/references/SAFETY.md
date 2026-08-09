# dsg Safety Reference

## Non-Negotiable Safety Rules

These rules are hardcoded in the dsg binary. They cannot be overridden without
an explicit escape flag.

### 1. Dry-run is the default

Every clean path previews first. Running `dsg clean` without `--force` always
exits with code 2 and a preview listing. The user must explicitly pass `--force`
to move anything.

```bash
dsg clean              # exits 2 — prints preview, never moves anything
dsg clean --dry-run    # same as above — explicit dry-run flag
dsg clean --force      # actually moves artifacts to Trash
```

### 2. Trash, not rm

`dsg` uses the `trash` crate (Rust) which calls the OS trash API:
- **macOS**: moves to `~/.Trash/` via `NSFileManager.trashItem`
- **Linux**: moves to `~/.local/share/Trash/files/` (FreeDesktop Trash spec)
- **Windows**: moves to Recycle Bin via `SHFileOperation`

`std::fs::remove_file` and `std::fs::remove_dir_all` are **never called**.
This is enforced at the code level — the only deletion function is `trash::delete()`.

### 3. Activity verification before any clean

For each path queued for removal, dsg runs:

1. **lsof** — `lsof +D <path>` — if any process has an open file descriptor,
   the path is skipped with a warning
2. **fuser** (Linux only) — cross-check with `fuser -m <path>`
3. **Git status** — if the path contains a `.git/` directory, `git -C <path> status --porcelain`
   must return empty output (no uncommitted changes)

If any check fails → **SKIP** (logged to stderr, not an error).

### 4. Age guard

Files and directories younger than `--min-age` (default: 24h) are never
candidates for cleaning. This prevents removing artifacts from an active build.

The age is measured by `mtime` (last modified time), not `ctime` or `atime`.

### 5. Protected paths (hardcoded exclusions)

These paths are never entered regardless of `--ecosystem` or `--path` flags:

| Path | Reason |
|------|--------|
| `~/.cargo/bin/` | Installed Rust binaries |
| `~/.local/bin/` | Installed CLI binaries |
| `~/.rustup/` | Rust toolchain manager |
| `/usr/`, `/bin/`, `/sbin/`, `/System/`, `/Library/` | System paths |
| `$HOME` (the dir itself) | Never scan the home root directly |
| Any path with SIP protection (macOS) | System Integrity Protection |
| Paths shorter than 4 path components | Too close to root — refuse |

### 6. Conservative ecosystem detection

`dsg` only treats a directory as a build cache if it contains a **known marker**:

- Rust `target/`: parent directory must have `Cargo.toml`
- `node_modules/`: parent must have `package.json`
- Python `__pycache__/`: contains `.pyc` files and parent has `*.py`
- Go `pkg/mod/cache/`: path matches `$GOPATH/pkg/mod/cache` pattern

A directory that matches by name alone but lacks the marker is **skipped**.

## Error Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success — scan or clean completed |
| 1 | Unexpected error (IO error, permission denied on root path) |
| 2 | Dry-run exit — preview printed, nothing moved (not an error in CI) |
| 3 | No artifacts found matching the criteria |

## Audit Trail

Every clean run appends to `~/.local/share/dsg/audit.log`:

```
2026-07-04T14:05:11Z  CLEAN  ~/.cargo/registry/src/github.com-1/serde-1.0.100  1.2MB  rust
2026-07-04T14:05:11Z  SKIP   ~/.cargo/registry/src/github.com-1/tokio-1.35.0   held-open:rustc/48291
2026-07-04T14:05:11Z  CLEAN  ~/.npm/_cacache/content-v2/sha512/...              340KB  node
```

Fields: `timestamp  action  path  size  ecosystem`

Actions: `CLEAN` (moved to Trash), `SKIP` (excluded by rule + reason), `DRY` (dry-run preview).

## Recovery Procedure

If a cleaned artifact was needed:

```bash
# macOS — open Trash in Finder
open ~/.Trash

# Linux — list trash contents
ls ~/.local/share/Trash/files/

# Linux — restore specific item
mv ~/.local/share/Trash/files/target ~/.local/share/Trash/files/target.bak
# (trashinfo file at ~/.local/share/Trash/info/target.trashinfo has the original path)

# Rebuild the artifact naturally
cargo build --release
```

The audit log at `~/.local/share/dsg/audit.log` lists every moved path with
its original location, size, and timestamp.
