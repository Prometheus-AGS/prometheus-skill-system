# dsg Ecosystem Reference

## Detected Ecosystems

### Rust — `--ecosystem rust`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `~/.cargo/registry/` | Downloaded crate source archives | Yes — re-downloaded on next build |
| `~/.cargo/git/` | Git-sourced crate checkouts | Yes — re-cloned on next build |
| `**/target/debug/` | Debug build artifacts | Yes if parent has `Cargo.toml` |
| `**/target/release/` | Release build artifacts | Yes if parent has `Cargo.toml` |
| `**/target/.fingerprint/` | Incremental build state | Yes — rebuilt on next build |

**Detection marker**: `Cargo.toml` in the parent of `target/`.

**Never cleaned**:
- `~/.cargo/bin/` — installed binaries (cargo-dist, cowork, dsg, etc.)
- `~/.rustup/` — toolchain manager and toolchains themselves

**Tip**: `~/.cargo/registry` is typically the largest Rust artifact (2–10 GB).
Clean with `--min-age 30d` to preserve recently-used crates.

---

### Node.js — `--ecosystem node`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `~/.npm/_cacache/` | npm download cache | Yes — re-fetched on next install |
| `~/.pnpm/store/` | pnpm content-addressable store | Yes — re-fetched on next install |
| `~/.yarn/cache/` | Yarn 2+ download cache | Yes — re-fetched on next install |
| `**/node_modules/` | Project dependencies | Yes if parent has `package.json` |

**Detection marker**: `package.json` in parent of `node_modules/`.

**Edge cases**:
- pnpm uses hardlinks from `~/.pnpm/store` into `node_modules/` — cleaning the
  store invalidates all projects using those packages. `dsg` detects pnpm workspaces
  and warns before cleaning the store.
- `.yarn/cache/` contains `.zip` archives — dsg treats them as reclaimable.

---

### Python — `--ecosystem python`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `~/.cache/pip/` | pip download cache | Yes — re-downloaded on next install |
| `**/__pycache__/` | Bytecode cache | Yes — regenerated on next run |
| `**/*.pyc` / `**/*.pyo` | Compiled bytecode files | Yes — regenerated on next import |
| `**/.venv/`, `**/venv/`, `**/env/` | Virtual environments | Yes if project is not active |

**Detection markers**:
- `__pycache__`: parent must contain `*.py` files
- `.venv`: parent must contain `pyproject.toml` or `requirements.txt`
- `*.pyc`: always safe if marker file `.py` exists alongside

**Edge case**: Active virtual environments (currently activated in shell via `VIRTUAL_ENV` env var) are **skipped automatically**.

---

### Go — `--ecosystem go`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `$GOPATH/pkg/mod/cache/` | Downloaded module cache | Yes — re-downloaded on next build |
| `$GOPATH/pkg/mod/` (specific versions) | Extracted module source | Yes — re-extracted on next build |

**Detection**: Respects `$GOPATH`. Defaults to `~/go/` if `$GOPATH` unset.

**Note**: Go module cache uses read-only permissions on source files. `dsg` calls
`chmod -R u+w` before trashing, matching the behavior of `go clean -modcache`.

---

### Docker — `--ecosystem docker`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `/var/lib/docker/overlay2/` | Image layer storage (Linux) | Yes — images re-pulled when needed |
| `/var/lib/docker/volumes/` | Named volumes | **WARNING**: may contain data |
| Docker Desktop VM disk (macOS) | All Docker data | Use Docker Desktop UI instead |

**Requires**: `sudo` on Linux for `/var/lib/docker/` access.

**macOS note**: Docker Desktop on macOS uses a VM disk image — `dsg` detects
this and defers to `docker system prune` instead of directly manipulating paths.

**WARNING on volumes**: Named Docker volumes may contain application data.
`dsg` flags them with a confirmation prompt even with `--force`.

---

### Xcode — `--ecosystem xcode`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `~/Library/Developer/Xcode/DerivedData/` | Build products for all projects | Yes — rebuilt on next build |
| `~/Library/Developer/Xcode/Archives/` | Distribution archives | **Caution** — may contain signed archives |
| `~/Library/Caches/com.apple.dt.Xcode/` | Xcode internal caches | Yes |

**DerivedData** is typically 10–50 GB and safe to remove. Xcode rebuilds it
on the next build (takes 2–5 minutes for medium-sized projects).

**Archives** contain `.xcarchive` bundles used for App Store submission.
`dsg` applies a 30-day minimum age for archives even when `--min-age` is shorter.

---

### Homebrew — `--ecosystem homebrew`

| Path | Purpose | Safe to clean? |
|------|---------|----------------|
| `~/Library/Caches/Homebrew/` | Formula download cache | Yes — re-downloaded on next install |
| `/usr/local/Cellar/` (old versions) | Outdated formula versions | Yes via `brew cleanup` |

**Note**: `dsg` calls `brew cleanup --prune=all` for Homebrew rather than
directly trashing brew-managed paths, to respect brew's internal state machine.
This is the only ecosystem where `dsg` delegates to a first-party tool.

---

## Adding a Custom Ecosystem

Custom ecosystem detection is not yet supported in v1.0. File an issue at
`https://github.com/GQAdonis/disk-space-guardian/issues` to request a new
ecosystem detector.

Planned for v1.1: a `[custom_ecosystems]` section in `~/.config/dsg/config.toml`
allowing users to define marker paths and candidate paths for project-specific
caches (e.g., Gradle, Maven, Bazel).
