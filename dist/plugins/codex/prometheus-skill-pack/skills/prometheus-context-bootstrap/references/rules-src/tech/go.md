---
paths: ['**/*.go', '**/go.mod']
---

# Go

Batch implementation until a production path is complete. Use a narrow build or vet command earlier only
to unblock work. At a completed change boundary, run the smallest integration target that exercises the
real package entry point and external collaborators, commonly a repository selector with
`-tags=integration`. Unit, filtered-function, mock-only, and per-edit tests are not completion evidence.
Reserve broad race-detector and workspace gates for the final applicable phase or release.

The race detector costs 5–10× memory and 2–20× time and finds races only on exercised paths — a final
gate. Packages by capability. No `.go` file over 500 lines: split by responsibility within the package.
