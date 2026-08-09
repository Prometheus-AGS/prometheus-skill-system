---
paths: ['**/*.go', '**/go.mod']
---

# Go

Loaded when a Go file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `go vet ./...`; `go build ./...` |
| T1 unit complete | `go test -run <name> ./pkg` |
| T2 phase complete | `go test ./...` |
| T3 milestone only | `go test -race ./...`; integration (`-tags=integration`) |

## Hard rules

- Race detection costs 5-10x memory and 2-20x execution time, and only finds
  races on paths the test actually exercises. It is a milestone gate, not a
  continuous check.
- Errors are values. Wrap with context at the boundary, do not swallow.

<!-- Replace the commands above with this project's real ones if they differ. -->
