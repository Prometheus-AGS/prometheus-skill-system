---
name: disk-space-guardian
version: '1.0.0'
license: MIT
description: >
  Intelligent, safety-first disk space management for developer workstations.
  Scans and safely removes stale Rust target/, Node node_modules, Python
  __pycache__, Go module cache, Docker layers, Xcode DerivedData, and Homebrew
  caches using the dsg CLI. Defaults to dry-run; moves artifacts to system
  Trash, never rm.
metadata:
  author: Travis James
  category: devops
  tags: [disk, cache, cleanup, rust, devtools, devops, dsg, build-cache]
---

# disk-space-guardian

Use the `dsg` CLI to reclaim disk space from stale build artifacts and
development caches — safely, predictably, and reversibly.

## When to use

- A developer asks "how do I free up disk space?" or "my disk is full"
- Running `/dsg` or mentioning "clean caches", "disk space", "build artifacts"
- Disk space drops below a comfortable threshold (typically < 20 GB free)
- Before a large build that needs more headroom
- Periodic maintenance (weekly/monthly automated runs)

## Quick Start

```bash
# 1. Check what's reclaimable without touching anything (always safe)
dsg status

# 2. Scan for stale artifacts with size breakdown
dsg scan

# 3. Deep scan — search across all home subdirectories
dsg scan --deep

# 4. Preview exactly what would be cleaned (dry-run, never deletes)
dsg clean --dry-run

# 5. Actually clean (moves to Trash — recoverable from macOS Trash or ~/.local/share/Trash)
dsg clean --force

# 6. Clean only one ecosystem
dsg clean --force --ecosystem rust
dsg clean --force --ecosystem node
```

## Safety First

`dsg` is built on a non-negotiable safety model:

| Rule | Detail |
|------|--------|
| **Dry-run default** | Every path previews first; `--force` required to move anything |
| **Trash, not rm** | Files go to system Trash — recoverable from macOS Trash or `~/.local/share/Trash` |
| **Activity check** | `lsof` + `fuser` verify no process holds the files open before cleaning |
| **Age guard** | Files younger than `--min-age` (default 24h) are never touched |
| **Protected paths** | `~/.cargo/bin`, `~/.local/bin`, SIP paths, home root are never scanned |
| **Marker-based detection** | Only treats a directory as a cache if a known marker exists |

**Never run `dsg clean --force` without first reviewing `dsg scan` output.** The dry-run output lists every path that would be moved.

See [Safety Reference](references/SAFETY.md) for the full rule set and edge cases.

## Ecosystem Detection

`dsg` auto-detects 7 development ecosystems:

| Ecosystem | Detected Paths |
|-----------|---------------|
| **Rust** | `~/.cargo/registry`, `~/.cargo/git`, `**/target/debug`, `**/target/release` |
| **Node** | `~/.npm/_cacache`, `~/.pnpm/store`, `**/node_modules` |
| **Python** | `~/.cache/pip`, `**/__pycache__`, `**/*.pyc`, `**/.venv`, `**/venv`, `**/env` |
| **Go** | `~/go/pkg/mod/cache` (respects `$GOPATH`) |
| **Docker** | `/var/lib/docker/` overlay/volumes (requires sudo on Linux) |
| **Xcode** | `~/Library/Developer/Xcode/DerivedData`, `~/Library/Developer/Xcode/Archives` |
| **Homebrew** | `~/Library/Caches/Homebrew` |

Filter to a single ecosystem with `--ecosystem <name>`:
```bash
dsg scan --ecosystem rust       # Rust only
dsg scan --ecosystem node       # Node only
dsg scan --ecosystem python     # Python only
```

See [Ecosystem Reference](references/ECOSYSTEMS.md) for detection marker details and known edge cases.

## Activity Verification

Before cleaning any path, `dsg` verifies it is safe to remove:

1. **lsof check**: confirms no process has a file descriptor open in the directory
2. **fuser check**: cross-verifies on Linux with `fuser`
3. **git status check**: if the directory contains a `.git/`, verifies no uncommitted changes

If any check fails, the path is **skipped with a warning** — never forcibly cleaned.

```
SKIP  ~/.cargo/registry/src/github.com-1/serde-1.0.195  (held open by rustc PID 48291)
```

## Retention Policies

Configure retention in `~/.config/dsg/config.toml` or pass flags:

```bash
# Keep artifacts newer than 7 days (default: 24h)
dsg clean --force --min-age 7d

# Keep artifacts newer than 30 days
dsg clean --force --min-age 30d

# Exclude specific paths from cleaning
dsg scan --exclude ~/.cargo/registry/src/github.com-1/my-private-crate
```

Default retention: **24 hours** — anything older than 24h is a candidate.

Recommended production policy:
- `target/debug`: 7d (rebuilt frequently; keep recent)
- `target/release`: 30d (slower to rebuild; keep longer)
- `.npm/_cacache`: 30d
- `~/.cargo/registry`: 90d (large but slow to re-download)

## Automation Setup

### Daily cron (dry-run report only)

```bash
# Add to crontab: daily at 6am, log to ~/.dsg-report.log
0 6 * * * /usr/local/bin/dsg scan --json >> ~/.dsg-report.log 2>&1
```

### Weekly cleanup with 7-day retention

```bash
# Weekly Sunday at 3am — auto-clean artifacts older than 7 days
0 3 * * 0 /usr/local/bin/dsg clean --force --min-age 7d >> ~/.dsg-clean.log 2>&1
```

### macOS launchd (recommended over cron on macOS)

```xml
<!-- ~/Library/LaunchAgents/com.prometheus.dsg.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.prometheus.dsg</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/you/.local/bin/dsg</string>
    <string>clean</string><string>--force</string><string>--min-age</string><string>7d</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict><key>Weekday</key><integer>0</integer><key>Hour</key><integer>3</integer></dict>
  <key>StandardOutPath</key><string>/tmp/dsg.log</string>
  <key>StandardErrorPath</key><string>/tmp/dsg-err.log</string>
</dict>
</plist>
```

```bash
launchctl load ~/Library/LaunchAgents/com.prometheus.dsg.plist
```

## Knowledge Logging

After a clean run, `dsg` can emit a structured JSON report for integration
with `pk` (prometheus-knowledge) or any log aggregator:

```bash
# JSON report — pipe to pk or save for auditing
dsg scan --json | pk ingest --type disk-report

# Human-readable summary
dsg scan
```

Example JSON output:

```json
{
  "scan_time": "2026-07-04T14:00:00Z",
  "total_reclaimable_bytes": 29985000000,
  "entries": [
    { "path": "~/.cargo/registry", "size_bytes": 3450000000, "ecosystem": "rust", "last_modified": "2026-06-28T..." },
    { "path": "~/.npm/_cacache",   "size_bytes": 14420000000, "ecosystem": "node", "last_modified": "2026-07-01T..." }
  ]
}
```

## Troubleshooting

### `dsg` not found

```bash
# Install via prometheus-skill-pack install script (recommended, installs v0.1.4+)
bash /path/to/prometheus-skill-pack/scripts/install-binaries.sh

# Or build from source (submodule must be initialized)
cd tools/disk-space-guardian && cargo build --release
cp target/release/dsg ~/.local/bin/dsg
```

### Permission denied on Docker paths

Docker paths under `/var/lib/docker/` require root access on Linux:
```bash
sudo dsg scan --ecosystem docker
sudo dsg clean --force --ecosystem docker
```
On macOS, Docker Desktop manages its own VM disk — use Docker Desktop's
"Purge data" option for Docker cleanup instead of dsg.

### Scan is slow

Use `--ecosystem` to narrow the scan:
```bash
dsg scan --ecosystem rust       # 3–5× faster than full scan
```
Or use `--min-size 100MB` to skip small artifacts:
```bash
dsg scan --min-size 100MB
```

### A cleaned path was needed

All deletions go to system Trash — recover from macOS Finder Trash or:
```bash
# Linux
ls ~/.local/share/Trash/files/
mv ~/.local/share/Trash/files/my-artifact ~/restore/path/
```

### False positive — path detected as cache but isn't

Add an exclude rule:
```bash
dsg scan --exclude ~/projects/my-special-target
```
Or add permanently in `~/.config/dsg/config.toml`:
```toml
[scan]
exclude = ["~/projects/my-special-target"]
```
