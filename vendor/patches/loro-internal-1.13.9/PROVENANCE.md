# Loro internal 1.13.9 source overlay

The base is the [crates.io release](https://crates.io/crates/loro-internal/1.13.9)
with SHA-256 `6fec47acf8db115b7146da5e1d5b18d0cf58032e6012c0c1ee3ec3442d1e936f`.
The crate metadata records upstream commit
[`1e529a6841b35572b05c3cb1258c0862b3aaef40`](https://github.com/loro-dev/loro/commit/1e529a6841b35572b05c3cb1258c0862b3aaef40).
The MIT license is from the same commit.

The published manifest substitutes `imbl` 7.0.2, a maintained fork of
`im` that removed the unmaintained `bitmaps` dependency in
[imbl issue 170](https://github.com/jneem/imbl/issues/170). The Loro version
vector uses the fork's generic persistent map with the existing Fx hasher
and reference-counted sharing. Serde continues to encode the version vector
as a map from peer IDs to counters. The map's internal iteration order may
differ. Both versions deserialize by key and neither stores the hash tree
structure. The archive's KBD and storage checks must exercise reopening and
replaying data written before this substitution.

The original published lock is omitted because the affected owner Cargo
roots resolve this overlay through their own locks.
