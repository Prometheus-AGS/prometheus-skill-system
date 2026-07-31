# Gap 3 — negative-search evidence, captured 2026-07-31
A negative claim ('nothing checks these') needs the search itself, not a summary.

## 1. No fabric-integration skill
```console
$ ls -d skills/*/fabric-integration
ls: skills/*/fabric-integration: No such file or directory
```

## 2. No version-checking references in CI or scripts
```console
$ grep -rln "loro\|wasmtime\|iroh" .github/workflows/ scripts/*.sh
(grep exit code: 1 — 1 means NO MATCHES)
```

## 3. iroh >= 1.0.2 IS enforced, via Cargo
```console
$ grep -n "^iroh" substrate/storage-provider/Cargo.toml substrate/sovereign-sync/Cargo.toml
substrate/storage-provider/Cargo.toml:24:iroh = { version = "1.0.2", optional = true }
substrate/storage-provider/Cargo.toml:25:iroh-blobs = { version = "0.103", default-features = false, features = ["fs-store"], optional = true }
substrate/storage-provider/Cargo.toml:26:iroh-docs = { version = "0.101", default-features = false, features = ["fs-store"], optional = true }
substrate/storage-provider/Cargo.toml:27:iroh-gossip = { version = "0.101", default-features = false, features = ["net"], optional = true }
substrate/storage-provider/Cargo.toml:39:iroh-docs-backend = ["dep:iroh", "dep:iroh-blobs", "dep:iroh-docs", "dep:iroh-gossip", "dep:n0-future"]
substrate/sovereign-sync/Cargo.toml:45:iroh = "1.0.2"
substrate/sovereign-sync/Cargo.toml:46:iroh-gossip = "0.101"
```
Archived change: openspec/changes/archive/2026-07-31-change-idt-008-feature-gate-iroh-docs
