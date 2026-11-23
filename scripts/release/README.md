# Native executable pins

The source installer uses the target digest in `config/prometheus-exec-binary.json` for `prometheus-exec`. An official binary release can use a different native toolchain for a specific Full pack version and target. `releaseBuildsByPackVersion` binds each such release digest to the Rust compiler, native host, Xcode build, SDK and Git inputs. The production job prints the selected linker and Mach-O build metadata before enforcing the digest.

When bumping the Full pack version, carry an existing release entry to the new version only when the recorded exec input fingerprint and toolchain are identical. The builder compares the SHA-256 of the NUL-delimited Git tree entries for the mirrored Rust sources, WIT, Cargo configuration and build scripts. An input or toolchain change requires a fresh, explicitly reviewed release digest for that version. Preserve the source installer pins when adding a release entry.
