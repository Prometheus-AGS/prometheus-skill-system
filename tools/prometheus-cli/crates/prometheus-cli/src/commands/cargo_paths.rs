//! Where cargo writes release binaries on this machine.
//!
//! `<workspace>/target/release` is only cargo's default. A machine that keeps build
//! output elsewhere says so in `CARGO_TARGET_DIR`, the variable cargo itself reads,
//! so the build and every reader of its output agree by construction. Reading a
//! hardcoded in-repository path instead finds either nothing or a binary left behind
//! by an older build: on 2026-11-16 `doctor` reported the hash of a stale 1.14.0
//! `prometheus` as the "source" of an installed 1.15.0.
//!
//! A target directory set only in a cargo config file (`build.target-dir`) is not
//! visible here. Export the same path as `CARGO_TARGET_DIR` on such a machine.

use std::env;
use std::path::{Path, PathBuf};

/// The variable cargo reads for its target directory.
const TARGET_DIR_VARIABLE: &str = "CARGO_TARGET_DIR";

/// The directory holding release binaries built from `workspace`.
///
/// `CARGO_TARGET_DIR` wins when it is set and non-empty. A relative value is resolved
/// against `workspace`, which is where the installers run cargo from; an absolute one
/// replaces it. Without the variable this is cargo's default,
/// `<workspace>/target/release`.
pub(crate) fn release_dir(workspace: &Path) -> PathBuf {
    let target_dir = env::var_os(TARGET_DIR_VARIABLE)
        .filter(|value| !value.is_empty())
        .map_or_else(|| workspace.join("target"), |value| workspace.join(value));
    target_dir.join("release")
}
