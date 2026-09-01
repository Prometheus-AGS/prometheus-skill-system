//! Shell-free hot-path hook dispatch.
//!
//! # Why this binary exists
//!
//! Claude Code hooks have two forms. Shell form is a single `command` string
//! handed to a shell -- `sh -c` on macOS and Linux, and on Windows **PowerShell
//! whenever Git Bash is not installed**. Every hook this pack emits today is
//! `bash -c '<multi-line bash>'`, which means the pack works on Windows only
//! because Git Bash happens to be present; without it those 31 entries are
//! handed to PowerShell, which does not parse them.
//!
//! Exec form -- a `command` plus an `args` array -- is spawned directly with no
//! shell on any platform, and path placeholders are substituted as plain
//! strings. That retires the entire class of quoting defects at once: a plugin
//! root containing backslashes, `$`, or backticks arrives here verbatim because
//! nothing tokenizes it.
//!
//! The price of exec form is that `command` must be something the operating
//! system can actually start. It cannot be a `.sh`, and on Windows it cannot be
//! a `.bat` or `.cmd` either. Hence a compiled dispatcher.
//!
//! # What it deliberately does not do
//!
//! It does not replace the leaf hook scripts. Those stay shell and are launched
//! by the explicit interpreter the signed manifest names, which is the seam the
//! design accepted: hot path compiled, cold path shell.

mod store;

use std::io::Write;
use std::process::{Command, ExitCode};

use store::{err, HookError, Result};

fn fail(error: &HookError, bundle: &str) -> ExitCode {
    // The same JSON error envelope the shell runtime emits, so a caller that
    // parses one parses the other.
    let _ = writeln!(
        std::io::stderr(),
        r#"{{"status":"HOOK_RUNTIME_ERROR","code":"{}","message":"{}","bundle":"{}"}}"#,
        error.code,
        error.message.replace('\\', "\\\\").replace('"', "\\\""),
        bundle
    );
    // 78 is EX_CONFIG: the hook is misconfigured, not the tool call.
    ExitCode::from(78)
}

/// Minimal `--flag value` parsing.
///
/// Hand-rolled rather than pulled from a crate: the grammar is six flags that
/// never change, and this binary is spawned on every PreToolUse, so every
/// dependency is paid for in startup time.
fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn run() -> std::result::Result<ExitCode, (HookError, String)> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = argv.first().cloned().unwrap_or_default();
    let bundle = flag(&argv, "--bundle").unwrap_or_default();
    let root = store::plugin_root();

    match subcommand.as_str() {
        "resolve" => {
            let resolved = store::resolve(&root, &bundle).map_err(|e| (e, bundle.clone()))?;
            println!(
                r#"{{"status":"ok","bundle":"{}","generation":"{}","abi":"hook-runtime-v1"}}"#,
                bundle, resolved.generation
            );
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            let hook = flag(&argv, "--hook").unwrap_or_default();
            let harness = flag(&argv, "--harness").unwrap_or_default();
            if hook.is_empty() {
                return Err((err("MISSING_HOOK", "hook id is required"), bundle));
            }
            if harness.is_empty() {
                return Err((err("MISSING_HARNESS", "harness is required"), bundle));
            }
            let resolved = store::resolve(&root, &bundle).map_err(|e| (e, bundle.clone()))?;
            dispatch(&resolved, &hook, &harness).map_err(|e| (e, bundle))
        }
        "version" | "--version" => {
            println!("prometheus-hook {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        other => Err((
            err("INVALID_ARGUMENT", format!("unknown subcommand: {other}")),
            bundle,
        )),
    }
}

/// Launch the dispatcher by the interpreter the signed receipt names.
///
/// The argument vector is explicit, so the hook id and harness reach the
/// dispatcher exactly as written even when they contain characters a shell
/// would treat as syntax.
fn dispatch(resolved: &store::Resolved, hook: &str, harness: &str) -> Result<ExitCode> {
    let status = Command::new(&resolved.interpreter)
        .arg(&resolved.dispatcher)
        .arg("--hook")
        .arg(hook)
        .arg("--harness")
        .arg(harness)
        .status()
        .map_err(|e| {
            err(
                "MISSING_INTERPRETER",
                format!("cannot launch {}: {e}", resolved.interpreter),
            )
        })?;
    Ok(ExitCode::from(status.code().unwrap_or(70) as u8))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err((error, bundle)) => fail(&error, &bundle),
    }
}

#[cfg(test)]
mod tests {
    use super::store;
    use std::path::Path;

    #[test]
    fn sha256_matches_known_answers() {
        // The dispatcher digest gate is only as good as this. Known-answer
        // vectors from FIPS 180-2.
        use sha2::{Digest, Sha256};
        assert_eq!(
            store::hex(&Sha256::digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            store::hex(&Sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verbatim_prefixes_reduce_to_one_spelling() {
        assert_eq!(
            store::strip_verbatim_prefix(Path::new(r"\\?\C:\store\x")),
            Path::new(r"C:\store\x")
        );
        assert_eq!(
            store::strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share")),
            Path::new(r"\\server\share")
        );
        // Anything else is returned unchanged, including every POSIX path.
        assert_eq!(
            store::strip_verbatim_prefix(Path::new("/srv/store")),
            Path::new("/srv/store")
        );
    }
}
