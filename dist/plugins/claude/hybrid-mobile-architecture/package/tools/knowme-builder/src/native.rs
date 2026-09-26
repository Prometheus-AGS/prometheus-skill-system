use std::{fs, path::Path, process::Command};

use crate::model::CommandResult;

/// Inspect prerequisites without installing tools or claiming application certification.
pub(crate) fn inspect(root: &Path, targets: &[String], result: &mut CommandResult) {
    for (tool, args) in [("rustc", vec!["-vV"]), ("cargo", vec!["--version"])] {
        match Command::new(tool).args(args).current_dir(root).output() {
            Ok(output) if output.status.success() => result
                .actions
                .push(String::from_utf8_lossy(&output.stdout).trim().to_owned()),
            _ => {
                result.ok = false;
                result
                    .warnings
                    .push(format!("native tool unavailable: {tool}"));
            }
        }
    }
    if targets.is_empty() {
        result.warnings.push("no cross-target requested; native library and application build readiness are unverified".to_owned());
        return;
    }
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    for target in targets {
        if installed.lines().any(|line| line.trim() == target) {
            result
                .actions
                .push(format!("Rust standard library installed: {target}"));
        } else {
            result.ok = false;
            result.warnings.push(format!(
                "missing Rust target {target}; install explicitly with rustup target add {target}"
            ));
        }
        if !cfg!(windows) {
            result.ok = false;
            result.warnings.push(format!("{target}: MSVC/Windows SDK and native execution require a Windows runner; target installation is insufficient"));
            continue;
        }
        let native_target = native_windows_target(std::env::consts::ARCH);
        if native_target != Some(target.as_str()) {
            result.ok = false;
            result.warnings.push(format!(
                "{target}: installed for cross-compilation, but this Windows host cannot execute that architecture natively"
            ));
            continue;
        }
        match compile_and_execute_probe(root, target) {
            Ok(()) => result.actions.push(format!(
                "{target}: MSVC linker, Windows SDK and native execution probe passed"
            )),
            Err(error) => {
                result.ok = false;
                result.warnings.push(format!(
                    "{target}: native Rust compile/execute probe failed: {error}"
                ));
            }
        }
        result.warnings.push(format!("{target}: linking, native dependencies and launch remain unverified until the target application build/run succeeds"));
    }
}

fn native_windows_target(arch: &str) -> Option<&'static str> {
    match arch {
        "x86_64" => Some("x86_64-pc-windows-msvc"),
        "aarch64" => Some("aarch64-pc-windows-msvc"),
        _ => None,
    }
}

fn compile_and_execute_probe(root: &Path, target: &str) -> Result<(), String> {
    let directory = tempfile::Builder::new()
        .prefix("knowme-builder-native-probe-")
        .tempdir()
        .map_err(|error| format!("could not create temporary directory: {error}"))?;
    let source = directory.path().join("main.rs");
    let executable = directory.path().join("native-probe.exe");
    fs::write(
        &source,
        b"fn main() { println!(\"knowme-builder-native-probe\"); }\n",
    )
    .map_err(|error| format!("could not write probe source: {error}"))?;
    let compile = Command::new("rustc")
        .args(["--edition=2024", "--target", target])
        .arg(&source)
        .arg("-o")
        .arg(&executable)
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not start rustc: {error}"))?;
    if !compile.status.success() {
        return Err(command_failure(&compile));
    }
    let execution = Command::new(&executable)
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not launch probe: {error}"))?;
    if !execution.status.success() {
        return Err(command_failure(&execution));
    }
    if String::from_utf8_lossy(&execution.stdout).trim() != "knowme-builder-native-probe" {
        return Err("probe returned unexpected output".to_owned());
    }
    Ok(())
}

fn command_failure(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = stderr.trim();
    if detail.is_empty() {
        format!("process exited with {}", output.status)
    } else {
        format!("process exited with {}: {detail}", output.status)
    }
}

#[cfg(test)]
mod tests {
    use super::native_windows_target;

    #[test]
    fn maps_supported_windows_host_architectures() {
        assert_eq!(
            native_windows_target("x86_64"),
            Some("x86_64-pc-windows-msvc")
        );
        assert_eq!(
            native_windows_target("aarch64"),
            Some("aarch64-pc-windows-msvc")
        );
        assert_eq!(native_windows_target("x86"), None);
    }
}
