// TJ-ARCH-MOB-001 compliant
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeSet,
    env,
    ffi::OsString,
    fs::File,
    io::{self, Read},
    path::Path,
    process::Command,
};
use zip::ZipArchive;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 || args[0] != "verify-apk" {
        eprintln!("usage: knowme-platform-native verify-apk /path/to/app.apk");
        std::process::exit(64);
    }
    let apk = Path::new(&args[1]);
    if !apk.is_file() {
        eprintln!("APK not found: {}", apk.display());
        std::process::exit(66);
    }
    let readelf = resolve_readelf()?;
    let mut archive = ZipArchive::new(File::open(apk)?).context("invalid APK ZIP archive")?;
    let scratch = tempfile::tempdir().context("create native inspection directory")?;
    let mut libraries = BTreeSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        if !name.starts_with("lib/") || !name.ends_with(".so") {
            continue;
        }
        ensure!(
            entry.enclosed_name().is_some(),
            "G2 FAIL: unsafe archive path: {name}"
        );
        let pieces: Vec<_> = name.split('/').collect();
        ensure!(
            pieces.len() == 3 && pieces[1] == "arm64-v8a",
            "G2 FAIL: APK contains non-arm64 or malformed native library: {name}; build with flutter build apk --target-platform android-arm64"
        );
        let library = pieces[2];
        ensure!(
            !library.contains('\\') && !library.contains(':'),
            "G2 FAIL: unsafe native library name: {name}"
        );
        ensure!(
            libraries.insert(library.to_owned()),
            "G2 FAIL: duplicate native library: {name}"
        );
        let lower = library.to_ascii_lowercase();
        ensure!(
            !(lower.starts_with("libopencl")
                || lower.starts_with("libgles_mali")
                || lower.starts_with("libpvrocl")
                || lower == "libvndksupport.so"),
            "G2 FAIL: APK bundles vendor GPU/OpenCL library: {name}"
        );
        // The filename is deliberately generated, not taken from the archive.
        let extracted = scratch.path().join(format!("library-{index}.so"));
        let mut target = File::create(&extracted)?;
        io::copy(&mut entry, &mut target).with_context(|| format!("extract {name}"))?;
        drop(target);
        let mut header = [0u8; 20];
        File::open(&extracted)?
            .read_exact(&mut header)
            .with_context(|| format!("G2 FAIL: {library} is not a complete ELF header"))?;
        ensure!(&header[..4] == b"\x7fELF", "G2 FAIL: {library} is not ELF");
        ensure!(header[4] == 2, "G2 FAIL: {library} must be ELF64");
        let machine = match header[5] {
            1 => u16::from_le_bytes([header[18], header[19]]),
            2 => u16::from_be_bytes([header[18], header[19]]),
            _ => bail!("G2 FAIL: {library} has invalid ELF byte order"),
        };
        ensure!(
            machine == 183,
            "G2 FAIL: {library} must contain AArch64 machine code, found ELF e_machine={machine}"
        );
        let output = Command::new(&readelf)
            .arg("-d")
            .arg(&extracted)
            .output()
            .with_context(|| format!("run {:?}", readelf))?;
        ensure!(
            output.status.success(),
            "readelf failed for {library}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            ensure!(
                !(line.contains("NEEDED")
                    && ["OpenCL", "GLES_mali", "PVROCL", "vndksupport"]
                        .iter()
                        .any(|name| line.contains(name))),
                "G1 FAIL: {library} hard-links a vendor GPU/OpenCL library: {line}"
            );
        }
    }
    for required in ["libgen_ui_ffi.so", "liblitertlm_jni.so"] {
        ensure!(
            libraries.contains(required),
            "G2 FAIL: missing lib/arm64-v8a/{required}"
        );
    }
    println!("G1 PASS: APK native libraries have no vendor GPU/OpenCL DT_NEEDED entries");
    println!("G2 PASS: APK is arm64-only and contains required app/LiteRT-LM native libraries");
    println!("G2 PASS: APK does not bundle vendor GPU/OpenCL libraries");
    println!("G3-G6 require install/launch/logcat/runtime self-test on a physical device");
    Ok(())
}

fn resolve_readelf() -> Result<OsString> {
    if let Some(path) = env::var_os("READELF") {
        ensure!(!path.is_empty(), "READELF must not be empty");
        return Ok(path);
    }
    for name in ["llvm-readelf", "readelf"] {
        if Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return Ok(name.into());
        }
    }
    if cfg!(target_os = "macos")
        && let Ok(output) = Command::new("xcrun")
            .args(["--find", "llvm-readelf"])
            .output()
        && output.status.success()
    {
        return Ok(String::from_utf8(output.stdout)?.trim().into());
    }
    bail!(
        "readelf/llvm-readelf not found; install LLVM or set READELF to the native executable path (including .exe on Windows)"
    )
}
