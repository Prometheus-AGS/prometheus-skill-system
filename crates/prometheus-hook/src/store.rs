//! Resolving a bundle to the generation directory it names.
//!
//! This is the same resolution `shared/scripts/hook-runtime-v1.sh` performs, in
//! a binary that needs no shell, no `awk`, and no `sha256sum`. The order of the
//! checks is deliberately identical, because the two must agree: a store the
//! shell runtime accepts and this one rejects (or the reverse) would be a much
//! worse failure than either being wrong on its own.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// The one interpreter a dispatcher may be launched with.
pub const ALLOWED_INTERPRETER: &str = "bash";
/// The one dispatcher path a manifest may name.
pub const ALLOWED_DISPATCHER: &str = "shared/scripts/generated/hook-dispatch-v1.sh";

#[derive(Debug)]
pub struct HookError {
    pub code: &'static str,
    pub message: String,
}

pub fn err(code: &'static str, message: impl Into<String>) -> HookError {
    HookError { code, message: message.into() }
}

pub type Result<T> = std::result::Result<T, HookError>;

/// A resolved, verified generation.
#[derive(Debug)]
pub struct Resolved {
    pub generation: String,
    pub dispatcher: PathBuf,
    pub interpreter: String,
}

/// Reduce a Windows verbatim path to its ordinary spelling.
///
/// `std::fs::canonicalize` always returns `\\?\C:\...`, and `read_link` returns
/// it for a junction. `Path` treats `VerbatimDisk('C')` and `Disk('C')` as
/// different components, so a containment check comparing one spelling against
/// the other rejects every valid bundle. Normalizing never widens the check: a
/// path outside the store is outside it in either spelling.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path.to_path_buf()
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// `generations/<sha256>` and nothing else.
///
/// Validated before any filesystem call is made with it, so a traversal or an
/// absolute path never reaches `join`.
fn generation_from_pointer(target: &str) -> Result<&str> {
    let rest = target
        .strip_prefix("generations/")
        .ok_or_else(|| err("INVALID_POINTER", "activation pointer does not name a generation"))?;
    if !is_sha256(rest) {
        return Err(err("INVALID_POINTER", "activation pointer does not name a generation"));
    }
    Ok(rest)
}

pub fn plugin_root() -> PathBuf {
    if let Ok(root) = std::env::var("PROMETHEUS_PLUGIN_ROOT") {
        if !root.is_empty() {
            return PathBuf::from(root);
        }
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    PathBuf::from(home).join(".prometheus/plugins/prometheus-skill-pack")
}

/// Resolve a bundle id to its generation directory.
///
/// The pointer FILE is authoritative; the convenience link is consulted only
/// when no pointer file exists, which is exactly a store written before pointer
/// files were introduced. Resolving it keeps such a store working.
fn resolve_generation_root(root: &Path, bundle: &str) -> Result<PathBuf> {
    let generations = root.join("generations");
    let canonical_generations = generations
        .canonicalize()
        .map_err(|e| err("BROKEN_STORE", format!("generation store is missing: {e}")))?;

    let pointer = root.join("pointers").join("bundles").join(bundle);
    let candidate = match fs::read_to_string(&pointer) {
        Ok(text) => {
            let target = text.lines().next().unwrap_or("").trim().to_owned();
            let generation = generation_from_pointer(&target)?;
            generations.join(generation)
        }
        Err(_) => {
            let link = root.join("bundles").join(bundle);
            let meta = fs::symlink_metadata(&link)
                .map_err(|_| err("NOT_ACTIVATED", "bundle index is missing"))?;
            if !meta.file_type().is_symlink() {
                return Err(err("NOT_ACTIVATED", "bundle index is missing"));
            }
            link
        }
    };

    let canonical = candidate
        .canonicalize()
        .map_err(|e| err("BROKEN_BUNDLE", format!("bundle index cannot be resolved: {e}")))?;
    // Both operands reduced to one spelling before the comparison.
    if !strip_verbatim_prefix(&canonical).starts_with(strip_verbatim_prefix(&canonical_generations))
    {
        return Err(err("ESCAPING_BUNDLE", "bundle index escapes generation store"));
    }
    if canonical == canonical_generations {
        return Err(err("ESCAPING_BUNDLE", "bundle index escapes generation store"));
    }
    // Return the ORDINARY spelling, not the verbatim one `canonicalize` produced.
    //
    // Everything downstream is derived from this: the manifest path, and the
    // dispatcher path that is handed to `bash` as an argument. msys2 cannot
    // represent `\\?\C:\...` at all -- `cygpath -u` turns it into `/c/?/C:/...`
    // -- so passing it through produced a mangled filename and exit 126. The
    // containment check above already ran on the stripped spelling, so nothing
    // is weakened by returning it.
    Ok(strip_verbatim_prefix(&canonical))
}

fn manifest_string(manifest: &serde_json::Value, key: &str) -> Result<String> {
    manifest
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .ok_or_else(|| err("MISSING_MANIFEST", format!("generation manifest omits {key}")))
}

/// Verify a bundle and return everything needed to dispatch it.
pub fn resolve(root: &Path, bundle: &str) -> Result<Resolved> {
    if !is_sha256(bundle) {
        return Err(err("INVALID_BUNDLE", "bundle id is not sha256"));
    }
    let generation_root = resolve_generation_root(root, bundle)?;
    let generation = generation_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    if !is_sha256(&generation) {
        return Err(err("INVALID_GENERATION", "generation directory is not sha256"));
    }

    let manifest_path = generation_root.join("manifest.json");
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|_| err("MISSING_MANIFEST", "generation manifest is missing"))?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| err("MISSING_MANIFEST", format!("generation manifest is invalid: {e}")))?;

    // The shell runtime reads these from the flattened `hookRuntime` receipt via
    // awk, which sees the first match at any nesting depth. Reading real JSON
    // means looking in the right place.
    let receipt = manifest.get("hookRuntime").unwrap_or(&serde_json::Value::Null);

    if manifest_string(receipt, "bundleId")? != bundle {
        return Err(err("BUNDLE_MISMATCH", "manifest bundle differs"));
    }
    if manifest_string(&manifest, "generation")? != generation {
        return Err(err("GENERATION_MISMATCH", "manifest generation differs"));
    }
    if manifest_string(receipt, "abi")? != "hook-runtime-v1" {
        return Err(err("ABI_MISMATCH", "unsupported dispatcher ABI"));
    }
    let dispatcher_path = manifest_string(receipt, "dispatcherPath")?;
    if dispatcher_path != ALLOWED_DISPATCHER {
        return Err(err("DISPATCHER_PATH", "dispatcher path is not allowlisted"));
    }
    let interpreter = manifest_string(receipt, "dispatcherInterpreter")?;
    if interpreter != ALLOWED_INTERPRETER {
        return Err(err("DISPATCHER_INTERPRETER", "dispatcher interpreter is not allowlisted"));
    }
    let expected_digest = manifest_string(receipt, "dispatcherSha256")?;

    let dispatcher = generation_root.join(dispatcher_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    // Execution eligibility comes from the manifest, never from a filesystem
    // executable bit: on a volume that cannot record one, every file reads as
    // non-executable including this dispatcher.
    let bytes = fs::read(&dispatcher).map_err(|_| err("MISSING_DISPATCHER", "dispatcher is missing"))?;
    let actual = hex(&Sha256::digest(&bytes));
    if actual != expected_digest {
        return Err(err("DISPATCHER_HASH", "dispatcher hash differs"));
    }

    Ok(Resolved { generation, dispatcher, interpreter })
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from_digit((byte >> 4) as u32, 16).unwrap());
        out.push(char::from_digit((byte & 0x0f) as u32, 16).unwrap());
    }
    out
}
