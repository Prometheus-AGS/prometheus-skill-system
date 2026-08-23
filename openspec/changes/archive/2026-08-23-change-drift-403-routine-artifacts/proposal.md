## Why

Seven routine files: 3 modified .prometheus wiki logs, 2 untracked karpathy-session files,
plus `.devin/` and `.agents/skills/.openspec-target`. An earlier draft claimed a standing
authorization to always commit .prometheus session logs — that authorization is recorded in
the HMA repository's CLAUDE.md and does NOT transfer here. This repo has C-02 instead.

## What Changes

- Scan the session logs against C-02 before committing; do not inherit a prior scan result.
- Commit the session logs.
- Decide `.devin/`: tracked or gitignored.
- Decide `.agents/skills/.openspec-target`: if tracked, it is arguably generated under C-01.

## Impact

- C-02 governs: session logs are exactly the artifact class that can capture a secret.
