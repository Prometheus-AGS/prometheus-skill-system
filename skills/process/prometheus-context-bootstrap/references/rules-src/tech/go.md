---
paths: ['**/*.go', '**/go.mod']
---

# Go

| Tier | Commands |
|---|---|
| T0 every edit | `go vet ./...`; `go build ./...` |
| T1 unit complete | `go test -run <name> ./pkg` |
| T2 phase complete | `go test ./...` |
| T3 milestone only | race-detector runs; `-tags=integration` |

The race detector costs 5–10× memory and 2–20× time and finds races only on exercised paths — a milestone
gate. Packages by capability. No `.go` file over 500 lines: split by responsibility within the package.
