---
type: SessionRecord
id: karpathy-session-d6cc1b02a37bf0ae
title: Karpathy session d6cc1b02a37b
tags:
- karpathy
- session-learning
sources:
- session:f5f07e47-e85d-4ba4-8f43-ad39a3df3f5d
timestamp: 2026-08-31T11:42:27.376380+00:00
created_at: 2026-08-31T11:42:27.376380+00:00
updated_at: 2026-08-31T11:42:27.376380+00:00
revision: 0
---

## Delta

I have what I need. Here's the picture from the actual code.

## Short answer: neither. You want **Git Bash/MSYS2 as the primary target**, with a small number of true Windows-native pieces — not batch equivalents of 439 scripts, and not WSL as a requirement.

## What the repo actually looks like

| Surface | Count | Windows story |
|---|---|---|
| `SKILL.md` files | 313 | 282 are manifest-only — **already portable**, nothing executes |
| Skills referencing a `.sh` | 31 | the real portability problem |
| Bash scripts total | 439 | most are installer/CI/validation, not per-invocation |
| launchd plists | 12 | **no Windows analogue at all** — this is the hard part |
| Symlink calls in the flat installer | 2 | `ln -s` — and Codex already can't follow symlinks anyway |

Two findings worth flagging:

**1. Windows support was already started and is half-wired.** [`scripts/check-prerequisites.sh:67`](scripts/check-prerequisites.sh:67) detects `MINGW*|MSYS*|CYGWIN*|Windows_NT` and sets `EXE=".exe"`. [`scripts/install-binaries.sh:439`](scripts/install-binaries.sh:439) maps to `x86_64-pc-windows-msvc`.

**2. But it dead-ends.** [`install-binaries.sh:476`](scripts/install-binaries.sh:476) says `run scripts/install-binaries.ps1 instead` — **that file does not exist.** A Windows user following the existing code path hits a `fail` pointing at a missing script. That's a live bug, not a design gap.

## Why not batch files

Batch/PowerShell rewrites of 439 bash scripts would give you two implementations to keep in sync forever, and the pack has already been bitten by exactly this class of drift (the `liter-llm` installer abort documented in CLAUDE.md). The scripts also use `set -euo pipefail`, process substitution, and `jq` pipelines that have no clean `.bat` translation.

## Why not WSL-only

WSL is a reasonable *option*, but as a *requirement* it breaks the thing you actually care about: skills that touch the user's real Windows filesystem and Windows-native harnesses. Claude Code on Windows runs as a Windows process with `C:\Users\...` paths; a WSL-side hook sees `/mnt/c/...`. Every path in the pack would need translation at the boundary. It also can't drive Windows-native Codex/Cursor skill directories.

## The recommended split

**Tier 1 — Git Bash / MSYS2 (covers ~90% of the surface).** Ships with Git for Windows, which any user of this pack already has. Nearly all 439 scripts run unmodified. Cost: audit for the bash-3.2-isms already documented in CLAUDE.md (`mapfile`, `declare -A`) — which you've already had to do for macOS launchd, so that work is largely done.

**Tier 2 — true Windows-native, small and unavoidable:**

- `scripts/install-binaries.ps1` — the file already referenced but missing. Needed because zip extraction and `Unblock-File`/execution policy are genuinely PowerShell territory.
- **Service supervision.** The 12 launchd plists have no bash equivalent. Windows needs either Scheduled Tasks (`schtasks`) or Service Control Manager entries. This is a real port, not a translation.
- **Symlinks.** `ln -s` on Windows needs Developer Mode or elevation. Since Codex already gets real directory copies via [`scripts/codex-sync-skills.sh`](scripts/codex-sync-skills.sh), the cleanest move is to make **all** Windows targets use the copy path plus the existing sync-on-interval mechanism — reuse, not new code.

**Tier 3 — the 31 script-bearing skills.** These already have a documented classification (`classify-mobile-execution.sh`, E0/E1/E2/R). Windows is structurally the same problem as mobile: an environment where spawning your script may not work. The E0/R classes (remote execution, drive-a-paired-desktop) apply directly.

## One caution

Your mobile work found that E1 was defined as a residual class and **all 18 of 18 members were misclassified**. If you build a Windows compatibility matrix, don't define any tier as "everything else" — hand-verify a sample of whatever the leftover bucket is.

## Suggested first step

Before any porting

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f5f07e47-e85d-4ba4-8f43-ad39a3df3f5d
- Captured: 2026-08-31T11:42:12.173687Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
