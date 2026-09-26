# Native platform artifact gates

`knowme-platform-native` replaces host shell extraction of APK libraries with
Rust ZIP handling and invokes the native LLVM/ELF inspector with argument arrays.
It does not compile an application or certify device execution.

```text
cargo install --locked --path tools/platform-native
knowme-platform-native verify-apk application.apk
```

Generated `scripts/android/verify-native-inference-gates.mjs` invokes this binary
from PATH. Set `KNOWME_PLATFORM_NATIVE` to an explicit executable path if needed.
Install LLVM with `llvm-readelf` on PATH, or set `READELF` to its full path. GNU
`readelf` is also accepted. Neither Bash, Python, unzip nor ripgrep is required.

Windows releases must build this crate separately for
`x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`, using the appropriate MSVC
and Windows SDK environment. `rustup target add` alone does not install the native
linker or LLVM. ARM64 support requires a native runner receipt before certification.

The gate preserves the arm64-only APK policy, required app/LiteRT-LM libraries,
vendor GPU/OpenCL bundle rejection and DT_NEEDED dependency inspection. Unsafe or
duplicate archive paths are rejected. Archive names are never used as extraction
destinations. A failed native inspector is a gate failure, not a successful empty
search. G3–G6 still require a physical Android device and model self-test evidence.

Public CLI tests use real ARM64 ELF files compiled with LLVM clang and lld, then
inspect them with the actual readelf executable. Put those three native tools on
PATH (`CLANG` and `READELF` may select explicit executable paths):

```text
cargo clippy --manifest-path tools/platform-native/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tools/platform-native/Cargo.toml --locked
```

These fixtures need no Android SDK. They cover valid APKs with Unicode/spaced
paths, wrong ABI, missing libraries, bundled vendor libraries, hard-linked vendor
DT_NEEDED entries, and malformed ELF inspection failures.
