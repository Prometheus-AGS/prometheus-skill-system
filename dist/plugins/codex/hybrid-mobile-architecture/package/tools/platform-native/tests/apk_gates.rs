use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;
use zip::{ZipWriter, write::SimpleFileOptions};

fn library(root: &Path, name: &str, flags: &[&str]) -> Vec<u8> {
    let source = root.join("fixture.c");
    fs::write(&source, "int fixture(void) { return 42; }\n").unwrap();
    let output = root.join(name);
    let result = Command::new(std::env::var_os("CLANG").unwrap_or_else(|| "clang".into()))
        .args([
            "--target=aarch64-linux-android29",
            "-fuse-ld=lld",
            "-shared",
            "-nostdlib",
        ])
        .arg(&source)
        .args(flags)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "native fixture compilation needs LLVM clang/lld: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    fs::read(output).unwrap()
}
fn apk(root: &Path, entries: &[(&str, &[u8])]) -> Output {
    let path = root.join("app space ü.apk");
    let mut zip = ZipWriter::new(File::create(&path).unwrap());
    for (name, bytes) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    Command::new(env!("CARGO_BIN_EXE_knowme-platform-native"))
        .arg("verify-apk")
        .arg(path)
        .output()
        .unwrap()
}
#[test]
fn accepts_real_arm64_libraries_and_spaced_apk_path() {
    let dir = TempDir::new().unwrap();
    let lib = library(dir.path(), "fixture.so", &[]);
    let result = apk(
        dir.path(),
        &[
            ("lib/arm64-v8a/libgen_ui_ffi.so", &lib),
            ("lib/arm64-v8a/liblitertlm_jni.so", &lib),
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("G1 PASS"));
}
#[test]
fn rejects_wrong_abi_missing_libraries_and_bundled_vendor_code() {
    let dir = TempDir::new().unwrap();
    for (entry, expected) in [
        ("lib/x86/libgen_ui_ffi.so", "non-arm64"),
        ("lib/arm64-v8a/libOpenCL.so", "bundles vendor"),
        ("assets/empty", "missing lib/arm64-v8a"),
    ] {
        let result = apk(dir.path(), &[(entry, b"not needed")]);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
#[test]
fn rejects_vendor_needed_and_malformed_elf() {
    let dir = TempDir::new().unwrap();
    library(dir.path(), "libOpenCL.so", &["-Wl,-soname,libOpenCL.so"]);
    let search = format!("-L{}", dir.path().display());
    let lib = library(
        dir.path(),
        "linked.so",
        &[&search, "-Wl,--no-as-needed", "-lOpenCL"],
    );
    for (bytes, expected) in [
        (lib.as_slice(), "hard-links"),
        (b"invalid elf".as_slice(), "not a complete ELF header"),
    ] {
        let result = apk(dir.path(), &[("lib/arm64-v8a/libgen_ui_ffi.so", bytes)]);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn rejects_non_arm64_payload_even_under_arm64_directory() {
    let dir = TempDir::new().unwrap();
    let lib = library(dir.path(), "x86.so", &["--target=x86_64-linux-android29"]);
    let result = apk(dir.path(), &[("lib/arm64-v8a/libgen_ui_ffi.so", &lib)]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("must contain AArch64"));
}
