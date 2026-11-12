# Keyring 3.6.3 source overlay

The base is the [crates.io release](https://crates.io/crates/keyring/3.6.3)
with SHA-256 `eebcc3aff044e5944a8fbaf69eb277d11986064cba30c468730e8b9909fb551c`.
The crate metadata records upstream commit
[`315cbdf6c6a9153d8c9f88b56568f29862d3e39d`](https://github.com/open-source-cooperative/keyring-rs/commit/315cbdf6c6a9153d8c9f88b56568f29862d3e39d).
The Apache-2.0 and MIT license files are included.

The new `sync-crypto-rust` feature enables Rust crypto on the existing
synchronous D-Bus Secret Service backend. Both session-open call sites select
Diffie-Hellman when that feature is used with `sync-secret-service`. The old
`crypto-rust` feature also addresses the asynchronous backend and added its
unmaintained Zbus support libraries to the owner lockfiles. The original
feature remains available for other callers. The KBD caller continues to
select the same backend, encryption mode and credential naming on Linux.
Apple and Windows continue to use their native backends.

The original published lock is omitted because the affected owner Cargo
roots resolve this overlay through their own locks.
