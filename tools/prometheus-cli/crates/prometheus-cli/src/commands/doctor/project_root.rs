//! Deliberate project-root resolution for the doctor's snapshot checks.
//!
//! The caller's working directory is not a reliable project identity: release
//! installs run from a git worktree, which has no committed snapshot store of its
//! own. Resolution order:
//!
//! 1. `PROMETHEUS_PROJECT_ROOT`
//! 2. the nearest ancestor of the working directory with `.prometheus/project.json`
//! 3. the main worktree, when the working directory is a linked worktree
//! 4. the working directory itself
//!
//! `.prometheus/project.json` is tracked in git, so a linked worktree contains it
//! and step 2 alone would resolve to the worktree. A linked-worktree candidate
//! that lacks a project snapshot pointer therefore yields to the main worktree
//! when that one has it.

use super::command_stdout;
use std::path::{Path, PathBuf};

const SNAPSHOT_POINTER: &str = ".prometheus/knowledge/.prompt-snapshots/project/current";

#[derive(Debug, Clone)]
pub(super) struct ProjectRoot {
    pub path: PathBuf,
    pub source: &'static str,
    /// Set when `path` (or the directory it was derived from) is a linked worktree.
    pub main_worktree: Option<PathBuf>,
}

fn git_path(dir: &Path, flag: &str) -> Option<PathBuf> {
    let dir = dir.to_str()?;
    command_stdout(&[
        "git",
        "-C",
        dir,
        "rev-parse",
        "--path-format=absolute",
        flag,
    ])
    .filter(|out| !out.is_empty())
    .map(PathBuf::from)
}

/// The main worktree's root when `dir` is inside a linked worktree.
fn linked_main_worktree(dir: &Path) -> Option<PathBuf> {
    let common = git_path(dir, "--git-common-dir")?;
    let git_dir = git_path(dir, "--git-dir")?;
    if common.canonicalize().ok()? == git_dir.canonicalize().ok()? {
        return None;
    }
    common.parent().map(Path::to_path_buf)
}

fn nearest_project_ancestor(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(".prometheus/project.json").is_file())
        .map(Path::to_path_buf)
}

fn has_pointer(root: &Path) -> bool {
    root.join(SNAPSHOT_POINTER).is_file()
}

pub(super) fn resolve_project_root() -> ProjectRoot {
    if let Some(explicit) = std::env::var_os("PROMETHEUS_PROJECT_ROOT") {
        let path = PathBuf::from(explicit);
        let main_worktree = linked_main_worktree(&path);
        return ProjectRoot {
            path,
            source: "PROMETHEUS_PROJECT_ROOT",
            main_worktree,
        };
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (candidate, source) = match nearest_project_ancestor(&cwd) {
        Some(dir) => (dir, ".prometheus/project.json ancestor"),
        None => (cwd, "working directory"),
    };
    let Some(main) = linked_main_worktree(&candidate) else {
        return ProjectRoot {
            path: candidate,
            source,
            main_worktree: None,
        };
    };
    if !has_pointer(&candidate) && has_pointer(&main) {
        return ProjectRoot {
            path: main.clone(),
            source: "main worktree (linked worktree has no project snapshot)",
            main_worktree: Some(main),
        };
    }
    ProjectRoot {
        path: candidate,
        source,
        main_worktree: Some(main),
    }
}
